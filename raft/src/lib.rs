#[macro_use]
extern crate slog;
use raft::{eraftpb::ConfState, prelude::*, storage::MemStorage, Config, Peer, StateRole};
use slog::Drain;

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{str, thread};

struct Node {
    // None if the raft is not initialized.
    raft_group: Option<RawNode<MemStorage>>,
    my_mailbox: Receiver<Message>,
    mailboxes: HashMap<u64, Sender<Message>>,
    // Key-value pairs after applied. `MemStorage` only contains raft logs,
    // so we need an additional storage engine.
    kv_pairs: HashMap<u16, String>,
    logger: slog::Logger,
}

// impl Node {
//     // Create a raft leader only with itself in its configuration.
//     fn create_raft_leader(
//         id: u64,
//         my_mailbox: Receiver<Message>,
//         mailboxes: HashMap<u64, Sender<Message>>,
//         logger: &slog::Logger,
//     ) -> Self {
//         let mut cfg = example_config();
//         cfg.id = id;
//         let logger = logger.new(o!("tag" => format!("peer_{}", id)));

//         // let storage = MemStorage::new_with_conf_state(ConfState::from((vec![id], vec![])));
//         let storage = MemStorage::new();
//         let peers = vec![Peer{id:id,vec!()}];
//         let raft_group = Some(RawNode::new(&cfg, storage, peers).unwrap());
//         Node {
//             raft_group,
//             my_mailbox,
//             mailboxes,
//             kv_pairs: Default::default(),
//             logger: logger,
//         }
//     }

//     // Create a raft follower.
//     fn create_raft_follower(
//         my_mailbox: Receiver<Message>,
//         mailboxes: HashMap<u64, Sender<Message>>,
//     ) -> Self {
//         Node {
//             raft_group: None,
//             my_mailbox,
//             mailboxes,
//             kv_pairs: Default::default(),
//             logger: (),
//         }
//     }

//     // Initialize raft for followers.
//     fn initialize_raft_from_message(&mut self, msg: &Message, logger: &slog::Logger) {
//         if !is_initial_msg(msg) {
//             return;
//         }
//         let mut cfg = example_config();
//         cfg.id = msg.to;
//         let logger = logger.new(o!("tag" => format!("peer_{}", msg.to)));
//         let storage = MemStorage::new();
//         self.raft_group = Some(RawNode::new(&cfg, storage, &logger).unwrap());
//     }

//     // Step a raft message, initialize the raft if need.
//     fn step(&mut self, msg: Message, logger: &slog::Logger) {
//         if self.raft_group.is_none() {
//             if is_initial_msg(&msg) {
//                 self.initialize_raft_from_message(&msg, &logger);
//             } else {
//                 return;
//             }
//         }
//         let raft_group = self.raft_group.as_mut().unwrap();
//         let _ = raft_group.step(msg);
//     }
// }

// fn on_ready(
//     raft_group: &mut RawNode<MemStorage>,
//     kv_pairs: &mut HashMap<u16, String>,
//     mailboxes: &HashMap<u64, Sender<Message>>,
//     proposals: &Mutex<VecDeque<Proposal>>,
//     logger: &slog::Logger,
// ) {
//     if !raft_group.has_ready() {
//         return;
//     }
//     let store = raft_group.raft.raft_log.store.clone();

//     // Get the `Ready` with `RawNode::ready` interface.
//     let mut ready = raft_group.ready();

//     // Persistent raft logs. It's necessary because in `RawNode::advance` we stabilize
//     // raft logs to the latest position.
//     if let Err(e) = store.wl().append(ready.entries()) {
//         error!(
//             logger,
//             "persist raft log fail: {:?}, need to retry or panic", e
//         );
//         return;
//     }

//     // Apply the snapshot. It's necessary because in `RawNode::advance` we stabilize the snapshot.
//     if *ready.snapshot() != Snapshot::default() {
//         let s = ready.snapshot().clone();
//         if let Err(e) = store.wl().apply_snapshot(s) {
//             error!(
//                 logger,
//                 "apply snapshot fail: {:?}, need to retry or panic", e
//             );
//             return;
//         }
//     }

//     // Send out the messages come from the node.
//     for msg in ready.messages.drain(..) {
//         let to = msg.to;
//         if mailboxes[&to].send(msg).is_err() {
//             error!(
//                 logger,
//                 "send raft message to {} fail, let Raft retry it", to
//             );
//         }
//     }

//     // Apply all committed proposals.
//     if let Some(committed_entries) = ready.committed_entries.take() {
//         for entry in &committed_entries {
//             if entry.data.is_empty() {
//                 // From new elected leaders.
//                 continue;
//             }
//             if let EntryType::EntryConfChange = entry.get_entry_type() {
//                 // For conf change messages, make them effective.
//                 let mut cc = ConfChange::default();
//                 cc.merge_from_bytes(&entry.data).unwrap();
//                 let node_id = cc.node_id;
//                 match cc.get_change_type() {
//                     ConfChangeType::AddNode => raft_group.raft.add_node(node_id).unwrap(),
//                     ConfChangeType::RemoveNode => raft_group.raft.remove_node(node_id).unwrap(),
//                     ConfChangeType::AddLearnerNode => raft_group.raft.add_learner(node_id).unwrap(),
//                 }
//                 let cs = raft_group.raft.prs().configuration().to_conf_state();
//                 store.wl().set_conf_state(cs);
//             } else {
//                 // For normal proposals, extract the key-value pair and then
//                 // insert them into the kv engine.
//                 let data = str::from_utf8(&entry.data).unwrap();
//                 let reg = Regex::new("put ([0-9]+) (.+)").unwrap();
//                 if let Some(caps) = reg.captures(&data) {
//                     kv_pairs.insert(caps[1].parse().unwrap(), caps[2].to_string());
//                 }
//             }
//             if raft_group.raft.state == StateRole::Leader {
//                 // The leader should response to the clients, tell them if their proposals
//                 // succeeded or not.
//                 let proposal = proposals.lock().unwrap().pop_front().unwrap();
//                 proposal.propose_success.send(true).unwrap();
//             }
//         }
//         if let Some(last_committed) = committed_entries.last() {
//             let mut s = store.wl();
//             s.mut_hard_state().commit = last_committed.index;
//             s.mut_hard_state().term = last_committed.term;
//         }
//     }
//     // Call `RawNode::advance` interface to update position flags in the raft.
//     raft_group.advance(ready);
// }

// fn example_config() -> raft::Config {
//     Config {
//         election_tick: 10,
//         heartbeat_tick: 3,
//         ..Default::default()
//     }
// }

// // The message can be used to initialize a raft node or not.
// fn is_initial_msg(msg: &Message) -> bool {
//     let msg_type = msg.get_msg_type();
//     msg_type == MessageType::MsgRequestVote
//         || msg_type == MessageType::MsgRequestPreVote
//         || (msg_type == MessageType::MsgHeartbeat && msg.commit == 0)
// }

// struct Proposal {
//     normal: Option<(u16, String)>, // key is an u16 integer, and value is a string.
//     conf_change: Option<ConfChange>, // conf change.
//     transfer_leader: Option<u64>,
//     // If it's proposed, it will be set to the index of the entry.
//     proposed: u64,
//     propose_success: SyncSender<bool>,
// }

// impl Proposal {
//     fn conf_change(cc: &ConfChange) -> (Self, Receiver<bool>) {
//         let (tx, rx) = mpsc::sync_channel(1);
//         let proposal = Proposal {
//             normal: None,
//             conf_change: Some(cc.clone()),
//             transfer_leader: None,
//             proposed: 0,
//             propose_success: tx,
//         };
//         (proposal, rx)
//     }

//     fn normal(key: u16, value: String) -> (Self, Receiver<bool>) {
//         let (tx, rx) = mpsc::sync_channel(1);
//         let proposal = Proposal {
//             normal: Some((key, value)),
//             conf_change: None,
//             transfer_leader: None,
//             proposed: 0,
//             propose_success: tx,
//         };
//         (proposal, rx)
//     }
// }

// fn propose(raft_group: &mut RawNode<MemStorage>, proposal: &mut Proposal) {
//     let last_index1 = raft_group.raft.raft_log.last_index() + 1;
//     if let Some((ref key, ref value)) = proposal.normal {
//         let data = format!("put {} {}", key, value).into_bytes();
//         let _ = raft_group.propose(vec![], data);
//     } else if let Some(ref cc) = proposal.conf_change {
//         let _ = raft_group.propose_conf_change(vec![], cc.clone());
//     } else if let Some(_transferee) = proposal.transfer_leader {
//         // TODO: implement transfer leader.
//         unimplemented!();
//     }

//     let last_index2 = raft_group.raft.raft_log.last_index() + 1;
//     if last_index2 == last_index1 {
//         // Propose failed, don't forget to respond to the client.
//         proposal.propose_success.send(false).unwrap();
//     } else {
//         proposal.proposed = last_index1;
//     }
// }

// // Proposes some conf change for peers [2, 5].
// fn add_all_followers(proposals: &Mutex<VecDeque<Proposal>>) {
//     for i in 2..6u64 {
//         let mut conf_change = ConfChange::default();
//         conf_change.node_id = i;
//         conf_change.set_change_type(ConfChangeType::AddNode);
//         loop {
//             let (proposal, rx) = Proposal::conf_change(&conf_change);
//             proposals.lock().unwrap().push_back(proposal);
//             if rx.recv().unwrap() {
//                 break;
//             }
//             thread::sleep(Duration::from_millis(100));
//         }
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
