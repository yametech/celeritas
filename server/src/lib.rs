use futures::future::try_join;
use futures::FutureExt;

use std::error::Error;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

pub struct Server<'a> {
    listen_addr: &'a str,
    addr_pool: Vec<&'a str>,
    // raft_node: Node,
}

impl<'a> Server<'a> {
    pub fn new(listen_addr: &'a str) -> Self {
        Server {
            listen_addr: listen_addr,
            addr_pool: vec!["127.0.0.1:6379"],
        }
    }
    #[tokio::main]
    pub async fn serve(&self) -> Result<(), Box<dyn Error>> {
        println!("Listening on: {} ", self.listen_addr,);
        let mut listener = TcpListener::bind(self.listen_addr).await?;

        while let Ok((inbound, _)) = listener.accept().await {
            let addr = self.addr_pool[0].to_owned();
            let transfer = Server::transfer(inbound, addr.clone()).map(|r| {
                if let Err(e) = r {
                    println!("Failed to transfer; error={}", e);
                }
            });

            tokio::spawn(transfer);
        }

        Ok(())
    }
    async fn transfer(inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
        let outbound = TcpStream::connect(proxy_addr).await?;
        let handlers = Handlers(vec![]);
        Ok(Self::iocopy(inbound, outbound, handlers).await?)
    }

    async fn iocopy(
        mut _i: TcpStream,
        mut _o: TcpStream,
        _h: Handlers,
    ) -> Result<(), Box<dyn Error>> {
        let (mut ri, mut wi) = _i.split();
        let (mut ro, mut wo) = _o.split();
        let client_to_server = io::copy(&mut ri, &mut wo);
        let server_to_client = io::copy(&mut ro, &mut wi);
        // handlers.0.iter().map(|h| h(&ri, &wo)).await?;
        try_join(client_to_server, server_to_client).await?;
        Ok(())
    }
}

struct Handlers(Vec<fn(_input: &mut TcpStream, _output: &mut TcpStream)>);
