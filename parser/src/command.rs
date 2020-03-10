use super::error::ParseError;
use std::str::from_utf8;
use util::format_repr;

#[derive(Debug, Clone)]
pub struct Argument {
    pub pos: usize,
    pub len: usize,
}

pub struct Command {
    pos: usize,
    data: Vec<u8>,
    pub argv: Vec<Argument>,
}

impl Command {
    pub fn cmd() -> Self {
        let cmd = Self {
            pos: 0,
            data: vec![],
            argv: vec![],
        };
        cmd
    }

    /// From bytes to cmmand
    ///
    pub fn new(input: &[u8], argv: Vec<Argument>) -> Self {
        Command {
            pos: 0,
            argv,
            data: input.to_vec(),
        }
    }

    /// Add argument position
    #[inline]
    fn add_argument_position(&mut self, len: usize) -> &mut Self {
        self.argv.push(Argument { pos: self.pos, len });
        self.pos += len;
        self
    }

    #[inline]
    fn pos_add_offset(&mut self) -> &mut Self {
        self.pos += 1;
        self
    }

    #[inline]
    fn extend_len_bytes(&mut self, buf: Vec<u8>) -> &mut Self {
        self.data.extend_from_slice(&buf);
        self.pos += buf.len();
        self.write_line();
        self
    }

    #[inline]
    fn extend_from_bytes(&mut self, buf: Vec<u8>) -> &mut Self {
        self.data.extend_from_slice(&buf);
        self.add_argument_position(buf.len());
        self.write_line();
        self
    }

    #[inline]
    fn put_byte(&mut self, byte: u8) -> &mut Self {
        self.data.push(byte);
        self.pos_add_offset();
        self
    }

    /// Write an command
    pub fn write_arrs(&mut self, n: usize) -> &mut Self {
        self.put_byte('*' as u8)
            .extend_len_bytes(n.to_string().into_bytes())
    }

    /// Write a simple string into command
    ///
    pub fn write_simple(&mut self, val: &str) -> &mut Self {
        self.put_byte('+' as u8)
            .extend_from_bytes(val.to_string().into_bytes())
    }

    pub fn write_blob(&mut self, val: &str) -> &mut Self {
        self.put_byte('$' as u8)
            .extend_len_bytes(val.len().to_string().into_bytes())
            .extend_from_bytes(val.to_string().into_bytes())
    }

    fn write_line(&mut self) -> &mut Self {
        self.data.append(&mut b"\r\n".to_vec());
        self.pos += 2;
        self
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

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for a in self.argv.iter() {
            format_repr(f, &self.data[a.pos..(a.pos + a.len)]);
            f.write_str(" ");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_command() {
        let mut cmd = Command::cmd();
        let cmd = cmd
            .write_arrs(3)
            .write_blob(&"set")
            .write_blob(&"a")
            .write_blob(&"123");

        // println!("{:?}", cmd);
        assert_eq!(cmd.get_str(0).unwrap(), "set");
        assert_eq!(cmd.get_str(1).unwrap(), "a");
        assert_eq!(cmd.get_str(2).unwrap(), "123");
    }

    #[test]
    fn test_write_simple() {
        let mut cmd = Command::cmd();
        cmd.write_simple(&"123");
        assert_eq!(cmd.get_data(), &b"+123\r\n"[..]);
        assert_eq!(cmd.get_str(0).unwrap(), "123");
    }

    #[test]
    fn test_write_blob() {
        let mut cmd = Command::cmd();
        cmd.write_blob("123");
        assert_eq!(cmd.get_data(), &b"$3\r\n123\r\n"[..]);
        assert_eq!(cmd.get_str(0).unwrap(), "123");
    }
}
