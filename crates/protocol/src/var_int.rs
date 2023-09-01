use std::{
    io::{Cursor, Read, Write},
    num::TryFromIntError,
};

use byteorder::ReadBytesExt;
use thiserror::Error;

use crate::io::{Decoder, Encoder};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct VarInt(pub i32);

impl VarInt {
    pub const MAX_SIZE: usize = 5;

    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn decode_partial(mut r: impl Read) -> Result<i32, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            let byte = r.read_u8().map_err(|_| VarIntDecodeError::Incomplete)?;
            val |= (byte as i32 & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(val);
            }
        }

        Err(VarIntDecodeError::TooLarge)
    }

    pub fn encode_write(&self, mut w: impl Write) -> anyhow::Result<()> {
        let x = self.0 as u64;
        let stage1 = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) >> 3;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

        let merged = stage1 | (msbs & msbmask);
        let bytes = merged.to_le_bytes();

        w.write_all(unsafe { bytes.get_unchecked(..bytes_needed as usize) })?;

        Ok(())
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<VarInt> for i32 {
    fn from(x: VarInt) -> Self {
        x.0
    }
}

impl TryFrom<VarInt> for usize {
    type Error = TryFromIntError;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl TryFrom<usize> for VarInt {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        value.try_into().map(VarInt)
    }
}

impl Decoder for VarInt {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(VarInt(VarInt::decode_partial(buffer)?))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum VarIntDecodeError {
    #[error("incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}

impl Encoder for VarInt {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        self.encode_write(buffer)
    }
}
