//! A compact bit vector

use bitvec::prelude::*;
use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("Failed to read BitVec length")]
    Length(#[from] super::varint::DecodeError),
}

impl ProtocolType for BitVec<u64, Lsb0> {
    type DecodeError = DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        const BITS: usize = u64::BITS as usize;

        let super::VarInt(longs) = super::VarInt::decode(buffer)?;
        if buffer.remaining() < longs as usize * 8 {
            Err(error::Eof)?
        }

        let mut result = Self::with_capacity(longs as usize * BITS);

        for i in 0..longs as usize {
            result[i..i + BITS].store(buffer.get_u64());
        }

        Ok(result)
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        const BITS: usize = u64::BITS as usize;

        let longs = self.len().div_ceil(BITS);
        super::VarInt(longs as i32).encode(buffer)?;

        for i in 0..longs {
            let upper = Ord::min((i + 1) * BITS, self.len());
            buffer.put_u64(self[i * BITS..upper].load());
        }

        Ok(())
    }
}
