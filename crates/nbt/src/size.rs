use std::hint::unreachable_unchecked;

use serde::{Deserialize, Deserializer, Serialize};

/// An index of an NBT collection.
/// Effectively, an `i31` value, i.e. an unsigned 31-bit value.
// #[rustc_layout_scalar_valid_range_start(0)])]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Serialize)]
pub struct NbtUsize(i32);

impl NbtUsize {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: i32) -> Option<Self> {
        if value.is_negative() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn to_be_bytes(self) -> [u8; 4] {
        self.assume_invariant();
        self.0.to_be_bytes()
    }

    #[inline(always)]
    const fn assume_invariant(self) {
        if self.0.is_negative() {
            // SAFETY: this type can only be created with a positive value
            unsafe { unreachable_unchecked() }
        }
    }
}

impl From<NbtUsize> for i32 {
    fn from(value: NbtUsize) -> Self {
        value.assume_invariant();
        value.0
    }
}

#[derive(Debug, thiserror::Error)]
#[error("NBT index should be non-negative but is {0}")]
pub struct NbtIndexFromI32Error(i32);

impl TryFrom<i32> for NbtUsize {
    type Error = NbtIndexFromI32Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(NbtIndexFromI32Error(value))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("NBT index should be non-negative but is {0}")]
pub struct NbtIndexFromUsizeError(usize);

impl TryFrom<usize> for NbtUsize {
    type Error = NbtIndexFromUsizeError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let Ok(i32_value) = i32::try_from(value) else {
            return Err(NbtIndexFromUsizeError(value));
        };

        Self::new(i32_value).ok_or(NbtIndexFromUsizeError(value))
    }
}

impl<'de> Deserialize<'de> for NbtUsize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let value = i32::deserialize(deserializer)?;
        Self::new(value)
            .ok_or_else(|| Error::custom(format_args!("NbtIndex cannot be negative {value}")))
    }
}
