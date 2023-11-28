use std::collections::BTreeMap;

use subenum::subenum;

use crate::{array::NbtArray, list::NbtList, string::NbtStr};

pub mod array;
pub mod compound;
// pub mod de; // FIXME
pub mod list;
pub mod ser;
mod size;
pub mod string;
pub mod write;

// no need for the extra `size` module
pub use size::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Nbt<'a> {
    /// [`TAG_End`](https://wiki.vg/NBT#Specification:end_tag),
    /// signifies the end of a [`TAG_Compound`].
    /// It is only ever used inside a [`TAG_Compound`],
    /// and is not named despite being in a [`TAG_Compound`].
    ///
    /// [`TAG_Compound`]: [`Self::Compound`]
    End,
    /// [`TAG_Byte`](https://wiki.vg/NBT#Specification:byte_tag),
    /// a single **signed** byte.
    Byte(i8),
    /// [`TAG_Short`](https://wiki.vg/NBT#Specification:short_tag),
    /// a single **signed**, big endian 16 bit integer.
    Short(i16),
    /// [`TAG_Int`](https://wiki.vg/NBT#Specification:int_tag),
    /// a single **signed**, big endian 32 bit integer.
    Int(i32),
    /// [`TAG_Long`](https://wiki.vg/NBT#Specification:long_tag),
    /// a single **signed**, big endian 64 bit integer.
    Long(i64),
    /// [`TAG_Float`](https://wiki.vg/NBT#Specification:float_tag),
    /// a single, big endian [IEEE-754] single-precision floating point number (NaN possible).
    ///
    /// [IEEE-754]: http://en.wikipedia.org/wiki/IEEE_754-2008
    Float(f32),
    /// [`TAG_Double`](https://wiki.vg/NBT#Specification:double_tag),
    /// a single, big endian IEEE-754 double-precision floating point number (NaN possible).
    ///
    /// [IEEE-754]: http://en.wikipedia.org/wiki/IEEE_754-2008
    Double(f64),
    /// [`TAG_Byte_Array`](https://wiki.vg/NBT#Specification:byte_array_tag),
    /// a length-prefixed array of **signed** bytes.
    /// The prefix is a **signed** integer (thus 4 bytes).
    ByteArray(NbtArray<'a, i8>),
    /// [`TAG_String`](https://wiki.vg/NBT#Specification:string_tag),
    /// length-prefixed [modified UTF-8] string.
    /// The prefix is an **unsigned** short (thus 2 bytes)
    /// signifying the length of the string in bytes.
    ///
    /// [modified UTF-8]: https://en.wikipedia.org/wiki/UTF-8#Modified_UTF-8
    List(NbtList<'a>),
    /// [`TAG_List`](https://wiki.vg/NBT#Specification:list_tag),
    /// A list of nameless tags, all of the same type.
    /// The list is prefixed with the Type ID of the items it contains (thus 1 byte),
    /// and the length of the list as a **signed** integer (a further 4 bytes).
    /// If the length of the list is 0 or negative,
    /// the type may be `0` ([`TAG_End`][Self::End]) but otherwise it must be any other type.
    /// (The notchian implementation uses `TAG_End` in that situation,
    /// but another reference implementation by Mojang uses `1` instead;
    /// parsers should accept any type if the length is `<= 0`).
    Compound(BTreeMap<NbtStr<'a>, Nbt<'a>>),
    /// [`TAG_Int_Array`](https://wiki.vg/NBT#Specification:int_array_tag),
    /// a length-prefixed array of **signed** integers.
    /// The prefix is a **signed** integer (thus 4 bytes)
    /// and indicates the number of 4 byte integers.
    IntArray(NbtArray<'a, i32>),
    /// [`TAG_Long_Array`](https://wiki.vg/NBT#Specification:long_array_tag),
    /// a length-prefixed array of **signed** longs.
    /// The prefix is a **signed** integer (thus 4 bytes)
    /// and indicates the number of 8 byte longs.
    LongArray(NbtArray<'a, i64>),
}

/// NBT tag.
#[subenum(NotEndTag, ArrayTag, ListTag, CompoundTag, ArrayElementTag)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    End = 0,
    #[subenum(NotEndTag, ArrayElementTag)]
    Byte = 1,
    #[subenum(NotEndTag)]
    Short = 2,
    #[subenum(NotEndTag, ArrayElementTag)]
    Int = 3,
    #[subenum(NotEndTag, ArrayElementTag)]
    Long = 4,
    #[subenum(NotEndTag)]
    Float = 5,
    #[subenum(NotEndTag)]
    Double = 6,
    #[subenum(NotEndTag, ArrayTag)]
    ByteArray = 7,
    #[subenum(NotEndTag)]
    String = 8,
    #[subenum(NotEndTag, ListTag)]
    List = 9,
    #[subenum(NotEndTag, CompoundTag)]
    Compound = 10,
    #[subenum(NotEndTag, ArrayTag)]
    IntArray = 11,
    #[subenum(NotEndTag, ArrayTag)]
    LongArray = 12,
}

/// An error which may occur while trying to parse [`Tag`] from raw [`u8`] value.
#[derive(Debug, thiserror::Error, Clone, Copy)]
#[error("invalid tag ID {0}")]
pub struct TagFromU8Error(
    /// Raw value which is not a valid NBT tag.
    u8,
);

impl Tag {
    pub const fn to_be_bytes(self) -> [u8; 1] {
        (self as u8).to_be_bytes()
    }
}

impl TryFrom<u8> for Tag {
    type Error = TagFromU8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::End,
            1 => Self::Byte,
            2 => Self::Short,
            3 => Self::Int,
            4 => Self::Long,
            5 => Self::Float,
            6 => Self::Double,
            7 => Self::ByteArray,
            8 => Self::String,
            9 => Self::List,
            10 => Self::Compound,
            11 => Self::IntArray,
            12 => Self::LongArray,
            value => return Err(TagFromU8Error(value)),
        })
    }
}

impl NotEndTag {
    pub const fn to_be_bytes(self) -> [u8; 1] {
        (self as u8).to_be_bytes()
    }
}

impl ArrayTag {
    pub fn element_tag(self) -> ArrayElementTag {
        match self {
            Self::ByteArray => ArrayElementTag::Byte,
            Self::IntArray => ArrayElementTag::Int,
            Self::LongArray => ArrayElementTag::Long,
        }
    }

    pub const fn to_be_bytes(self) -> [u8; 1] {
        (self as u8).to_be_bytes()
    }
}

impl ArrayElementTag {
    pub fn array_tag(self) -> ArrayTag {
        match self {
            Self::Byte => ArrayTag::ByteArray,
            Self::Int => ArrayTag::IntArray,
            Self::Long => ArrayTag::LongArray,
        }
    }

    pub const fn to_be_bytes(self) -> [u8; 1] {
        (self as u8).to_be_bytes()
    }
}

impl From<ListTag> for NotEndTag {
    fn from(value: ListTag) -> Self {
        match value {
            ListTag::List => NotEndTag::List,
        }
    }
}

impl From<CompoundTag> for NotEndTag {
    fn from(value: CompoundTag) -> Self {
        match value {
            CompoundTag::Compound => NotEndTag::Compound,
        }
    }
}

impl From<ArrayTag> for NotEndTag {
    fn from(value: ArrayTag) -> Self {
        match value {
            ArrayTag::ByteArray => NotEndTag::ByteArray,
            ArrayTag::IntArray => NotEndTag::IntArray,
            ArrayTag::LongArray => NotEndTag::LongArray,
        }
    }
}

impl From<ArrayElementTag> for NotEndTag {
    fn from(value: ArrayElementTag) -> Self {
        match value {
            ArrayElementTag::Byte => NotEndTag::Byte,
            ArrayElementTag::Int => NotEndTag::Int,
            ArrayElementTag::Long => NotEndTag::Long,
        }
    }
}
