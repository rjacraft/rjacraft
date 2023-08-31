use std::io::Cursor;

use crate::io::Decoder;
use crate::Encoder;

use super::*;

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

impl Decoder for HandshakeState {
    fn decode(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let discriminant = VarInt::decode(buffer)?.0;
        match discriminant {
            1 => Ok(HandshakeState::Status),
            2 => Ok(HandshakeState::Login),
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

impl Encoder for HandshakeState {
    fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let discriminant = match self {
            HandshakeState::Handshaking => 0,
            HandshakeState::Status => 1,
            HandshakeState::Login => 2,
            HandshakeState::Configuration => 3,
            HandshakeState::Play => 4,
        };
        VarInt(discriminant).encode(buffer)
    }
}
