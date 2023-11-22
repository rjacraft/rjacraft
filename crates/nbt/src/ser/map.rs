use std::{
    fmt::Display,
    io::{self, Write},
};

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::{
    ser::{payload, unserializable_type, PayloadSerializer},
    string::{NbtStr, NbtStrFromStrError},
    CompoundTag,
    Tag,
};

/// An error occurring while serializing using [`MapSerializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("the key should be of string type but is of type {0}")]
    InvalidKeyType(NotMapKeyType),
    #[error("the key is invalid")]
    InvalidKey(#[from] NbtStrFromStrError),
    #[error("failed to deserialize value")]
    InvalidValue(#[from] Box<payload::Error>),
    #[error("{0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl From<payload::Error> for Error {
    fn from(value: payload::Error) -> Self {
        Self::from(Box::new(value))
    }
}

unserializable_type! {
    /// The types which cannot be serialized using [`MapSerializer`].
    pub enum NotMapKeyType {
        Bool, I8, I16, I32, I64, I128, U16, U32, U64, U8, U128, F32, F64,
        Char, Str, Bytes, None, Some, EmptyTuple, Unit, UnitStruct, UnitVariant,
        NewtypeStruct, NewtypeVariant, Seq,
        Tuple, TupleStruct, TupleVariant, Map, Struct, StructVariant,
    }
}

// type Result<T> = std::result::Result<T, KeyError>;
type ImpossibleKey = serde::ser::Impossible<(), Error>;

#[derive(Debug)]
pub struct MapSerializer<'w, W: ?Sized> {
    writer: &'w mut W,
}

impl<'w, W: ?Sized> MapSerializer<'w, W> {
    #[inline]
    pub fn new(writer: &'w mut W) -> Self {
        Self { writer }
    }
}

impl<'w, W: ?Sized + Write> SerializeMap for MapSerializer<'w, W> {
    type Ok = CompoundTag;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<(), Error> {
        todo!()
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Error> {
        todo!()
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        key.serialize(EntrySerializer {
            writer: self.writer,
            value,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(&Tag::End.to_be_bytes())?;
        Ok(CompoundTag::Compound)
    }
}

struct EntrySerializer<'w, 'v, W: ?Sized, V: ?Sized> {
    writer: &'w mut W,
    value: &'v V,
}

impl<W: ?Sized + Write, V: ?Sized + Serialize> Serializer for EntrySerializer<'_, '_, W, V> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ImpossibleKey;
    type SerializeTuple = ImpossibleKey;
    type SerializeTupleStruct = ImpossibleKey;
    type SerializeTupleVariant = ImpossibleKey;
    type SerializeMap = ImpossibleKey;
    type SerializeStruct = ImpossibleKey;
    type SerializeStructVariant = ImpossibleKey;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Bool))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::I8))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::I16))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::I32))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::I64))
    }

    fn serialize_i128(self, _: i128) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::I128))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::U8))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::U16))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::U32))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::U64))
    }

    fn serialize_u128(self, _: u128) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::U128))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::F32))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::F64))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Char))
    }

    fn serialize_str(self, key: &str) -> Result<Self::Ok, Self::Error> {
        self.value.serialize(&mut PayloadSerializer::named(
            self.writer,
            NbtStr::try_from(key)?,
        ))?;
        Ok(())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Bytes))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::None))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Some))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Unit))
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::UnitStruct))
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::UnitVariant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::NewtypeStruct))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::NewtypeVariant))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Seq))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Tuple))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::TupleStruct))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::TupleVariant))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Map))
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::Struct))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidKeyType(NotMapKeyType::StructVariant))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
