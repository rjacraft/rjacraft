use std::{
    fmt::Display,
    io::{self, Write},
};

use serde::{ser::SerializeSeq, Serialize};

use crate::{
    ser::{payload, PayloadSerializer},
    ArrayTag,
    ListTag,
    NotEndTag,
    Tag,
};

/// An error occurring while serializing using [`SeqSerializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("invalid element payload")]
    InvalidPayload(#[from] payload::Error),
    #[error("{0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

#[derive(Debug)]
pub struct ArraySeqSerializer<'w, W: ?Sized> {
    writer: &'w mut W,
    tag: ArrayTag,
}

impl<'w, W: ?Sized> ArraySeqSerializer<'w, W> {
    pub fn new(writer: &'w mut W, tag: ArrayTag) -> Self {
        Self { writer, tag }
    }
}

impl<'w, W: ?Sized + Write> SerializeSeq for ArraySeqSerializer<'_, W> {
    type Ok = ArrayTag;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut PayloadSerializer::seq_element(
            self.writer,
            self.tag.element_tag().into(),
        ))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.tag)
    }
}

#[derive(Debug)]
pub struct ListSeqSerializer<'w, W: ?Sized> {
    writer: &'w mut W,
    tag: Tag,
    len: i32,
}

impl<'w, W: ?Sized> ListSeqSerializer<'w, W> {
    pub fn new(writer: &'w mut W, len: i32) -> Self {
        assert!(len >= 0, "len cannot be negative but is {}", len);

        Self {
            writer,
            tag: Tag::End,
            len,
        }
    }
}

impl<'w, W: ?Sized + Write> SerializeSeq for ListSeqSerializer<'_, W> {
    type Ok = ListTag;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        if let Ok(tag) = NotEndTag::try_from(self.tag) {
            value.serialize(&mut PayloadSerializer::seq_element(self.writer, tag))?;
        } else {
            let mut serializer = PayloadSerializer::list_head(self.writer, self.len);
            self.tag = value.serialize(&mut serializer)?.into();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.tag == Tag::End {
            // The sequence has no elements thus we do
            // what the reference implementation by Mojang does
            // and set the type to `TAG_End`
            self.writer.write_all(&Tag::End.to_be_bytes())?;
            self.writer.write_all(&0u32.to_be_bytes())?;
        }
        // FIXME: strict ype for `List`
        Ok(ListTag::List)
    }
}
