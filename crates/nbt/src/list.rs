use std::borrow::Cow;

use serde::Serialize;

use crate::compound::NbtCompound;

/// [NBT List](https://wiki.vg/NBT#Specification:list_tag) value.
#[derive(Debug, PartialEq, Clone)]
pub enum NbtList<'a> {
    End,
    Byte(RawNbtList<'a, i8>),
    Short(RawNbtList<'a, i16>),
    Int(RawNbtList<'a, i32>),
    Long(RawNbtList<'a, i64>),
    Float(RawNbtList<'a, f32>),
    Double(RawNbtList<'a, f64>),
    ByteArray(RawNbtList<'a, RawNbtList<'a, i8>>),
    // TODO: do we even need boxing here? Typeck says that we do,
    //  but repr of NbtList is either `&NbtList` or `Vec<NbtList>`
    List(Box<RawNbtList<'a, NbtList<'a>>>),
    Compound(RawNbtList<'a, NbtCompound<'a>>),
    IntArray(RawNbtList<'a, RawNbtList<'a, i32>>),
    LongArray(RawNbtList<'a, RawNbtList<'a, i64>>),
}

impl<'a> NbtList<'a> {
    pub fn len(&self) -> i32 {
        match self {
            NbtList::End => 0,
            NbtList::Byte(l) => l.len(),
            NbtList::Short(l) => l.len(),
            NbtList::Int(l) => l.len(),
            NbtList::Long(l) => l.len(),
            NbtList::Float(l) => l.len(),
            NbtList::Double(l) => l.len(),
            NbtList::ByteArray(l) => l.len(),
            NbtList::List(l) => l.len(),
            NbtList::Compound(l) => l.len(),
            NbtList::IntArray(l) => l.len(),
            NbtList::LongArray(l) => l.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            NbtList::End => true,
            NbtList::Byte(l) => l.is_empty(),
            NbtList::Short(l) => l.is_empty(),
            NbtList::Int(l) => l.is_empty(),
            NbtList::Long(l) => l.is_empty(),
            NbtList::Float(l) => l.is_empty(),
            NbtList::Double(l) => l.is_empty(),
            NbtList::ByteArray(l) => l.is_empty(),
            NbtList::List(l) => l.is_empty(),
            NbtList::Compound(l) => l.is_empty(),
            NbtList::IntArray(l) => l.is_empty(),
            NbtList::LongArray(l) => l.is_empty(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub struct RawNbtList<'a, T>(Cow<'a, [T]>)
where
    [T]: ToOwned<Owned = Vec<T>>;

/// An error which may occur while creating a [`RawNbtList`] from a slice.
#[derive(Debug, thiserror::Error)]
#[error("NBT array length cannot exceed {} but is {0}", i32::MAX)]
pub struct NbtListFromSliceError(usize);

impl<'a, T> RawNbtList<'a, T>
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

impl<'a, T> TryFrom<&'a [T]> for RawNbtList<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    type Error = NbtListFromSliceError;

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        if i32::try_from(value.len()).is_ok() {
            Ok(Self(Cow::Borrowed(value)))
        } else {
            Err(NbtListFromSliceError(value.len()))
        }
    }
}

impl<'a, T> TryFrom<Vec<T>> for RawNbtList<'a, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    type Error = NbtListFromSliceError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if i32::try_from(value.len()).is_ok() {
            Ok(Self(Cow::Owned(value)))
        } else {
            Err(NbtListFromSliceError(value.len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::list::RawNbtList;

    #[test]
    fn lists_should_have_i32_size() {
        // NOTE: `vec![(), <size>]` cannot be used here since it works for too long
        // due to a missing specialization
        assert!(
            RawNbtList::try_from([(); 1].repeat(i32::MAX as usize)).is_ok(),
            "lists should permit the size up to {}",
            i32::MAX
        );
        assert!(
            RawNbtList::try_from([(); 1].repeat(i32::MAX as usize + 1)).is_err(),
            "lists should have the size of at most {}",
            i32::MAX
        );
    }
}
