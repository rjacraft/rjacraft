//! A length-prefixed string with a length limit (prefixed by a [`super::VarInt`])

use std::string;

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

/// Guaranteed to be no larger than `MAX_SIZE`.
#[derive(Debug, Clone)]
pub struct LenString<const MAX_SIZE: usize>(pub(super) String);

#[derive(Debug, thiserror::Error, nevermore::FromNever)]
pub enum DecodeError<const MAX_SIZE: usize> {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("Failed to read string length")]
    Length(#[from] super::varint::DecodeError),
    #[error(transparent)]
    Overrun(#[from] error::Overrun<MAX_SIZE>),
    #[error("UTF-8 error")]
    Utf8(#[from] string::FromUtf8Error),
}

impl<const MAX_SIZE: usize> ProtocolType for LenString<MAX_SIZE> {
    type DecodeError = DecodeError<MAX_SIZE>;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt(len) = super::VarInt::decode(buffer)?;

        if len as usize > MAX_SIZE {
            Err(error::Overrun(len as usize))?;
        }

        if buffer.remaining() < len as usize {
            Err(error::Eof)?
        }

        Ok(Self(String::from_utf8(
            buffer.copy_to_bytes(len as usize).to_vec(),
        )?))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let bytes = self.0.as_bytes();

        super::VarInt(bytes.len() as i32).encode(buffer)?;
        buffer.put(bytes);

        Ok(())
    }
}

impl<const MAX_SIZE: usize> TryFrom<String> for LenString<MAX_SIZE> {
    type Error = error::Overrun<MAX_SIZE>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > MAX_SIZE {
            Err(error::Overrun(value.len()))
        } else {
            Ok(LenString(value))
        }
    }
}

impl<const MAX_SIZE: usize> From<LenString<MAX_SIZE>> for String {
    fn from(value: LenString<MAX_SIZE>) -> Self {
        value.0
    }
}

impl<const MAX_SIZE: usize> AsRef<str> for LenString<MAX_SIZE> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
