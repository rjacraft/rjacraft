//! A string of typed, [`serde`]-powered JSON

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

/// `MAX_SIZE` will not be checked before trying to encode the packet.
#[derive(Debug, Clone)]
pub struct JsonString<const MAX_SIZE: usize, T>(pub T);

#[derive(Debug, thiserror::Error)]
pub enum DecodeError<const MAX_SIZE: usize> {
    #[error(transparent)]
    String(#[from] super::len_string::DecodeError<MAX_SIZE>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum EncodeError<const MAX_SIZE: usize> {
    #[error(transparent)]
    Overrun(#[from] error::Overrun<MAX_SIZE>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl<const MAX_SIZE: usize, T: serde::Serialize + serde::de::DeserializeOwned> ProtocolType
    for JsonString<MAX_SIZE, T>
{
    type DecodeError = DecodeError<MAX_SIZE>;
    type EncodeError = EncodeError<MAX_SIZE>;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let string = super::LenString::<MAX_SIZE>::decode(buffer)?;

        Ok(JsonString(serde_json::from_str(string.as_ref())?))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let serialized = serde_json::to_string(&self.0)?;

        super::LenString::<MAX_SIZE>::try_from(serialized)?.encode(buffer)?;

        Ok(())
    }
}

impl<const MAX_SIZE: usize, T> From<T> for JsonString<MAX_SIZE, T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// impl<T> From<JsonString<T>> for T {
//     fn from(value: JsonString<T>) -> Self {
//         value.0
//     }
// }
