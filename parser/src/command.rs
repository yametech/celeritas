use super::error::ParseError;
use std::str::from_utf8;
use util::format_repr;

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
