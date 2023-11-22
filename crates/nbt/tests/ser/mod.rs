mod basic;
mod servers;

macro_rules! concat_byte_vec {
    ($($part:expr),* $(,)?) => {{
        let mut bytes = ::std::vec![0u8; 0];
        $(
            ::core::iter::Extend::extend(
                &mut bytes,
                ::core::iter::IntoIterator::into_iter($part),
            );
        )*
        bytes
    }};
}
use concat_byte_vec;

struct U16LenStr<'a>(&'a [u8]);

impl<'a> U16LenStr<'a> {
    fn new(string: &'a str) -> Self {
        assert!(
            u16::try_from(string.len()).is_ok(),
            "string should consist of at most {} bytes",
            u16::MAX
        );

        Self(string.as_bytes())
    }

    fn len(&self) -> u16 {
        self.0
            .len()
            .try_into()
            .expect("length should be validated on creation")
    }
}

impl<'a> IntoIterator for U16LenStr<'a> {
    type Item = u8;
    type IntoIter =
        Chain<<[u8; 2] as IntoIterator>::IntoIter, Copied<<&'a [u8] as IntoIterator>::IntoIter>>;

    fn into_iter(self) -> Self::IntoIter {
        self.len()
            .to_be_bytes()
            .into_iter()
            .chain(self.0.iter().copied())
    }
}

fn to_bytes_named<V: ?Sized + Serialize>(name: &str, value: &V) -> Result<Vec<u8>, ser::Error> {
    let mut bytes = vec![];
    ser::to_writer_named(
        &mut bytes,
        NbtStr::try_from(name).expect("invalid name string"),
        value,
    )?;

    Ok(bytes)
}

fn to_bytes<V: ?Sized + Serialize>(value: &V) -> Result<Vec<u8>, ser::Error> {
    let mut bytes = vec![];
    ser::to_writer(&mut bytes, value)?;

    Ok(bytes)
}

use std::iter::{Chain, Copied};

use rjacraft_nbt::{ser, string::NbtStr};
use serde::Serialize;

mod tag {
    pub const END: u8 = 0;
    pub const BYTE: u8 = 1;
    pub const SHORT: u8 = 2;
    pub const INT: u8 = 3;
    pub const LONG: u8 = 4;
    pub const FLOAT: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const BYTE_ARRAY: u8 = 7;
    pub const STRING: u8 = 8;
    pub const LIST: u8 = 9;
    pub const COMPOUND: u8 = 10;
    pub const INT_ARRAY: u8 = 11;
    pub const LONG_ARRAY: u8 = 12;
}
