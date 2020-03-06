use super::*;
use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct RedisCodec;

impl RedisCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Encoder for RedisCodec {
    type Item = Value;
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
        match parse_redis_value(&src[..]) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e),
        }
    }
}
