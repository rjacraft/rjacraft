//! An element that comes after a boolean indicating whether it'll be present.

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

#[derive(Debug, Clone)]
pub struct BoolOption<T>(pub Option<T>);

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError<E: std::error::Error> {
    #[error("Failed to read BoolOption marker")]
    Marker(#[from] error::Eof),
    #[error("Failed to read BoolOption item")]
    Item(#[source] E),
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum EncodeError<E: std::error::Error> {
    #[error("Failed to write LengthVec item")]
    Item(#[source] E),
}

impl<T: ProtocolType> ProtocolType for BoolOption<T> {
    type DecodeError = DecodeError<T::DecodeError>;
    type EncodeError = EncodeError<T::EncodeError>;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::Primitive(marker) = super::Primitive::<bool>::decode(buffer)?;

        Ok(Self(if marker {
            Some(T::decode(buffer).map_err(|e| DecodeError::Item(e))?)
        } else {
            None
        }))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        if let Some(item) = &self.0 {
            item.encode(buffer).map_err(|e| EncodeError::Item(e))?;
        }

        Ok(())
    }
}

impl<T> From<Option<T>> for BoolOption<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> From<BoolOption<T>> for Option<T> {
    fn from(value: BoolOption<T>) -> Self {
        value.0
    }
}
