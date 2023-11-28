use std::{
    fmt::Display,
    io::{self},
};

use serde::{ser::SerializeStruct, Serialize};

use crate::{
    ser::payload::{self, PayloadSerializer},
    string::{NbtStr, NbtStrFromStrError},
    write::NbtWrite,
    CompoundTag,
};

#[derive(Debug)]
pub struct StructSerializer<W> {
    writer: W,
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

impl<W> StructSerializer<W> {
    #[inline]
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: NbtWrite> SerializeStruct for StructSerializer<W> {
    type Ok = CompoundTag;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value
            .serialize(PayloadSerializer::named(
                self.writer.fork(),
                NbtStr::try_from(key)?,
            ))
            .map_err(Box::new)?;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        self.writer.end_compound()?;
        Ok(CompoundTag::Compound)
    }
}
