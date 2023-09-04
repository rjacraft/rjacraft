//! Server-bound packets

pub use self::{configuration::*, handshake::*, login::*, play::*, status::*};
use crate::packets::prelude::*;

mod configuration;
mod handshake;
mod login;
mod play;
mod status;

packet_sumtype! {
    HandshakePacket {
        0x00 = Handshake,
    }

    StatusPacket {
        0x00 = Request,
        0x01 = Ping,
    }

    LoginPacket {
        0x00 = LoginStart,
        0x01 = EncryptionResponse,
        0x02 = LoginPluginResponse,
        0x03 = LoginAck,
    }

    ConfigurationPacket {
        0x00 = PluginMessageConfiguration,
        0x01 = FinishConfiguration,
        0x02 = KeepAlive,
        0x03 = Pong,
        0x04 = ResourcePack,
    }

    PlayPacket {}
}
