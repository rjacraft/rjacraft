use std::{
    fmt::Display,
    io::{self, Read},
};

use serde::de::{DeserializeSeed, SeqAccess};

use crate::{de::payload::PayloadDeserializer, Tag};

pub struct SeqDeserializer<'r, R: ?Sized> {
    reader: &'r mut R,
    tag: Tag,
    remaining: u32,
}

/// An error occurring while deserializing using [`SeqDeserializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

type Result<T> = std::result::Result<T, Error>;

impl<'r, R: ?Sized> SeqDeserializer<'r, R> {
    #[inline]
    pub fn new(reader: &'r mut R, tag: Tag, remaining: u32) -> Self {
        Self {
            reader,
            tag,
            remaining,
        }
    }
}

impl<'de: 'r, 'r, R: ?Sized + Read> SeqAccess<'de> for SeqDeserializer<'r, R> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.remaining > 0 {
            self.remaining -= 1;
            Ok(Some(seed.deserialize(PayloadDeserializer::new(
                self.reader,
                self.tag,
            ))?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining as usize)
    }
}
