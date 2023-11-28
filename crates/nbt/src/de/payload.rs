use std::{
    fmt::Display,
    io::{self, Read},
};

use serde::{de::Visitor, forward_to_deserialize_any, Deserializer};

use crate::{de::map::MapDeserializer, Tag};

pub struct PayloadDeserializer<'r, R: ?Sized> {
    reader: &'r mut R,
    tag: Tag,
}

/// An error occurring while deserializing using [`PayloadDeserializer`].
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

impl<'r, R: ?Sized> PayloadDeserializer<'r, R> {
    #[inline]
    pub fn new(reader: &'r mut R, tag: Tag) -> Self {
        Self { reader, tag }
    }
}

impl<'de: 'r, 'r, R: ?Sized + Read> Deserializer<'de> for PayloadDeserializer<'r, R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.tag == Tag::Compound {
            visitor.visit_map(MapDeserializer::new(self.reader, fields))
        } else {
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
