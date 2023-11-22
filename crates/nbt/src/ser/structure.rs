use std::{
    fmt::Display,
    io::{self, Write},
};

use serde::{ser::SerializeStruct, Serialize};

use crate::{
    ser::payload::{self, PayloadSerializer},
    string::{NbtStr, NbtStrFromStrError},
    CompoundTag,
    Tag,
};

#[derive(Debug)]
pub struct StructSerializer<'w, W: ?Sized> {
    writer: &'w mut W,
}

/// An error occurring while serializing using [`StructSerializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("the key is invalid")]
    InvalidKey(#[from] NbtStrFromStrError),
    #[error("the value is of invalid type")]
    InvalidValue(#[from] Box<payload::Error>),
    #[error("{0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl<'w, W: ?Sized> StructSerializer<'w, W> {
    #[inline]
    pub fn new(writer: &'w mut W) -> Self {
        Self { writer }
    }
}

impl<W: ?Sized + Write> SerializeStruct for StructSerializer<'_, W> {
    type Ok = CompoundTag;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value
            .serialize(&mut PayloadSerializer::named(
                self.writer,
                NbtStr::try_from(key)?,
            ))
            .map_err(Box::new)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&Tag::End.to_be_bytes())?;
        Ok(CompoundTag::Compound)
    }
}
