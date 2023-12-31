use std::{fmt, str};

use serde::{Deserialize, Serialize};

pub const NAME_LENGTH: usize = 16;

/// The contents are guaranteed to be no longer than [`PROFILE_NAME_LENGTH`].
#[derive(Debug, Clone, Serialize)]
pub struct Name(String);

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<String> for Name {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("name is longer than expected ({0} chars)")]
pub struct NameTooLong(pub usize);

impl TryFrom<String> for Name {
    type Error = NameTooLong;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= NAME_LENGTH {
            Ok(Self(value))
        } else {
            Err(NameTooLong(value.len()))
        }
    }
}

impl str::FromStr for Name {
    type Err = NameTooLong;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() <= NAME_LENGTH {
            Ok(Self(value.to_string()))
        } else {
            Err(NameTooLong(value.len()))
        }
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        if string.len() <= NAME_LENGTH {
            Ok(Self(string))
        } else {
            Err(serde::de::Error::custom("name is longer than expected"))
        }
    }
}
