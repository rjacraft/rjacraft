//! Server-bound packets

use rjacraft_macro::ProtocolType;

use crate::{types::*, ProtocolType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolType)]
#[variant(VarInt)]
pub enum NextState {
    #[variant(1)]
    Status,

    #[variant(2)]
    Login,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum HandshakePacket {
    #[variant(0x00)]
    Handshake {
        protocol_version: crate::ProtocolVersion,
        server_address: LenString<255>,
        server_port: Primitive<u16>,
        next_state: NextState,
    },
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum StatusPacket {
    #[variant(0x00)]
    Request,

    #[variant(0x01)]
    Ping { payload: Primitive<i64> },
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum LoginPacket {
    #[variant(0x00)]
    LoginStart {
        username: LenString<16>,
        uuid: ::uuid::Uuid,
    },

    #[variant(0x01)]
    EncryptionResponse {
        shared_secret: LenVec<u8>,
        verify_token: LenVec<u8>,
    },

    #[variant(0x02)]
    LoginPluginResponse {
        message_id: VarInt,
        successful: Primitive<bool>,
        data: RemainingBytes<{ 1 << 20 }>,
    },

    #[variant(0x03)]
    SuccessAck,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum ConfigurationPacket {
    #[variant(0x00)]
    PluginMessage {
        channel: Identifier,
        data: RemainingBytes<{ 1 << 20 }>,
    },

    #[variant(0x01)]
    FinishConfiguration,

    #[variant(0x02)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x03)]
    Pong { payload: Primitive<i64> },

    #[variant(0x04)]
    ResourcePack { result: VarInt },
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayPacket {
    #[variant(0x14)]
    KeepAlive { id: Primitive<i64> },
}
