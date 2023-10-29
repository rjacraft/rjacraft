//! The variable-length integer type. Backed by an [`i32`]

use bytes::{Buf, BufMut};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::{error, ProtocolType, ProtocolTypeRaw};

pub const MAX_SIZE: usize = 5;

#[derive(Debug, thiserror::Error, nevermore::FromNever)]
pub enum DecodeError {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("The var int is larger than {MAX_SIZE}")]
    TooLarge,
}

#[derive(Debug, Clone)]
pub struct VarInt(pub i32);

impl VarInt {
    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl ProtocolType for VarInt {
    type DecodeError = DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let mut val = 0;

        for i in 0..MAX_SIZE {
            if buffer.remaining() == 0 {
                Err(error::Eof)?;
            }

            let byte = buffer.get_u8();
            val |= (byte as i32 & 0b01111111) << (i * 7);

            if byte & 0b10000000 == 0 {
                return Ok(Self(val));
            }
        }

        Err(DecodeError::TooLarge)
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
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

        buffer.put(&bytes[..bytes_needed as usize]);

        Ok(())
    }
}

impl ProtocolTypeRaw for VarInt {
    async fn decode_raw(
        read: &mut (impl io::AsyncRead + Unpin + Send),
    ) -> io::Result<Result<Self, Self::DecodeError>> {
        let mut val = 0;
        let mut byte = [0];

        for i in 0..MAX_SIZE {
            if read.read_exact(&mut byte).await? == 0 {
                return Ok(Err(Self::DecodeError::Eof(error::Eof)));
            }

            val |= (byte[0] as i32 & 0b01111111) << (i * 7);

            if byte[0] & 0b10000000 == 0 {
                return Ok(Ok(Self(val)));
            }
        }

        Ok(Err(DecodeError::TooLarge))
    }

    async fn encode_raw(
        &self,
        write: &mut (impl io::AsyncWrite + Unpin + Send),
    ) -> io::Result<Result<(), Self::EncodeError>> {
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

        write.write_all(&bytes[..bytes_needed as usize]).await?;

        Ok(Ok(()))
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}
