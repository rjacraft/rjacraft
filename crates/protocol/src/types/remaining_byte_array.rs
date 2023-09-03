//! A byte array that takes everything that's left in the packet

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

/// Put this as your last field or you'll always get EOFs
#[derive(Debug, Clone)]
pub struct RemainingByteArray(pub bytes::Bytes);

impl ProtocolType for RemainingByteArray {
    type DecodeError = error::Infallible;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        Ok(Self(bytes::Bytes::copy_from_slice(buffer.chunk())))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put(&self.0[..]);

        Ok(())
    }
}

impl From<bytes::Bytes> for RemainingByteArray {
    fn from(value: bytes::Bytes) -> Self {
        Self(value)
    }
}

impl From<RemainingByteArray> for bytes::Bytes {
    fn from(value: RemainingByteArray) -> Self {
        value.0
    }
}
