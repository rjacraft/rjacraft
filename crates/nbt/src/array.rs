use std::{borrow::Cow, fmt::Debug, io, io::Write};

use serde::{ser::SerializeTupleStruct, Deserialize, Serialize, Serializer};

use crate::{array::sealed::Element, ArrayElementTag, ArrayTag};

/// [` TAG_Byte_Array`](https://wiki.vg/NBT#Specification:byte_array_tag) value.
pub type NbtByteArray<'a> = NbtArray<'a, i8>;

/// [` TAG_Int_Array`](https://wiki.vg/NBT#Specification:int_array_tag) value.
pub type NbtIntArray<'a> = NbtArray<'a, i32>;

/// [` TAG_Long_Array`](https://wiki.vg/NBT#Specification:long_array_tag) value.
pub type NbtLongArray<'a> = NbtArray<'a, i64>;

/// An array which can safely be stored in a NBT.
///
/// This is serialized to either [`TAG_Byte_Array`], [`TAG_Int_Array`] or [`TAG_Long_Array`].
///
/// [`TAG_Byte_Array`]: https://wiki.vg/NBT#Specification:byte_array_tag
/// [`TAG_Int_Array`]: https://wiki.vg/NBT#Specification:int_array_tag
/// [`TAG_Long_Array`]: https://wiki.vg/NBT#Specification:long_array_tag
#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
pub struct NbtArray<'a, T>(Cow<'a, [T]>)
where
    [T]: ToOwned<Owned = Vec<T>>;

/// An error which may occur while creating a [`NbtArray`] from a slice.
#[derive(Debug, thiserror::Error)]
#[error("NBT array length cannot exceed {} but is {0}", i32::MAX)]
pub struct NbtArrayFromSliceError(usize);

impl<'a, T: NbtArrayElement> NbtArray<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub fn empty() -> Self {
        // empty `Vec`s don't require allocations, so we can start with an owned variant
        Self(Cow::Owned(Vec::new()))
    }

    pub fn len(&self) -> i32 {
        // the conversion is always valid since the length should be checked on creation
        self.0.len() as i32
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> Cow<'a, [T]> {
        self.0
    }
}

impl<'a, T: NbtArrayElement> TryFrom<&'a [T]> for NbtArray<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    type Error = NbtArrayFromSliceError;

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        if i32::try_from(value.len()).is_ok() {
            Ok(Self(Cow::Borrowed(value)))
        } else {
            Err(NbtArrayFromSliceError(value.len()))
        }
    }
}

impl<'a, T: NbtArrayElement> TryFrom<Vec<T>> for NbtArray<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    type Error = NbtArrayFromSliceError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if i32::try_from(value.len()).is_ok() {
            Ok(Self(Cow::Owned(value)))
        } else {
            Err(NbtArrayFromSliceError(value.len()))
        }
    }
}

mod sealed {
    pub trait Element {
        const MAGIC: &'static str;
    }

    impl Element for i8 {
        const MAGIC: &'static str = super::magic::BYTE;
    }

    impl Element for i32 {
        const MAGIC: &'static str = super::magic::INT;
    }

    impl Element for i64 {
        const MAGIC: &'static str = super::magic::LONG;
    }
}

/// An element of [`NbtArray`].
///
/// This trait is [sealed] to only permit `i8`, `i32` and `i64`
/// which are the only possible array element types.
///
/// [sealed]: https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/
pub trait NbtArrayElement: Sized + Element + Serialize
where
    [Self]: ToOwned<Owned = Vec<Self>>,
{
    const TAG: ArrayElementTag;

    const ARRAY_TAG: ArrayTag;

    fn write(self, write: impl Write) -> io::Result<()>;
}

impl NbtArrayElement for i8 {
    const TAG: ArrayElementTag = ArrayElementTag::Byte;
    const ARRAY_TAG: ArrayTag = ArrayTag::ByteArray;

    fn write(self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&self.to_be_bytes())
    }
}

impl NbtArrayElement for i32 {
    const TAG: ArrayElementTag = ArrayElementTag::Int;
    const ARRAY_TAG: ArrayTag = ArrayTag::IntArray;

    fn write(self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&self.to_be_bytes())
    }
}

impl NbtArrayElement for i64 {
    const TAG: ArrayElementTag = ArrayElementTag::Long;
    const ARRAY_TAG: ArrayTag = ArrayTag::LongArray;

    fn write(self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&self.to_be_bytes())
    }
}

impl<T: NbtArrayElement> Serialize for NbtArray<'_, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // we use this
        let mut serializer = serializer.serialize_tuple_struct(T::MAGIC, self.0.len())?;
        for element in self.0.iter() {
            serializer.serialize_field(element)?;
        }
        serializer.end()
    }
}

/// Magical identifiers to support special serialization of [`NbtArray`].
pub(crate) mod magic {
    pub(crate) const BYTE: &str = "$__nbt_internal_TAG_Byte_Array";
    pub(crate) const INT: &str = "$__nbt_internal_TAG_Int_Array";
    pub(crate) const LONG: &str = "$__nbt_internal_TAG_Long_Array";
}
