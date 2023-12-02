use super::VarInt;
use crate::{error, ProtocolType};

#[derive(Debug)]
pub enum BlockEvent {
    ChestUsers(u8),
}

impl ProtocolType for BlockEvent {
    type DecodeError = error::Infallible;
    type EncodeError = error::Infallible;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        match self {
            Self::ChestUsers(n) => {
                buffer.put_u8(1);
                buffer.put_u8(*n);
                // todo registry reference
                VarInt(177i32).encode(buffer)?;
            }
        }

        Ok(())
    }
}
