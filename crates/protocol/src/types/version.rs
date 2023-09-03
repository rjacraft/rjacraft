//! A protocol version number backed by [`crate::ProtocolVersion`].

use bytes::{Buf, BufMut};

use crate::{error, version::ProtocolVersion, ProtocolType};

impl ProtocolType for ProtocolVersion {
    type DecodeError = super::varint::DecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt(numeric) = super::VarInt::decode(buffer)?;

        Ok(Self::from(numeric))
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        super::VarInt((*self).into()).encode(buffer)
    }
}
