use server::{redis_main, Server};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Server::new("127.0.0.1:8080").serve()?;

    redis_main()?;
    Ok(())
}
