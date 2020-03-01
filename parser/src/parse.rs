use super::*;

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
