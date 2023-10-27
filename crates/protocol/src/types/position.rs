//! A u64 bitfield used for position in many packets.

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

#[derive(Debug, Clone, Copy)]
pub struct Position(pub u32, pub u16, pub u32);

impl ProtocolType for Position {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::Primitive(value) = super::Primitive::<u64>::decode(buffer)?;

        Ok(Self(
            (value >> 38) as u32,
            (value << 26 >> 38) as u16,
            (value << 52 >> 52) as u32,
        ))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let Position(x, y, z) = *self;
        super::Primitive(
            ((x as u64 & 0x3FFFFFF) << 38) | ((z as u64 & 0x3FFFFFF) << 12) | (y as u64 & 0xFFF),
        )
        .encode(buffer)?;

        Ok(())
    }
}
