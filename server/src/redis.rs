use futures::SinkExt;
use parser::{parse_array, RedisCodec, ValueEvent};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::Framed;

#[tokio::main]
pub async fn redis_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:7000").await?;
    println!("listening on port 7000");
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut frame = Framed::new(socket, RedisCodec::new());
            while let Some(Ok(event)) = frame.next().await {
                match event {
                    ValueEvent::Array(value) => {
                        if let Ok((cmd, size)) = parse_array(&ValueEvent::Array(value).as_bytes()) {
                            println!("cmd = {:?}", std::str::from_utf8(&*cmd.get_data()));
                            if size < 1 {
                                continue;
                            }
                            if let Ok(op) = cmd.get_str(0) {
                                match &*op.to_lowercase() {
                                    "set" => {}
                                    "command" => {
                                        let cmd = ValueEvent::Array(vec![
                                            ValueEvent::Blob(b"watch".to_vec()),
                                            ValueEvent::Number(-2_i64),
                                        ]);
                                        if let Err(error) = frame.send(cmd).await {
                                            println!("An error occured {:?}", error);
                                            return;
                                        }
                                    }
                                    _ => break,
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
