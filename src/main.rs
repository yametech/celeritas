#[macro_use(sendlog)]
extern crate logger;

use logger::{Level, Logger};
use std::sync::mpsc::channel;

fn main() {
    let (tx, rx) = channel();
    let logger = Logger::channel(Level::Debug, tx);
    let sender = logger.sender();

    sendlog!(sender, Debug, "hello {}", "Celeritas");
    println!(
        "{:?}",
        rx.recv()
            .unwrap()
            .iter()
            .map(|&x| x as char)
            .collect::<String>()
    );
}
