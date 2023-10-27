//! Fixed-length byte array

use bytes::Buf;

use crate::{error, ProtocolType};

impl<const SIZE: usize> ProtocolType for [u8; SIZE] {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() < SIZE {
            Err(error::Eof)?
        }

        let mut result = [0; SIZE];
        buffer.copy_to_slice(&mut result[..]);

        Ok(result)
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(&self[..]);

        Ok(())
    }
}
