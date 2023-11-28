use std::{
    fmt::Display,
    io::{self},
};

use serde::{ser::SerializeSeq, Serialize};

use crate::{
    ser::{payload, PayloadSerializer},
    write::NbtWrite,
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
pub struct ArraySeqSerializer<W> {
    writer: W,
    tag: ArrayTag,
}

impl<W: NbtWrite> ArraySeqSerializer<W> {
    pub fn new(writer: W, tag: ArrayTag) -> Self {
        Self { writer, tag }
    }
}

impl<W: NbtWrite> SerializeSeq for ArraySeqSerializer<W> {
    type Ok = ArrayTag;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(PayloadSerializer::seq_element(
            self.writer.fork(),
            self.tag.element_tag().into(),
        ))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.tag)
    }
}

#[derive(Debug)]
pub struct ListSeqSerializer<W> {
    writer: W,
    tag: Tag,
    len: i32,
}

impl<W: NbtWrite> ListSeqSerializer<W> {
    pub fn new(writer: W, len: i32) -> Self {
        assert!(len >= 0, "len cannot be negative but is {}", len);

        Self {
            writer,
            tag: Tag::End,
            len,
        }
    }
}

impl<W: NbtWrite> SerializeSeq for ListSeqSerializer<W> {
    type Ok = ListTag;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        if let Ok(tag) = NotEndTag::try_from(self.tag) {
            value.serialize(PayloadSerializer::seq_element(self.writer.fork(), tag))?;
        } else {
            let serializer = PayloadSerializer::list_head(self.writer.fork(), self.len);
            self.tag = value.serialize(serializer)?.into();
        }

        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        if self.tag == Tag::End {
            // The sequence has no elements thus we do
            // what the reference implementation by Mojang does
            // and set the type to `TAG_End`
            self.writer.start_list(Tag::End, 0i32)?;
        }
        self.writer.end_list()?;
        // FIXME: strict ype for `List`
        Ok(ListTag::List)
    }
}
