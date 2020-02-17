extern crate util;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::iter;
use std::str::{from_utf8, Utf8Error};
use util::format_repr;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum OperatType {
    String,
    Hash,
    List,
    Set,
    SortedSet,
    HyperLogLog,
    PubSub,
}

lazy_static! {
    pub static ref OPERAT_HASHMAP: HashMap<OperatType, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert(OperatType::String, vec!["get", "set", "del"]);
        m
    };
    pub static ref COUNT: usize = OPERAT_HASHMAP.len();
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // incomplete command line
    Incomplete,
    // protocol bug
    BadProtocol(String),
    // argument invalid
    InvalidArgument,
    // other
    Unknown(&'static str),
}

impl ParseError {
    pub fn in_complete(&self) -> bool {
        match *self {
            ParseError::Incomplete => true,
            _ => false,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match *self {
            ParseError::Unknown(_) => true,
            _ => false,
        }
    }

    pub fn response_string(&self) -> String {
        match *self {
            ParseError::Incomplete => "Incomplete data".to_owned(),
            ParseError::BadProtocol(ref s) => format!("Protocol error: {}", s),
            ParseError::InvalidArgument => "Invalid argument".to_owned(),
            ParseError::Unknown(ref s) => format!("Unknown error: {}", s),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Incomplete => "Incomplete data",
            ParseError::BadProtocol(_) => "Protocol error",
            ParseError::InvalidArgument => "Invalid argument",
            ParseError::Unknown(_) => "Unknown error",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.response_string().fmt(f)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        ParseError::InvalidArgument
    }
}

impl From<&'static str> for ParseError {
    fn from(err: &'static str) -> Self {
        ParseError::Unknown(err)
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub pos: usize,
    pub len: usize,
}

pub struct Command<'a> {
    data: &'a [u8],
    pub argv: Vec<Argument>,
}

impl<'a> Command<'a> {
    pub fn new(input: &'a [u8], argv: Vec<Argument>) -> Self {
        Command {
            argv: argv,
            data: input,
        }
    }

    /// Gets an str from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parser::{Command, Argument};
    /// let parser = Command::new(b"foo", vec![Argument { pos: 0, len: 3 }]);
    /// assert_eq!(parser.get_str(0).unwrap(), "foo");
    /// ```
    pub fn get_str(&self, pos: usize) -> Result<&str, ParseError> {
        let data = self.get_slice(pos)?;
        Ok(from_utf8(data)?)
    }

    /// Gets a &[u8] from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parser::{Command, Argument};
    /// let parser = Command::new(b"foo", vec![Argument { pos: 0, len: 3 }]);
    /// assert_eq!(parser.get_slice(0).unwrap(), b"foo");
    /// ```
    pub fn get_slice(&self, pos: usize) -> Result<&[u8], ParseError> {
        if pos > self.argv.len() {
            return Err(ParseError::InvalidArgument);
        }
        let arg = &self.argv[pos];
        Ok(&self.data[arg.pos..arg.pos + arg.len])
    }

    /// Gets a Vec<u8> from a parameter
    ///
    /// # Examples
    ///
    /// ```
    /// # use parser::{Command, Argument};
    /// let parser = Command::new(b"foo", vec![Argument { pos: 0, len: 3 }]);
    /// assert_eq!(parser.get_vec(0).unwrap(), b"foo".to_vec());
    /// ```
    pub fn get_vec(&self, pos: usize) -> Result<Vec<u8>, ParseError> {
        let data = self.get_slice(pos)?;
        Ok(data.to_vec())
    }

    pub fn get_data(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> std::fmt::Debug for Command<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for a in self.argv.iter() {
            format_repr(f, &self.data[a.pos..(a.pos + a.len)]);
            f.write_str(" ");
        }
        Ok(())
    }
}

pub struct Parser {
    data: Vec<u8>,
    pub position: usize,
    pub written: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            data: vec![],
            position: 0,
            written: 0,
        }
    }

    pub fn allocate(&mut self) {
        if self.position > 0 && self.written == self.position {
            self.written = 0;
            self.position = 0;
        }

        let len = self.data.len();
        let add = if len == 0 {
            16
        } else if self.written * 2 > len {
            len
        } else {
            0
        };

        if add > 0 {
            self.data.extend(iter::repeat(0).take(add));
        }
    }

    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn is_incompletes(&self) -> bool {
        let data = &(&*self.data)[self.position..self.written];
        match parse(data) {
            Ok(_) => false,
            Err(e) => e.in_complete(),
        }
    }

    pub fn next(&mut self) -> Result<Command, ParseError> {
        let data = &(&*self.data)[self.position..self.written];
        let (r, len) = parse(data)?;
        self.position += len;
        Ok(r)
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("Parser: ")?;
        format_repr(f, &(&*self.data)[self.position..self.written])
    }
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

/// Creates a parser from a buffer.
///
/// # Examples
///
/// ```
/// # use parser::parse;
/// let s = b"*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$2\r\n10\r\n";
/// let (parser, len) = parse(s).unwrap();
/// assert_eq!(len, 32);
/// ```
pub fn parse(input: &[u8]) -> Result<(Command, usize), ParseError> {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let s = b"*3\r\n$3\r\nset\r\n$2\r\nxy\r\n$2\r\nab\r\n";
        let (parse, len) = parse(s).unwrap();
        assert_eq!(len, 29);
        assert_eq!(parse.get_str(0).unwrap(), "set");
        assert_eq!(parse.get_str(1).unwrap(), "xy");
        assert_eq!(parse.get_str(2).unwrap(), "ab");
    }
    #[test]
    fn test_operat_hashmap() {
        let r = match OPERAT_HASHMAP.get(&OperatType::String) {
            Some(_) => 0,
            None => 1,
        };
        assert_eq!(r, 0);
    }
}
