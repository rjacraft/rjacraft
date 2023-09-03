//! A length-prefixed array (prefixed by a [`super::VarInt`]).

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

/// There are 2 implementations of this:
///
///   - One for anything that's [`ProtocolType`] and can of be any length
///   - One just for bytes (faster than decoding [`super::Primitive<u8>`])
#[derive(Debug, Clone)]
pub struct LenVec<T>(pub Vec<T>);

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError<E: std::error::Error> {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("Failed to read LengthVec length")]
    Length(#[from] super::varint::DecodeError),
    #[error("Failed to read LengthVec item")]
    Item(#[source] E),
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum EncodeError<E: std::error::Error> {
    #[error("Failed to write LengthVec item")]
    Item(#[source] E),
}

// # Generic implementation for complex items

impl<T: ProtocolType> ProtocolType for LenVec<T> {
    type DecodeError = DecodeError<T::DecodeError>;
    type EncodeError = EncodeError<T::EncodeError>;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt(len) = super::VarInt::decode(buffer)?;
        let mut result = Vec::with_capacity(len as usize);

        for _ in 0..len {
            result.push(T::decode(buffer).map_err(|e| DecodeError::Item(e))?);
        }

        Ok(Self(result))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        for item in &self.0 {
            item.encode(buffer).map_err(|e| EncodeError::Item(e))?;
        }

        Ok(())
    }
}

impl<P: ProtocolType, F> From<Vec<F>> for LenVec<P>
where
    P: From<F>,
{
    fn from(value: Vec<F>) -> Self {
        Self(value.into_iter().map(|x| x.into()).collect())
    }
}

impl<P: ProtocolType, F> From<LenVec<P>> for Vec<F>
where
    F: From<P>,
{
    fn from(value: LenVec<P>) -> Self {
        value.0.into_iter().map(|x| x.into()).collect()
    }
}

// # Special impl for byte vectors (faster than Primitive<u8>)

impl ProtocolType for LenVec<u8> {
    type DecodeError = DecodeError<error::Eof>;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt(len) = super::VarInt::decode(buffer)?;

        if buffer.remaining() >= len as usize {
            Ok(Self(buffer.copy_to_bytes(len as usize).to_vec()))
        } else {
            Err(DecodeError::Item(error::Eof))
        }
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(self.0.as_slice());

        Ok(())
    }
}

impl From<Vec<u8>> for LenVec<u8> {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<LenVec<u8>> for Vec<u8> {
    fn from(value: LenVec<u8>) -> Self {
        value.0
    }
}
