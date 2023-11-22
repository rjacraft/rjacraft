mod adapter;
mod macros;
mod map;
mod payload;
mod seq;
mod structure;

use std::{
    fmt::Display,
    io::{self, Write},
};

use adapter::SerializerAdapter;
use serde::Serialize;

use self::macros::unserializable_type;
pub use self::payload::PayloadSerializer;
use crate::{string::NbtStr, CompoundTag, Tag};

/// An error occurring while serializing using [`RootSerializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("the value should either be of map or struct type but is of type {0}")]
    InvalidType(NotRootType),
    #[error("invalid value of type struct")]
    InvalidStruct(#[from] structure::Error),
    #[error("invalid value of type map")]
    InvalidMap(#[from] map::Error),
    #[error("{0}")]
    Custom(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

unserializable_type! {
    /// The types which cannot be serialized using [`RootSerializer`].
    pub enum NotRootType {
        Bool, I8, I16, I32, I64, U8, I128, U16, U32, U64, U128, F32, F64,
        Char, Str, Bytes, None, Some, EmptyTuple, Unit, UnitStruct, UnitVariant,
        NewtypeStruct, NewtypeVariant, Seq, Tuple, TupleStruct, TupleVariant, StructVariant,
    }
}

type Result<T> = std::result::Result<T, Error>;
type Impossible = serde::ser::Impossible<CompoundTag, Error>;

pub fn to_writer<W: Write, T: ?Sized + Serialize>(writer: &mut W, value: &T) -> Result<()> {
    to_writer_named(writer, NbtStr::empty(), value)
}

pub fn to_writer_named<W: Write, T: ?Sized + Serialize>(
    writer: &mut W,
    name: NbtStr,
    value: &T,
) -> Result<()> {
    let mut ser = RootSerializer::new(writer, name);
    let _ = value.serialize(&mut ser)?;
    Ok(())
}

#[derive(Debug)]
pub struct RootSerializer<'n, W> {
    writer: W,
    name: NbtStr<'n>,
}

impl<'n, W: Write> RootSerializer<'n, W> {
    /// Creates a new NBT serializer.
    #[inline]
    pub fn new(writer: W, name: NbtStr<'n>) -> Self {
        Self { writer, name }
    }

    fn begin(&mut self) -> Result<()> {
        self.writer.write_all(&Tag::Compound.to_be_bytes())?;
        self.name.write(&mut self.writer)?;
        Ok(())
    }
}

impl<'w, W: Write> serde::Serializer for &'w mut RootSerializer<'w, W> {
    type Ok = CompoundTag;
    type Error = Error;
    type SerializeSeq = Impossible;
    type SerializeTuple = Impossible;
    type SerializeTupleStruct = Impossible;
    type SerializeTupleVariant = Impossible;
    type SerializeMap = SerializerAdapter<map::MapSerializer<'w, W>, CompoundTag, Error>;
    type SerializeStruct =
        SerializerAdapter<structure::StructSerializer<'w, W>, CompoundTag, Error>;
    type SerializeStructVariant = Impossible;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Bool))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::I8))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::I16))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::I32))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::I64))
    }

    fn serialize_i128(self, _: i128) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::I128))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::U8))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::U16))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::U32))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::U64))
    }

    fn serialize_u128(self, _: u128) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::U128))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::F32))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::F64))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Char))
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Str))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Bytes))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::None))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Some))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::Unit))
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::UnitStruct))
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::UnitVariant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::NewtypeStruct))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotRootType::NewtypeVariant))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::InvalidType(NotRootType::Seq))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Err(Error::InvalidType(NotRootType::Tuple))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::InvalidType(NotRootType::TupleStruct))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::InvalidType(NotRootType::TupleVariant))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.begin()?;
        Ok(SerializerAdapter::new(map::MapSerializer::new(
            &mut self.writer,
        )))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.begin()?;
        Ok(SerializerAdapter::new(structure::StructSerializer::new(
            &mut self.writer,
        )))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::InvalidType(NotRootType::StructVariant))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
