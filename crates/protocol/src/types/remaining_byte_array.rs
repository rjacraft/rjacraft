//! A byte array that takes everything that's left in the packet up to a certain length

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

/// Put this as your last field or you'll always get EOFs.
/// Guaranteed to be no larger than `MAX_SIZE`.
#[derive(Debug, Clone)]
pub struct RemainingByteArray<const MAX_SIZE: usize>(pub(crate) bytes::Bytes);

impl<const MAX_SIZE: usize> ProtocolType for RemainingByteArray<MAX_SIZE> {
    type DecodeError = error::Overrun<MAX_SIZE>;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() > MAX_SIZE {
            Err(error::Overrun(buffer.remaining()))
        } else {
            Ok(Self(bytes::Bytes::copy_from_slice(buffer.chunk())))
        }
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(&self.0[..]);

        Ok(())
    }
}

impl<const MAX_SIZE: usize> TryFrom<bytes::Bytes> for RemainingByteArray<MAX_SIZE> {
    type Error = error::Overrun<MAX_SIZE>;

    fn try_from(value: bytes::Bytes) -> Result<Self, Self::Error> {
        if value.len() > MAX_SIZE {
            Err(error::Overrun(value.len()))
        } else {
            Ok(Self(value))
        }
    }
}

impl<const MAX_SIZE: usize> From<RemainingByteArray<MAX_SIZE>> for bytes::Bytes {
    fn from(value: RemainingByteArray<MAX_SIZE>) -> Self {
        value.0
    }
}
