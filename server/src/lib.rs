extern crate dbc;
use dbc::DatabaseConnections;

use net2::{TcpBuilder, TcpStreamExt};
use std::fs::File;

#[cfg(unix)]
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use unix_socket::{UnixListener, UnixStream};
/// A stream connection.
#[cfg(unix)]
enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

#[cfg(not(unix))]
enum Stream {
    Tcp(TcpStream),
}

#[cfg(unix)]
impl Stream {
    /// Creates a new independently owned handle to the underlying socket.
    fn try_clone(&self) -> io::Result<Stream> {
        match *self {
            Stream::Tcp(ref s) => Ok(Stream::Tcp(s.try_clone()?)),
            Stream::Unix(ref s) => Ok(Stream::Unix(s.try_clone()?)),
        }
    }
    /// Write a buffer into this object, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Stream::Tcp(ref mut s) => s.write(buf),
            Stream::Unix(ref mut s) => s.write(buf),
        }
    }
    /// Sets the keepalive timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_keepalive(&self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => TcpStreamExt::set_keepalive(s, duration),
            Stream::Unix(_) => Ok(()),
        }
    }
    /// Sets the write timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => s.set_write_timeout(dur),
            // TODO: couldn't figure out how to enable this in unix_socket
            Stream::Unix(_) => Ok(()),
        }
    }
    /// Sets the read timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => s.set_read_timeout(dur),
            // TODO: couldn't figure out how to enable this in unix_socket
            Stream::Unix(_) => Ok(()),
        }
    }
}

#[cfg(not(unix))]
impl Stream {
    /// Creates a new independently owned handle to the underlying socket.
    fn try_clone(&self) -> io::Result<Stream> {
        match *self {
            Stream::Tcp(ref s) => Ok(Stream::Tcp(s.try_clone())?),
        }
    }

    /// Write a buffer into this object, returning how many bytes were written.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Stream::Tcp(ref mut s) => s.write(buf),
        }
    }

    /// Sets the keepalive timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_keepalive(&self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => TcpStreamExt::set_keepalive(s, duration),
        }
    }

    /// Sets the write timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => s.set_write_timeout(dur),
        }
    }

    /// Sets the read timeout to the timeout specified.
    /// It fails silently for UNIX sockets.
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Stream::Tcp(ref s) => s.set_read_timeout(dur),
        }
    }
}

#[cfg(not(unix))]
impl Read for Stream {
    /// Pull some bytes from this source into the specified buffer,
    /// returning how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Stream::Tcp(ref mut s) => s.read(buf),
        }
    }
}

/// A client connection
struct Client {
    /// The socket connection
    stream: Stream,
    /// A reference to the database connections
    db: Arc<Mutex<DatabaseConnections>>,
    /// The client unique identifier
    id: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
