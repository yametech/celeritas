use parser::parse_redis_value;
use server::Server;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Server::new("127.0.0.1:8080").serve()?;

    let s = b"*3\r\n$3\r\nset\r\n$2\r\nxy\r\n$2\r\nab\r\n";
    let redis_value = parse_redis_value(&s[..])?;
    println!("{:?}", std::str::from_utf8(&redis_value.as_bytes()));
    // assert_eq!(redis_value.get_char(), '*');

    Ok(())
}
