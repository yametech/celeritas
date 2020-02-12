use std::error::Error;
use std::str::{from_utf8, Utf8Error};

/// Error parsing
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
            ParseError::Incomplete => true,
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

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Incomplete => "Incomplete data",
            ParseError::BadProtocol(_) => "Protocol error",
            ParseError::InvalidArgument => "Invalid argument",
            ParseError::Unknown(_) => "Unknown error",
        }
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.response_string().fmt(f)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(err: Utf8Error) -> Self {
        ParseError::InvalidArgument
    }
}

impl From<&'static str> for ParseError {
    fn from(err: &'static str) -> Self {
        ParseError::Unknown(err)
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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // // fn test_parse_int() {}

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
