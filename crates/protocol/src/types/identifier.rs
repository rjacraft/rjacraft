//! Minecraft's identifier type

use std::{fmt, str::FromStr};

use bytes::{Buf, BufMut};
use serde::de;

use crate::{error, ProtocolType};

const MAX_SIZE: usize = 1 << 15;
const DEFAULT_NS: &str = "minecraft";

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
    #[error("Too many colons")]
    TooManyColons,
    #[error("Invalid character '{0}' in namespace")]
    InvalidNamespace(char),
    #[error("Invalid character '{0}' in location")]
    InvalidLocation(char),
    #[error("Namespace is empty (remove the : if you want the default)")]
    EmptyNamespace,
    #[error("Location is empty")]
    EmptyLocation,
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
        let right = split.next();

        if split.next().is_some() {
            Err(IdentifierError::TooManyColons)?;
        }

        if let Some(right) = right {
            if left.is_empty() {
                Err(IdentifierError::EmptyNamespace)?;
            }

            if right.is_empty() {
                Err(IdentifierError::EmptyLocation)?;
            }

            for char in left.chars() {
                if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '-') {
                    Err(IdentifierError::InvalidNamespace(char))?;
                }
            }

            for char in right.chars() {
                if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '/' | '.' | '-') {
                    Err(IdentifierError::InvalidLocation(char))?;
                }
            }

            Ok(Identifier {
                namespace: left.into(),
                location: right.into(),
            })
        } else {
            if left.is_empty() {
                Err(IdentifierError::EmptyLocation)?;
            }

            for char in left.chars() {
                if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '/' | '.' | '-') {
                    Err(IdentifierError::InvalidLocation(char))?;
                }
            }

            Ok(Identifier {
                namespace: DEFAULT_NS.into(),
                location: left.into(),
            })
        }
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
            Err(IdentifierError::TooManyColons) => Err(de::Error::invalid_value(
                de::Unexpected::Other("an extra colon"),
                &"exactly one colon",
            )),
            Err(IdentifierError::InvalidNamespace(x)) => Err(de::Error::invalid_value(
                de::Unexpected::Char(x),
                &"a namespace that's lowercase alphanumeric with underscores and dashes",
            )),
            Err(IdentifierError::InvalidLocation(x)) => Err(de::Error::invalid_value(
                de::Unexpected::Char(x),
                &"a location that's lowercase alphanumeric with underscores, slashes, dots and dashes",
            )),
            Err(IdentifierError::EmptyNamespace) => Err(de::Error::missing_field("namespace")),
            Err(IdentifierError::EmptyLocation) => Err(de::Error::missing_field("location")),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing() {
        Identifier::from_str("minecraft:stone").unwrap();
        Identifier::from_str("stone").unwrap();
        Identifier::from_str("stone/special").unwrap();
        Identifier::from_str("").unwrap_err();
        Identifier::from_str("minecraft:").unwrap_err();
        Identifier::from_str(":stone").unwrap_err();
        Identifier::from_str("minecraft:stone:special").unwrap_err();
        Identifier::from_str("minecraft:русский_камень").unwrap_err();
        Identifier::from_str("minecraft:stone2").unwrap();
        Identifier::from_str("minecraft2:stone2").unwrap();
        Identifier::from_str("minecraft:stone/special").unwrap();
        Identifier::from_str("minecraft/special:stone/special").unwrap_err();
        Identifier::from_str("minecraft:stone.com").unwrap();
        Identifier::from_str("minecraft.net:stone.com").unwrap_err();
        Identifier::from_str("minecraft:stone-special").unwrap();
        Identifier::from_str("minecraft-2:stone-special").unwrap();
    }
}
