use std::{
    fmt::Display,
    io::{self, Read},
};

use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess},
    Deserialize,
};

use crate::{de::payload::PayloadDeserializer, Tag, TagFromU8Error};

pub struct MapDeserializer<'r, R: ?Sized> {
    reader: &'r mut R,
    tag: Tag,
}

/// An error occurring while deserializing using [`MapDeserializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("invalid tag")]
    InvalidTag(#[from] TagFromU8Error),
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

impl<'r, R: ?Sized> MapDeserializer<'r, R> {
    #[inline]
    pub fn new(reader: &'r mut R) -> Self {
        Self {
            reader,
            tag: Tag::End,
        }
    }
}

impl<'de: 'r, 'r, R: ?Sized + Read> MapAccess<'de> for MapDeserializer<'r, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let mut buffer = [0u8];
        self.reader.read_exact(&mut buffer)?;
        self.tag = Tag::try_from(buffer[0])?;

        if self.tag == Tag::End {
            return Ok(None);
        }

        seed.deserialize(PayloadDeserializer::new(self.reader, Tag::String))
            .map(Some)
            .map_err(|e| match e {
                [f, ..] => e.field(*f),
                [] => e,
            })
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        todo!()
    }
}
