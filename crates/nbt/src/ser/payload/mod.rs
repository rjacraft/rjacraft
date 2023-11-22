use std::{
    fmt::Display,
    io::{self, Write},
};

use serde::Serialize;

use crate::{
    ser::{
        adapter::{SerializeSeqAsSerializeTupleStruct, SerializerAdapter},
        macros::unserializable_type,
        map,
        seq,
        seq::{ArraySeqSerializer, ListSeqSerializer},
        structure,
    },
    string::{NbtStr, NbtStrFromStrError},
    ArrayTag,
    NotEndTag,
};

type Result<T> = std::result::Result<T, Error>;
type Impossible = serde::ser::Impossible<NotEndTag, Error>;

#[derive(Debug)]
pub struct PayloadSerializer<'w, W: ?Sized, S> {
    /// Writer to which the data is written
    writer: &'w mut W,
    /// Extra code called when the data is written
    kind: S,
}

/// An error occurring while serializing using [`PayloadSerializer`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O operation failed")]
    Io(#[from] io::Error),
    #[error("the value is of invalid type {0}")]
    InvalidType(NotPayloadType),
    #[error("string is not valid for NBT")]
    InvalidString(#[from] NbtStrFromStrError),
    #[error("invalid value of type struct")]
    InvalidStruct(#[from] Box<structure::Error>),
    #[error("invalid value of type map")]
    InvalidMap(#[from] Box<map::Error>),
    #[error("invalid value of type seq")]
    InvalidSeq(#[from] Box<seq::Error>),
    #[error("array length should be specified explicitly")]
    UnknownArrayLen,
    #[error("array length {0} exceeds the permitted maximum of {}", i32::MAX)]
    ArrayTooBig(usize),
    #[error("list length should be specified explicitly")]
    UnknownListLen,
    #[error("list length {0} exceeds the permitted maximum of {}", i32::MAX)]
    ListTooBig(usize),
    // FIXME: only used for seq
    #[error(
        "sequence should only contain the values of type {expected:?} but {found:?} was found"
    )]
    HeterogeneousSeq {
        expected: NotEndTag,
        found: NotEndTag,
    },
    #[error("{0}")]
    Custom(String),
}

unserializable_type! {
    /// The types which cannot be serialized using [`PayloadSerializer`].
    pub enum NotPayloadType {
        I128, U8, U16, U32, U64, U128,
        Char, Bytes, None, Unit, UnitStruct, UnitVariant,
        NewtypeStruct, NewtypeVariant, Tuple, TupleStruct, TupleVariant, StructVariant,
    }
}

mod payload {
    use std::io::Write;

    use crate::NotEndTag;

    /// The kind of payload stored by this
    pub trait Kind {
        fn write_tag<W: ?Sized + Write>(&self, writer: &mut W, tag: NotEndTag)
            -> super::Result<()>;
    }
}

/// Appends a name to the tag.
#[derive(Debug)]
pub struct NamedPayload<'n>(NbtStr<'n>);
impl payload::Kind for NamedPayload<'_> {
    #[inline]
    fn write_tag<W: ?Sized + Write>(&self, writer: &mut W, tag: NotEndTag) -> Result<()> {
        writer.write_all(&tag.to_be_bytes())?;
        self.0.write(writer)?;
        Ok(())
    }
}

/// Appends a length to the tag.
#[derive(Debug)]
pub struct ListHeadPayload(i32);
impl payload::Kind for ListHeadPayload {
    #[inline]
    fn write_tag<W: ?Sized + Write>(&self, writer: &mut W, tag: NotEndTag) -> Result<()> {
        writer.write_all(&tag.to_be_bytes())?;
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }
}

/// Appends a length to the tag.
#[derive(Debug)]
pub struct SeqElementPayload(NotEndTag);
impl payload::Kind for SeqElementPayload {
    #[inline]
    fn write_tag<W: ?Sized + Write>(&self, _: &mut W, tag: NotEndTag) -> Result<()> {
        if tag != self.0 {
            return Err(Error::HeterogeneousSeq {
                expected: self.0,
                found: tag,
            });
        }
        Ok(())
    }
}

impl<'w, W: ?Sized + Write, S: payload::Kind> PayloadSerializer<'w, W, S> {
    fn begin(&mut self, tag: NotEndTag) -> Result<()> {
        self.kind.write_tag(&mut self.writer, tag)?;
        Ok(())
    }
}

impl<'w, 'n, W: ?Sized + Write> PayloadSerializer<'w, W, NamedPayload<'n>> {
    pub fn named(writer: &'w mut W, name: NbtStr<'n>) -> Self {
        Self {
            writer,
            kind: NamedPayload(name),
        }
    }
}

impl<'w, 'n, W: ?Sized + Write> PayloadSerializer<'w, W, ListHeadPayload> {
    pub fn list_head(writer: &'w mut W, len: i32) -> Self {
        Self {
            writer,
            kind: ListHeadPayload(len),
        }
    }
}

impl<'w, 'n, W: ?Sized + Write> PayloadSerializer<'w, W, SeqElementPayload> {
    pub fn seq_element(writer: &'w mut W, tag: NotEndTag) -> Self {
        Self {
            writer,
            kind: SeqElementPayload(tag),
        }
    }
}

impl<'w, W: ?Sized + Write, S: payload::Kind> serde::Serializer
    for &'w mut PayloadSerializer<'_, W, S>
{
    type Ok = NotEndTag;
    type Error = Error;
    type SerializeSeq = SerializerAdapter<ListSeqSerializer<'w, W>, NotEndTag, Error>;
    type SerializeTuple = Impossible;
    type SerializeTupleStruct =
        SerializeSeqAsSerializeTupleStruct<ArraySeqSerializer<'w, W>, NotEndTag, Error>;
    type SerializeTupleVariant = Impossible;
    type SerializeMap = SerializerAdapter<map::MapSerializer<'w, W>, NotEndTag, Error>;
    type SerializeStruct = SerializerAdapter<structure::StructSerializer<'w, W>, NotEndTag, Error>;
    type SerializeStructVariant = Impossible;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.begin(NotEndTag::Byte)?;
        self.writer.write_all(&(v as u8).to_be_bytes())?;
        Ok(NotEndTag::Byte)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.begin(NotEndTag::Byte)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Byte)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.begin(NotEndTag::Short)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Short)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.begin(NotEndTag::Int)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Int)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.begin(NotEndTag::Long)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Long)
    }

    fn serialize_i128(self, _: i128) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::I128))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::U8))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::U16))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::U32))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::U64))
    }

    fn serialize_u128(self, _: u128) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::U128))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.begin(NotEndTag::Float)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Float)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.begin(NotEndTag::Double)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(NotEndTag::Double)
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::Char))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.begin(NotEndTag::String)?;
        NbtStr::try_from(v)?.write(&mut self.writer)?;
        Ok(NotEndTag::String)
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok> {
        // Minecraft uses signed bytes, so this is unsupported
        Err(Error::InvalidType(NotPayloadType::Bytes))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::None))
    }

    /// Serialize a [`Some(T)`] value.
    ///
    /// While we [cannot](Self::serialize_none) serialize `None` values as they don't have any tag,
    /// it is fine to serialize existing values allowing the following code like:
    ///
    /// ```rust
    /// use serde::Serialize;
    /// use rjacraft_nbt::ser::to_writer;
    ///
    /// #[derive(Serialize)]
    /// struct OptExample {
    ///     #[serde(skip_serializing_if = "Option::is_none")]
    ///     value: Option<i32>,
    /// }
    ///
    /// let mut out_some = vec![];
    /// assert!(
    ///     to_writer(&mut out_some, &OptExample { value: Some(123) }).is_ok(),
    ///     "Some value should serialize just fine",
    /// );
    /// let mut out_none = vec![];
    /// assert!(
    ///     to_writer(&mut out_none, &OptExample { value: None }).is_ok(),
    ///     "None value should not fail serialization thanks to explicit skip",
    /// );
    /// assert!(
    ///     out_some.len() > out_none.len(),
    ///     "None value should just be skipped in the binary output"
    /// );
    /// ```
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::Unit))
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::UnitStruct))
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::UnitVariant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::NewtypeStruct))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok> {
        Err(Error::InvalidType(NotPayloadType::NewtypeVariant))
    }

    // `NBT_Tag_List` is always used for sequences.
    // `NBT_*_Array` types are implemented using a separate `NbtArray` type.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let Some(len) = len else {
            return Err(Error::UnknownListLen);
        };
        let Ok(len) = i32::try_from(len) else {
            return Err(Error::ListTooBig(len));
        };
        self.begin(NotEndTag::List)?;

        Ok(SerializerAdapter::new(ListSeqSerializer::new(
            self.writer,
            len,
        )))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Err(Error::InvalidType(NotPayloadType::Tuple))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        use crate::array;

        // This method may be used by `NbtArray<_>`
        // to implement custom serialization to `NBT_*_Array` types
        let array_tag = match name {
            array::magic::BYTE => ArrayTag::ByteArray,
            array::magic::INT => ArrayTag::IntArray,
            array::magic::LONG => ArrayTag::LongArray,
            _ => return Err(Error::InvalidType(NotPayloadType::TupleStruct)),
        };

        let Ok(len) = i32::try_from(len) else {
            return Err(Error::ListTooBig(len));
        };

        self.writer.write_all(&array_tag.to_be_bytes())?;
        self.writer.write_all(&len.to_be_bytes())?;

        Ok(Self::SerializeTupleStruct::new(ArraySeqSerializer::new(
            self.writer,
            array_tag,
        )))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::InvalidType(NotPayloadType::TupleVariant))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.begin(NotEndTag::Compound)?;

        Ok(Self::SerializeMap::new(map::MapSerializer::new(
            self.writer,
        )))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.begin(NotEndTag::Compound)?;

        Ok(Self::SerializeStruct::new(
            structure::StructSerializer::new(self.writer),
        ))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::InvalidType(NotPayloadType::StructVariant))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

// Error conversions

impl From<structure::Error> for Error {
    fn from(value: structure::Error) -> Self {
        Self::InvalidStruct(Box::new(value))
    }
}

impl From<map::Error> for Error {
    fn from(value: map::Error) -> Self {
        Self::InvalidMap(Box::new(value))
    }
}

impl From<seq::Error> for Error {
    fn from(value: seq::Error) -> Self {
        Self::InvalidSeq(Box::new(value))
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}
