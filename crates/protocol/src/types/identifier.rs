use std::{fmt, str::FromStr};

use bytes::{Buf, BufMut};

use crate::ProtocolType;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub namespace: String,
    pub location: String,
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.location)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdentifierError {
    #[error("Not enough colons")]
    NotEnoughColons,
    #[error("Too many colons")]
    TooManyColons,
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(":");

        let left = split.next().unwrap();
        let right = split.next().ok_or(IdentifierError::NotEnoughColons)?;

        if split.next().is_some() {
            Err(IdentifierError::TooManyColons)?
        }

        Ok(Identifier {
            namespace: left.into(),
            location: right.into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    String(#[from] super::string::DecodeError),
    #[error(transparent)]
    Item(#[from] IdentifierError),
}

impl ProtocolType for Identifier {
    type DecodeError = DecodeError;
    type EncodeError = super::string::EncodeError;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        Ok(Self::from_str(&String::decode(buffer)?)?)
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        self.to_string().encode(buffer)?;

        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_str(<&str>::deserialize(deserializer)?).map_err(serde::de::Error::custom)?)
    }
}

impl serde::Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}
