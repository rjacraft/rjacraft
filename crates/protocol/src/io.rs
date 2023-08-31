use std::borrow::Cow;
use std::io::{Cursor, Read};
use std::iter;
use std::marker::PhantomData;

use anyhow::{bail, Context};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;
use uuid::Uuid;

use crate::var_int::VarInt;

pub trait Decoder: Sized {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>;
}

pub trait Encoder {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()>;
}

macro_rules! integer_impl {
    ($($int:ty, $read_fn:tt, $write_fn:tt),* $(,)?) => {
        $(
            impl Decoder for $int {
                fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
                    buffer.$read_fn::<BigEndian>().map_err(anyhow::Error::from)
                }
            }

            impl Encoder for $int {
                fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
                    buffer.$write_fn::<BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

integer_impl! {
    u16, read_u16, write_u16,
    u32, read_u32, write_u32,
    u64, read_u64, write_u64,

    i16, read_i16, write_i16,
    i32, read_i32, write_i32,
    i64, read_i64, write_i64,

    f32, read_f32, write_f32,
    f64, read_f64, write_f64,
}

impl Decoder for u8 {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        buffer.read_u8().map_err(anyhow::Error::from)
    }
}

impl Encoder for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.write_u8(*self)?;
        Ok(())
    }
}

impl Decoder for i8 {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        buffer.read_i8().map_err(anyhow::Error::from)
    }
}

impl Encoder for i8 {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.write_i8(*self)?;
        Ok(())
    }
}

impl Decoder for bool {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(buffer.read_u8()? != 0)
    }
}

impl Encoder for bool {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.write_u8(if *self { 1 } else { 0 })?;
        Ok(())
    }
}

const STRING_MAX_LENGTH: usize = 32767;

impl Decoder for String {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let len = VarInt::decode(buffer)
            .context("failed to decode string length")?
            .0 as usize;

        if len > STRING_MAX_LENGTH {
            bail!(
                "string length {} exceeds maximum allowed length of {}",
                len,
                STRING_MAX_LENGTH
            );
        }

        let mut bytes = vec![0u8; len];
        buffer
            .read_exact(&mut bytes)
            .map_err(|_| Error::UnexpectedEof("String"))?;

        let str = std::str::from_utf8(&bytes).context("failed to decode string")?;

        Ok(str.to_owned())
    }
}

impl Encoder for String {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let bytes = self.as_bytes();
        if bytes.len() > STRING_MAX_LENGTH {
            bail!(
                "string length {} exceeds maximum allowed length of {}",
                bytes.len(),
                STRING_MAX_LENGTH
            );
        }

        VarInt(bytes.len() as i32).encode(buffer)?;
        buffer.extend_from_slice(bytes);
        Ok(())
    }
}

impl Decoder for Uuid {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut bytes = [0u8; 16];
        buffer
            .read_exact(&mut bytes)
            .map_err(|_| Error::UnexpectedEof("Uuid"))?;

        Ok(Uuid::from_bytes(bytes))
    }
}

impl Encoder for Uuid {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.extend_from_slice(self.as_bytes());
        Ok(())
    }
}

impl<T> Decoder for Option<T>
where
    T: Decoder,
{
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let present = bool::decode(buffer)?;
        if present {
            Ok(Some(T::decode(buffer)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> Encoder for Option<T>
where
    T: Encoder,
{
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            Some(value) => {
                true.encode(buffer)?;
                value.encode(buffer)?;
            }
            None => {
                false.encode(buffer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of input: failed to read value of type `{0}`")]
    UnexpectedEof(&'static str),
}

const MAX_LENGTH: usize = 1024 * 1024; // 2^20 elements

pub struct LengthPrefixedVec<'a, P, T>(pub Cow<'a, [T]>, PhantomData<P>)
where
    [T]: ToOwned<Owned = Vec<T>>;

impl<'a, P, T> Decoder for LengthPrefixedVec<'a, P, T>
where
    T: Decoder,
    [T]: ToOwned<Owned = Vec<T>>,
    P: Decoder + TryInto<usize>,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let len = P::decode(buffer)?.try_into()?;

        if len > MAX_LENGTH {
            bail!(
                "length {} exceeds maximum allowed length of {}",
                len,
                MAX_LENGTH
            )
        }

        let vec = iter::repeat_with(|| T::decode(buffer))
            .take(len)
            .collect::<anyhow::Result<Vec<T>>>()?;

        Ok(Self(Cow::Owned(vec), PhantomData))
    }
}

impl<'a, P, T> Encoder for LengthPrefixedVec<'a, P, T>
where
    T: Encoder,
    [T]: ToOwned<Owned = Vec<T>>,
    P: Encoder + TryFrom<usize>,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        P::try_from(self.0.len())?.encode(buffer)?;

        self.0.iter().enumerate().for_each(|(index, item)| {
            item.encode(buffer)
                .context(format!("failed to encode element at index {}", index))
                .unwrap();
        });

        Ok(())
    }
}

impl<'a, P, T> From<LengthPrefixedVec<'a, P, T>> for Vec<T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(x: LengthPrefixedVec<'a, P, T>) -> Self {
        x.0.into_owned()
    }
}

impl<'a, P, T> From<&'a [T]> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(slice: &'a [T]) -> Self {
        Self(Cow::Borrowed(slice), PhantomData)
    }
}

impl<'a, P, T> From<Vec<T>> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }
}

pub type VarIntPrefixedVec<'a, T> = LengthPrefixedVec<'a, VarInt, T>;
pub type ShortPrefixedVec<'a, T> = LengthPrefixedVec<'a, u16, T>;

pub struct LengthInferredVecU8<'a>(pub Cow<'a, [u8]>);

impl<'a> Decoder for LengthInferredVecU8<'a> {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut vec = Vec::new();
        buffer.read_to_end(&mut vec)?;
        Ok(LengthInferredVecU8(Cow::Owned(vec)))
    }
}

impl<'a> Encoder for LengthInferredVecU8<'a> {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.extend_from_slice(&self.0);
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for LengthInferredVecU8<'a> {
    fn from(slice: &'a [u8]) -> Self {
        LengthInferredVecU8(Cow::Borrowed(slice))
    }
}

impl<'a> From<LengthInferredVecU8<'a>> for Vec<u8> {
    fn from(x: LengthInferredVecU8<'a>) -> Self {
        x.0.into_owned()
    }
}
