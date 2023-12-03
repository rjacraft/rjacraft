//! The variable-length integer type. Backed by either [`i32`] or [`i64`]

use bytes::{Buf, BufMut};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::{error, ProtocolType, ProtocolTypeIo};

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError<const BITS: u32> {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("The var int is larger than {BITS} bits")]
    TooLarge,
}

pub type I32DecodeError = DecodeError<{ i32::BITS }>;
pub type I64DecodeError = DecodeError<{ i64::BITS }>;

fn decode_generic<const BITS: u32>(buffer: &mut impl Buf) -> Result<u128, DecodeError<BITS>> {
    let mut result = 0;
    let mut bit = 0;

    loop {
        if buffer.remaining() == 0 {
            return Err(DecodeError::Eof(error::Eof));
        }

        let byte = buffer.get_u8();
        result |= (byte as u128 & 0b01111111) << bit;

        if byte & 0b10000000 == 0 {
            return Ok(result);
        }

        bit += 7;

        if bit > BITS {
            return Err(DecodeError::TooLarge);
        }
    }
}

fn encode_generic(buffer: &mut impl BufMut, mut value: u128) {
    loop {
        let byte = (value as u8) & 0b01111111;
        value >>= 7;
        if value == 0 {
            buffer.put_u8(byte);
            break;
        } else {
            buffer.put_u8(byte | 0b10000000);
        }
    }
}

async fn decode_generic_raw<const BITS: u32>(
    read: &mut (impl io::AsyncRead + Unpin + Send),
) -> io::Result<u128> {
    let mut result = 0;
    let mut bit = 0;
    let mut byte = [0];

    loop {
        read.read_exact(&mut byte).await?;

        result |= (byte[0] as u128 & 0b01111111) << bit;

        if byte[0] & 0b10000000 == 0 {
            return Ok(result.into());
        }

        bit += 7;

        if bit > BITS {
            return Err(io::Error::new(io::ErrorKind::Other, "var int is too large"));
        }
    }
}

async fn encode_generic_raw(
    write: &mut (impl io::AsyncWrite + Unpin + Send),
    mut value: u128,
) -> io::Result<()> {
    let mut result = [0; 8];
    let mut i = 0;

    loop {
        result[i] = (value as u8) & 0b01111111;
        value >>= 7;

        if value == 0 {
            i += 1;
            break;
        } else {
            result[i] |= 0b10000000;
            i += 1;
        }
    }

    write.write_all(&result[..i]).await?;

    Ok(())
}

pub fn written_size<const BITS: u32>(value: u128) -> usize {
    let used_bits = BITS - (value.leading_zeros() - (u128::BITS - BITS));

    usize::div_ceil(used_bits as usize, 7).max(1)
}

#[derive(Debug)]
pub struct VarInt<T>(pub T);

impl ProtocolType for VarInt<i32> {
    type DecodeError = I32DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        Ok(Self(decode_generic::<{ i32::BITS }>(buffer)? as i32))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        encode_generic(buffer, self.0 as u32 as u128);

        Ok(())
    }
}

/// The var int that everyone calls var int
#[async_trait::async_trait]
impl ProtocolTypeIo for VarInt<i32> {
    async fn decode_io(read: &mut (impl io::AsyncRead + Unpin + Send)) -> io::Result<Self> {
        decode_generic_raw::<{ i32::BITS }>(read)
            .await
            .map(|x| Self(x as i32))
    }

    async fn encode_io(&self, write: &mut (impl io::AsyncWrite + Unpin + Send)) -> io::Result<()> {
        encode_generic_raw(write, self.0 as u32 as u128).await
    }

    fn written_size(&self) -> usize {
        written_size::<{ i32::BITS }>(self.0 as u32 as u128)
    }
}

impl From<i32> for VarInt<i32> {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<VarInt<i32>> for i32 {
    fn from(value: VarInt<i32>) -> Self {
        value.0
    }
}

/// The var int that everyone calls var long
impl ProtocolType for VarInt<i64> {
    type DecodeError = I64DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        Ok(Self(decode_generic::<{ i64::BITS }>(buffer)? as i64))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        encode_generic(buffer, self.0 as u32 as u128);

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProtocolTypeIo for VarInt<i64> {
    async fn decode_io(read: &mut (impl io::AsyncRead + Unpin + Send)) -> io::Result<Self> {
        decode_generic_raw::<{ i64::BITS }>(read)
            .await
            .map(|x| Self(x as i64))
    }

    async fn encode_io(&self, write: &mut (impl io::AsyncWrite + Unpin + Send)) -> io::Result<()> {
        encode_generic_raw(write, self.0 as u64 as u128).await
    }

    fn written_size(&self) -> usize {
        written_size::<{ i64::BITS }>(self.0 as u32 as u128)
    }
}

impl From<i64> for VarInt<i64> {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<VarInt<i64>> for i64 {
    fn from(value: VarInt<i64>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA_I32: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];

    const DATA_I64: &[(i64, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (
            9223372036854775807,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        ),
        (
            -1,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -2147483648,
            &[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -9223372036854775808,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ),
    ];

    #[test]
    fn decode_i32() {
        for &(input, output) in DATA_I32 {
            let mut buffer = bytes::Bytes::from(output);
            assert_eq!(
                decode_generic::<{ i32::BITS }>(&mut buffer).unwrap() as i32,
                input
            );
        }
    }

    #[test]
    fn encode_i32() {
        for &(input, output) in DATA_I32 {
            let mut buffer = bytes::BytesMut::new();
            encode_generic(&mut buffer, input as u32 as u128);
            assert_eq!(buffer.chunk(), output);
        }
    }

    #[test]
    fn decode_i64() {
        for &(input, output) in DATA_I64 {
            let mut buffer = bytes::Bytes::from(output);
            assert_eq!(
                decode_generic::<{ i64::BITS }>(&mut buffer).unwrap() as i64,
                input
            );
        }
    }

    #[test]
    fn encode_i64() {
        for &(input, output) in DATA_I64 {
            let mut buffer = bytes::BytesMut::new();
            encode_generic(&mut buffer, input as u64 as u128);
            assert_eq!(buffer.chunk(), output);
        }
    }

    #[test]
    fn written_size_i64() {
        for &(input, output) in DATA_I64 {
            assert_eq!(
                written_size::<{ i64::BITS }>(input as u64 as u128),
                output.len()
            );
        }
    }
}
