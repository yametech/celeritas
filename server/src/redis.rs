use futures::SinkExt;
use parser::{parse_array, Command, RedisCodec, Value};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

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
            while let Some(event) = frame.next().await {
                match event {
                    Ok(Value::Array(value)) => {
                        println!(
                            "array => {:?}",
                            std::str::from_utf8(&Value::Array(value).as_bytes())
                        );
                        if let Err(e) = frame.send(parser::write_simple(&"OK")).await {
                            println!("resp ok error {:?}", e);
                        }
                    }
                    Err(e) => {
                        println!("error on decoding from socket; error = {:?}", e);
                    }
                    _ => {
                        println!("unknow event");
                    }
                }
            }
        });
    }
}
