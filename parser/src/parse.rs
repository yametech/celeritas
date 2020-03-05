use super::*;

use linked_hash_map::LinkedHashMap;
use std::io::{self, BufReader, Read};

pub struct Parser<T> {
    reader: T,
}

impl<'a, T: Read> Parser<T> {
    /// Creates a new parser that parses the data behind the reader.  More
    /// than one value can be behind the reader in which case the parser can
    /// be invoked multiple times.  In other words: the stream does not have
    /// to be terminated.
    pub fn new(reader: T) -> Parser<T> {
        Parser { reader: reader }
    }

    /// parses a single value out of the stream.  If there are multiple
    /// values you can call this multiple times.  If the reader is not yet
    /// ready this will block.
    pub fn parse_value(&mut self) -> Result<Value, ParseError> {
        let b = self.read_byte()?;
        match b as char {
            '+' => self.parse_simple_string(),
            ':' => self.parse_int(),
            '$' => self.parse_blob(),
            '*' => self.parse_array(),
            '-' => self.parse_error(),
            '_' => self.parse_null(),
            '%' => self.parse_map(),
            '~' => self.parse_set(),
            '(' => self.parse_bigint(),
            '#' => self.parse_boolean(),
            ',' => self.parse_double(),
            _ => Err(ParseError::InvalidArgument),
        }
    }

    #[inline]
    fn expect_char(&mut self, refchar: char) -> Result<(), ParseError> {
        if self.read_byte().unwrap() as char == refchar {
            Ok(())
        } else {
            Err(ParseError::Incomplete)
        }
    }

    #[inline]
    fn expect_newline(&mut self) -> Result<(), ParseError> {
        match self.read_byte().unwrap() as char {
            '\n' => Ok(()),
            '\r' => self.expect_char('\n'),
            _ => Err(ParseError::Incomplete),
        }
    }

    fn read_byte(&mut self) -> Result<u8, ParseError> {
        let buf: &mut [u8; 1] = &mut [0];
        let nread = self.reader.read(buf)?;

        if nread < 1 {
            return Err(ParseError::from(io::Error::new(
                io::ErrorKind::WouldBlock,
                "would block",
            )));
        } else {
            Ok(buf[0])
        }
    }

    fn read_string_line(&mut self) -> Result<String, ParseError> {
        match String::from_utf8(self.read_line()?) {
            Err(_) => Err(ParseError::Incomplete),
            Ok(value) => Ok(value),
        }
    }

    fn read_line(&mut self) -> Result<Vec<u8>, ParseError> {
        let mut rv = vec![];
        loop {
            let b = self.read_byte()?;
            match b as char {
                '\n' => {
                    break;
                }
                '\r' => {
                    self.expect_char('\n')?;
                    break;
                }
                _ => rv.push(b),
            };
        }
        Ok(rv)
    }

    fn read_int_line(&mut self) -> Result<i64, ParseError> {
        let line = self.read_string_line()?;
        match line.trim().parse::<i64>() {
            Err(_) => Err(ParseError::from("Expected int line integer, got garbage")),
            Ok(value) => Ok(value),
        }
    }

    fn read(&mut self, bytes: usize) -> Result<Vec<u8>, ParseError> {
        let mut rv = vec![0; bytes];
        let mut i = 0;
        while i < bytes {
            let res_nread = {
                let ref mut buf = &mut rv[i..];
                self.reader.read(buf)
            };
            match res_nread {
                Ok(nread) if nread > 0 => i += nread,
                Ok(_) => return Err(ParseError::from("Could not read enough bytes")),
                Err(e) => return Err(From::from(e)),
            }
        }
        Ok(rv)
    }

    fn parse_blob(&mut self) -> Result<Value, ParseError> {
        let bytes = self.read_int_line()? as usize;
        let buf = self.read(bytes)?;
        self.expect_newline()?;
        Ok(Value::Blob(buf))
    }

    fn parse_int(&mut self) -> Result<Value, ParseError> {
        Ok(Value::Number(self.read_int_line()?))
    }

    fn parse_simple_string(&mut self) -> Result<Value, ParseError> {
        Ok(Value::String(self.read_string_line()?.as_bytes().to_vec()))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        let length = self.read_int_line()? as usize;
        let mut rv = vec![];
        rv.reserve(length);
        for _ in 0..length {
            let v = self.parse_value()?;
            rv.push(v);
        }
        Ok(Value::Array(rv))
    }

    fn parse_map(&mut self) -> Result<Value, ParseError> {
        let length = self.read_int_line()?;
        let mut map = LinkedHashMap::<Value, Value>::new();
        for _ in 0..length {
            let key = self.parse_value()?;
            let value = self.parse_value()?;
            map.insert(key, value);
        }
        Ok(Value::Map(map))
    }

    fn parse_set(&mut self) -> Result<Value, ParseError> {
        let length = self.read_int_line()? as usize;
        let mut rv = vec![];
        rv.reserve(length);
        for _ in 0..length {
            let v = self.parse_value()?;
            rv.push(v);
        }
        Ok(Value::Set(rv))
    }

    fn parse_bigint(&mut self) -> Result<Value, ParseError> {
        let line = self.read_string_line()?;
        Ok(Value::Bigint(
            BigInt::parse_bytes(line.as_bytes(), 10).unwrap(),
        ))
    }

    fn parse_error(&mut self) -> Result<Value, ParseError> {
        Ok(Value::Error(self.read_string_line()?))
    }

    fn parse_null(&mut self) -> Result<Value, ParseError> {
        Ok(Value::Null)
    }

    fn parse_double(&mut self) -> Result<Value, ParseError> {
        match self.read_string_line()?.trim().parse::<f64>() {
            Ok(value) => Ok(Value::Double(Float64::from(value))),
            Err(_) => Err(ParseError::Incomplete),
        }
    }

    fn parse_boolean(&mut self) -> Result<Value, ParseError> {
        let line = self.read_string_line()?;
        if line.len() != 1 {
            return Err(ParseError::Incomplete);
        }
        Ok(if line.as_bytes()[0] as char == 't' {
            Value::Boolean(true)
        } else {
            Value::Boolean(false)
        })
    }
}

/// Parses bytes into a redis value.
///
/// This is the most straightforward way to parse something into a low
/// level redis value instead of having to use a whole parser.
///
pub fn parse_redis_value<R: Read>(stream: R) -> Result<Value, ParseError> {
    let mut parser = Parser::new(BufReader::new(stream));
    parser.parse_value()
}

/// Parses the length of the paramenter in the slice
/// Upon success, it returns a tuple with the length of the argument and the
/// length of the parsed length.
fn parse_int(input: &[u8], len: usize, name: &str) -> Result<(Option<usize>, usize), ParseError> {
    if input.len() == 0 {
        return Err(ParseError::Incomplete);
    }
    let mut i = 0;
    let mut argc = 0;
    let mut argco = None;
    while input[i] as char != '\r' {
        let c = input[i] as char;
        if argc == 0 && c == '-' {
            while input[i] as char != '\r' {
                i += 1;
            }
            argco = None;
            break;
        } else if c < '0' || c > '9' {
            return Err(ParseError::BadProtocol(format!("invalid {} length", name)));
        }
        argc *= 10;
        argc += input[i] as usize - '0' as usize;
        i += 1;
        if i == len {
            return Err(ParseError::Incomplete);
        }
        argco = Some(argc);
    }
    i += 1;
    if i == len {
        return Err(ParseError::Incomplete);
    }
    if input[i] as char != '\n' {
        return Err(ParseError::BadProtocol(format!(
            "expected \\r\\n separator, got \\r{}",
            input[i] as char
        )));
    }
    return Ok((argco, i + 1));
}

/// Creates a parser array from a buffer.
///
/// # Examples
///
/// ```
/// # use parser::parse_array;
/// let s = b"*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$2\r\n10\r\n";
/// let (parser, len) = parse_array(s).unwrap();
/// assert_eq!(len, 32);
/// ```
pub fn parse_array(input: &[u8]) -> Result<(Command, usize), ParseError> {
    let mut pos = 0;
    while input.len() > pos && input[pos] as char == '\r' {
        if pos + 1 < input.len() {
            if input[pos + 1] as char != '\n' {
                return Err(ParseError::BadProtocol(format!(
                    "expected \\r\\n separator, got \
                     \\r{}",
                    input[pos + 1] as char
                )));
            }
            pos += 2;
        } else {
            return Err(ParseError::Incomplete);
        }
    }
    if pos >= input.len() {
        return Err(ParseError::Incomplete);
    }
    if input[pos] as char != '*' {
        return Err(ParseError::BadProtocol(format!(
            "expected '*', got '{}'",
            input[pos] as char
        )));
    }
    pos += 1;
    let len = input.len();
    let (argco, intlen) = parse_int(&input[pos..len], len - pos, "multibulk")?;
    let argc = match argco {
        Some(i) => i,
        None => 0,
    };
    pos += intlen;
    if argc > 1024 * 1024 {
        return Err(ParseError::BadProtocol(
            "invalid multibulk length".to_owned(),
        ));
    }
    let mut argv = Vec::new();
    for i in 0..argc {
        if input.len() == pos {
            return Err(ParseError::Incomplete);
        }
        if input[pos] as char != '$' {
            return Err(ParseError::BadProtocol(format!(
                "expected '$', got '{}'",
                input[pos] as char
            )));
        }
        pos += 1;
        let (argleno, arglenlen) = parse_int(&input[pos..len], len - pos, "bulk")?;
        let arglen = match argleno {
            Some(i) => i,
            None => return Err(ParseError::BadProtocol("invalid bulk length".to_owned())),
        };
        if arglen > 512 * 1024 * 1024 {
            return Err(ParseError::BadProtocol("invalid bulk length".to_owned()));
        }
        pos += arglenlen;
        let arg = Argument {
            pos: pos,
            len: arglen,
        };
        argv.push(arg);
        pos += arglen + 2;
        if pos > len || (pos == len && i != argc - 1) {
            return Err(ParseError::Incomplete);
        }
    }
    Ok((Command::new(input, argv), pos))
}
