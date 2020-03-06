use futures::SinkExt;
use parser::{Event, RedisCodec};
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
                    Event::Array(value) => {
                        println!(
                            "Event {:?}",
                            String::from_utf8(Event::Array(value).as_bytes())
                        );
                        if let Err(error) = frame.send(Event::String(b"+OK".to_vec())).await {
                            println!("An error occured {:?}", error);
                            return;
                        }
                    }
                    _ => {
                        // nop
                    }
                }
            }
        });
    }
}
