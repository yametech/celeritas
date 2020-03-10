use futures::SinkExt;
use parser::{parse_array, Command, RedisCodec, Value};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::Framed;

use std::collections::HashMap;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

// use std::io::{Error as IOError, ErrorKind};

/// This database will be shared via `Arc`, so to mutate the internal map we're
/// going to use a `Mutex` for interior mutability.
struct Database {
    map: Mutex<HashMap<String, String>>,
}

fn handle(cmd: &Command, db: &Arc<Database>) -> Result<(), Box<dyn Error>> {
    let (key, value) = (
        cmd.get_str(0).unwrap().to_string(),
        cmd.get_str(1).unwrap().to_string(),
    );

    let mut db = db.map.lock().unwrap();
    db.insert(key.clone(), value.clone());

    Ok(())
}
#[tokio::main]
pub async fn redis_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("listening on port 7000");

    let mut initial_db = HashMap::new();
    initial_db.insert("foo".to_string(), "bar".to_string());
    let db = Arc::new(Database {
        map: Mutex::new(initial_db),
    });

    loop {
        let (socket, _) = listener.accept().await?;

        let db = db.clone();

        tokio::spawn(async move {
            let mut frame = Framed::new(socket, RedisCodec::new());

            while let Some(Ok(event)) = frame.next().await {
                match event {
                    Value::Array(value) => {
                        if let Ok((cmd, _)) = parse_array(&Value::Array(value).as_bytes()) {
                            // println!("cmd = {:?}", std::str::from_utf8(&*cmd.get_data()));

                            if let Ok(op) = cmd.get_str(0) {
                                match &*op.to_uppercase() {
                                    "SET" => {
                                        println!("{:?}", std::str::from_utf8(&*cmd.get_data()));
                                        if let Err(e) =
                                            frame.send(Value::String(b"OK".to_vec())).await
                                        {
                                            println!("An set op error occured {:?}", e);
                                            return;
                                        }

                                        if let Err(e) = handle(&cmd, &db) {
                                            println!("Handle set op error {:?}", e);
                                            break;
                                        }
                                    }
                                    // "COMMAND" => {
                                    //     // if let Err(e) =
                                    //     //     frame.send(Value::String(b"OK".to_vec())).await
                                    //     // {
                                    //     //     println!("An command op error occured {:?}", e);
                                    //     //     break;
                                    //     // }
                                    //     println!("Response to client command");
                                    // }
                                    _ => {
                                        if let Err(e) = frame
                                            .send(Value::String(b"not support command".to_vec()))
                                            .await
                                        {
                                            println!(
                                                "Response to other command  {:?} error: {:?}",
                                                String::from_utf8(cmd.get_data().to_vec()),
                                                e,
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }

                    _ => {
                        // nop
                        println!("{:?}", event);
                    }
                }
            }
        });
    }
}
