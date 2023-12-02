//! Odd position types

use bitfield_struct::bitfield;
use rjacraft_macro::ProtocolType;

use super::Primitive;
use crate::{error, ProtocolType};

#[derive(ProtocolType)]
#[bitfield(u64)]
pub struct BlockPos {
    #[bits(12)]
    pub y: i16,
    #[bits(26)]
    pub z: i32,
    #[bits(26)]
    pub x: i32,
}

#[derive(Debug)]
pub struct BlockPosColumn {
    pub x: u8,
    pub y: i16,
    pub z: u8,
}

impl ProtocolType for BlockPosColumn {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() < 3 {
            Err(error::Eof)?;
        }

        let xz = buffer.get_u8();

        Ok(Self {
            x: xz >> 4,
            y: buffer.get_i16(),
            z: xz & 0b00001111,
        })
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        buffer.put_u8(self.x << 4 | self.z);
        buffer.put_i16(self.y);

        Ok(())
    }
}
