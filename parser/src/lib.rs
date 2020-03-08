extern crate resp;
use num_bigint::BigInt;
use resp::{resp_event_type, Float64, Value, ValuePair};

mod codec;
mod command;
mod error;
mod parse;

pub use codec::RedisCodec;
pub use command::{Argument, Command};
pub use error::ParseError;
pub use parse::{parse_array, parse_redis_value};
pub use resp::Value as ValueEvent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_test_array() {
        let array = b"*3\r\n$3\r\nset\r\n$2\r\nxy\r\n$2\r\nab\r\n";
        assert_eq!(
            parse_redis_value(&array[..]).unwrap().as_bytes(),
            array.to_vec()
        );
    }

    #[test]
    fn roundtrip_test_map() {
        let map = b"%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n";
        assert_eq!(
            parse_redis_value(&map[..]).unwrap().as_bytes(),
            map.to_vec()
        );
    }

    #[test]
    fn roundtrip_tset_set() {
        let set = b"~5\r\n+orange\r\n+apple\r\n#t\r\n:100\r\n:999\r\n";
        assert_eq!(
            parse_redis_value(&set[..]).unwrap().as_bytes(),
            set.to_vec()
        );
    }

    #[test]
    fn roundtrip_test_all_base_type() {
        let set = b"~6\r\n+orange\r\n#t\r\n:1111\r\n(321328139271389216321689\r\n,1.23\r\n~1\r\n*3\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\n1\r\n";
        assert_eq!(
            parse_redis_value(&set[..]).unwrap().as_bytes(),
            set.to_vec()
        );
    }

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
