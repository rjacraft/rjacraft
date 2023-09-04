//! Minecraft's identifier type

use std::{fmt, str::FromStr};

use bytes::{Buf, BufMut};
use serde::de;

use crate::{error, ProtocolType};

const MAX_SIZE: usize = 1 << 15;

/// The way to construct this is by calling [`FromStr`].
#[derive(Debug, Clone)]
pub struct Identifier {
    namespace: String,
    location: String,
}

impl Identifier {
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn location(&self) -> &str {
        &self.namespace
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.location)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdentifierError {
    #[error("Not enough colons")]
    MissingColon,
    #[error("Too many colons")]
    TooManyColons,
    #[error(transparent)]
    Overrun(#[from] error::Overrun<MAX_SIZE>),
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_SIZE {
            Err(error::Overrun(s.len()))?;
        }

        let mut split = s.split(":");

        let left = split.next().unwrap();
        let right = split.next().ok_or(IdentifierError::MissingColon)?;

        if split.next().is_some() {
            Err(IdentifierError::TooManyColons)?;
        }

        // TODO alphanumeric

        Ok(Identifier {
            namespace: left.into(),
            location: right.into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error(transparent)]
    String(#[from] super::len_string::DecodeError<MAX_SIZE>),
    #[error(transparent)]
    This(#[from] IdentifierError),
}

impl ProtocolType for Identifier {
    type DecodeError = DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        Ok(Self::from_str(
            &super::LenString::<MAX_SIZE>::decode(buffer)?.0,
        )?)
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        super::LenString::<MAX_SIZE>(self.to_string()).encode(buffer)?;

        Ok(())
    }
}

impl<'de> serde::Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Self::from_str(<&str>::deserialize(deserializer)?) {
            Ok(x) => Ok(x),
            Err(IdentifierError::MissingColon) => Err(de::Error::invalid_value(
                de::Unexpected::Other("missing colon"),
                &"exactly one colon",
            )),
            Err(IdentifierError::TooManyColons) => Err(de::Error::invalid_value(
                de::Unexpected::Other("too many colons"),
                &"exactly one colon",
            )),
            Err(IdentifierError::Overrun(e)) => Err(e.as_serde()),
        }
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
