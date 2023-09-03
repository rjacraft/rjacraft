//! Client-bound packets

use configuration::*;
use login::*;
use play::*;
use status::*;

use super::macros;

pub mod configuration;
pub mod login;
pub mod play;
pub mod status;

macros::packet_sumtype! {
    HandshakePacket {}

    StatusPacket {
        0x00 = Response,
        0x01 = Pong,
    }

    LoginPacket {
        0x00 = DisconnectLogin,
        0x01 = EncryptionRequest,
        0x02 = LoginSuccess,
        0x03 = SetCompression,
        0x04 = LoginPluginRequest,
    }

    ConfigurationPacket {
        0x00 = PluginMessage,
        0x01 = Disconnect,
        0x02 = FinishConfiguration,
        0x03 = KeepAlive,
        0x04 = Ping,
        0x05 = RegistryData,
        0x06 = ResourcePack,
        0x07 = FeatureFlags,
        0x08 = UpdateTags,
    }

    PlayPacket {
        0x2A = JoinGame,
    }
}
