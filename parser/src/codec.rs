use super::*;

use bytes::{BufMut, BytesMut};
// use std::mem;
use tokio_util::codec::{Decoder, Encoder};

pub struct RedisCodec {
    max_buffer_length: usize,
    buffer: Vec<u8>,
}

impl RedisCodec {
    pub fn new(max_buffer_length: usize) -> Self {
        Self {
            max_buffer_length: max_buffer_length,
            buffer: Vec::new(),
        }
    }
}

impl Encoder for RedisCodec {
    type Item = Value;
    type Error = ParseError;
    fn encode(&mut self, _event: Value, _buf: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Decoder for RedisCodec {
    type Item = Value;
    type Error = ParseError;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let byte = src[0];
        let value = match byte as char {
            resp_event_type::BLOB_STRING => {
                self.buffer.push(byte);
                Some(Value::String(self.buffer.to_vec()))
            }
            _ => None,
        };

        Ok(value)
    }
}
