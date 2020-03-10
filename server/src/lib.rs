use futures::future::try_join;
use futures::FutureExt;

// use parser::Command;
// use resp::{Value, ValuePair};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

pub struct Server<'a> {
    listen_addr: &'a str,
    proxy_addr_pool: Vec<&'a str>,
    // raft_node: Node,
    // cmd_handle: Option<Handler>,
}

impl<'a> Server<'a> {
    pub fn new(listen_addr: &'a str) -> Self {
        Server {
            listen_addr,
            proxy_addr_pool: vec!["127.0.0.1:6379"],
            // cmd_handle: None,
        }
    }

    #[tokio::main]
    pub async fn serve(&self) -> Result<(), Box<dyn Error>> {
        println!("Listening on: {} ", self.listen_addr,);
        let mut listener = TcpListener::bind(self.listen_addr).await?;

        while let Ok((inbound, _)) = listener.accept().await {
            let addr = self.proxy_addr_pool[0].to_owned();
            tokio::spawn(Server::transfer(inbound, addr.clone()).map(|r| {
                if let Err(e) = r {
                    println!("Failed to transfer; error={}", e);
                }
            }));
        }

        Ok(())
    }

    async fn transfer(inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
        let outbound = TcpStream::connect(proxy_addr).await?;

        let handlers = Handlers(vec![]);

        Ok(Server::copy(inbound, outbound, handlers).await?)
    }

    async fn copy(
        mut _i: TcpStream,
        mut _o: TcpStream,
        _h: Handlers,
    ) -> Result<(), Box<dyn Error>> {
        let (mut ri, mut wi) = _i.split();
        let (mut ro, mut wo) = _o.split();

        // match parse_from_client(&mut ri) {
        //     Ok(cmd) => {}
        //     Err(err) => {
        //         eprintln!("parse command error: {:?}", err);
        //         if let Err(e) = wo
        //             .write(&Value::Error(format!("{:?}", err)).as_bytes())
        //             .await
        //         {
        //             eprintln!("write resp error: {:?}", e);
        //         };
        //     }
        // };
        let client_to_server = io::copy(&mut ri, &mut wo);
        let server_to_client = io::copy(&mut ro, &mut wi);
        // handlers.0.iter().map(|h| h(&ri, &wo)).await?;
        try_join(client_to_server, server_to_client).await?;
        Ok(())
    }
}

struct Handlers(Vec<fn(_input: &mut TcpStream, _output: &mut TcpStream)>);

async fn parse_from_client(
    input: &mut tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn write_to_client(
    data: &mut [u8],
    output: &mut tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

mod redis;
pub use redis::redis_main;
