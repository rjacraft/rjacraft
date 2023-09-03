//! A string of typed, [`serde`]-powered JSON.

use bytes::{Buf, BufMut};

use crate::ProtocolType;

#[derive(Debug, Clone)]
pub struct JsonString<T>(pub T);

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    String(#[from] super::string::DecodeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error(transparent)]
    String(#[from] super::string::EncodeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> ProtocolType for JsonString<T> {
    type DecodeError = DecodeError;
    type EncodeError = EncodeError;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let string = String::decode(buffer)?;

        Ok(JsonString(serde_json::from_str(&string)?))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        serde_json::to_string(&self.0)?.encode(buffer)?;

        Ok(())
    }
}

impl<T> From<T> for JsonString<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// impl<T> From<JsonString<T>> for T {
//     fn from(value: JsonString<T>) -> Self {
//         value.0
//     }
// }
