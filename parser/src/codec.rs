use super::*;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct RedisCodec;

impl RedisCodec {
    pub fn new() -> Self {
        Self {}
    }
}

/// upgrade tokio util 0.3
impl Encoder<Value> for RedisCodec {
    type Error = ParseError;
    fn encode(&mut self, event: Value, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = event.as_bytes();
        buf.reserve(bytes.len());
        buf.put(&bytes[..]);
        Ok(())
    }
}

impl Decoder for RedisCodec {
    type Item = Value;
    type Error = ParseError;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        if len == 0 {
            // there are no bytes to consume, stop querying the buffer
            return Ok(None);
        }
        match parse_redis_value(&src[..]) {
            Ok(v) => {
                src.advance(v.as_bytes().len());
                Ok(Some(v))
            }
            Err(e) => Err(e),
        }
    }
    // fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    //     Ok(match self.decode(buf)? {
    //         Some(frame) => Some(frame),
    //         None => None,
    //     })
    // }
}
