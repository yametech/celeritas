extern crate resp;
use resp::{resp_type, Value, ValuePair};

mod codec;
mod command;
mod error;
mod parse;

pub use codec::RedisCodec;
pub use command::{Argument, Command};
pub use error::ParseError;
pub use parse::parse_array;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_array() {
        let s = b"*3\r\n$3\r\nset\r\n$2\r\nxy\r\n$2\r\nab\r\n";
        let (p1, len) = parse_array(s).unwrap();
        assert_eq!(len, 29);
        assert_eq!(p1.get_str(0).unwrap(), "set");
        assert_eq!(p1.get_str(1).unwrap(), "xy");
        assert_eq!(p1.get_str(2).unwrap(), "ab");
    }
}
