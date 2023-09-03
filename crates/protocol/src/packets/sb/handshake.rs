use bytes::{Buf, BufMut};

use crate::packets::prelude::*;
use crate::{error, types, ProtocolType};

packets_struct! {
    Handshake {
        protocol_version: crate::ProtocolVersion;
        server_address: String;
        server_port: Primitive<u16>;
        next_state: NextState;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

// TODO need a macro for this

impl ProtocolType for NextState {
    type DecodeError = error::EnumError<VarInt>;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        let types::VarInt(discriminator) =
            types::VarInt::decode(buffer).map_err(|e| error::EnumError::ReadingDiscriminator(e))?;

        match discriminator {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            _ => Err(error::EnumError::OutOfRange(discriminator.into())),
        }
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let discriminator = match self {
            Self::Status => 1,
            Self::Login => 2,
        };

        VarInt(discriminator).encode(buffer)?;

        Ok(())
    }
}
