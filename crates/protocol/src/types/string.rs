//! A length-prefixed string (prefixed by a [`super::VarInt`])

use std::string;

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

const MAX_SIZE: usize = 32767;

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("Failed to read string length")]
    Length(#[from] super::varint::DecodeError),
    #[error("The string is larger than {MAX_SIZE}")]
    TooLarge,
    #[error("UTF-8 error")]
    Utf8(#[from] string::FromUtf8Error),
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum EncodeError {
    #[error("The string is larger than {MAX_SIZE}")]
    TooLarge,
}

impl ProtocolType for String {
    type DecodeError = DecodeError;
    type EncodeError = EncodeError;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt(len) = super::VarInt::decode(buffer)?;

        if len as usize > MAX_SIZE {
            Err(DecodeError::TooLarge)?;
        }

        if buffer.remaining() < len as usize {
            Err(error::Eof)?
        }

        Ok(String::from_utf8(
            buffer.copy_to_bytes(len as usize).to_vec(),
        )?)
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let bytes = self.as_bytes();

        if bytes.len() > MAX_SIZE {
            Err(EncodeError::TooLarge)?;
        }

        super::VarInt(bytes.len() as i32).encode(buffer)?;
        buffer.put(bytes);

        Ok(())
    }
}
