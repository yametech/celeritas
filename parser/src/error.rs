use std::io::Error;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum ParseError {
    IO(Error),
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
            ParseError::IO(ref err) => format!("IO error: {}", err),
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
            ParseError::IO(_) => "IO error",
            ParseError::Incomplete => "Incomplete data",
            ParseError::BadProtocol(_) => "Protocol error",
            ParseError::InvalidArgument => "Invalid argument",
            ParseError::Unknown(_) => "Unknown error",
        }
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
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

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::IO(err)
    }
}

impl PartialEq for ParseError {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (ParseError::IO(a), ParseError::IO(b)) => a.kind() == b.kind(),
            (ParseError::Incomplete, ParseError::Incomplete) => true,
            (ParseError::BadProtocol(_), ParseError::BadProtocol(_)) => true,
            (ParseError::InvalidArgument, ParseError::InvalidArgument) => true,
            (ParseError::Unknown(_), ParseError::Unknown(_)) => true,
            _ => false,
        }
    }
}
