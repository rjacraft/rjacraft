//! A binary UUID composed of 16 bytes

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

impl ProtocolType for uuid::Uuid {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() >= 16 {
            let mut bytes = [0; 16];

            buffer.copy_to_slice(&mut bytes);

            Ok(uuid::Uuid::from_bytes(bytes))
        } else {
            Err(error::Eof)
        }
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(&self.as_bytes()[..]);

        Ok(())
    }
}
