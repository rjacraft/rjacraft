use std::io::Cursor;

use super::*;
use crate::io::{Decode, Encode};

packets!(
    Handshake {
        protocol_version VarInt;
        server_address String;
        server_port u16;
        next_state HandshakeState;
    }
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Configuration = 3,
    Play = 4,
}

impl Decode for HandshakeState {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let discriminant = VarInt::decode(buffer)?.0;
        match discriminant {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            _ => Err(anyhow::anyhow!(
                concat!(
                    "invalid discriminant ",
                    stringify!(HandshakeState),
                    " = ",
                    "{}"
                ),
                discriminant
            )),
        }
    }
}

impl Encode for HandshakeState {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let discriminant = match self {
            Self::Handshaking => 0,
            Self::Status => 1,
            Self::Login => 2,
            Self::Configuration => 3,
            Self::Play => 4,
        };
        VarInt(discriminant).encode(buffer)
    }
}
