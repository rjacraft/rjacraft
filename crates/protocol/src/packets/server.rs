pub use configuration::*;
pub use login::*;
pub use play::*;
pub use status::*;

use super::*;

pub mod configuration;
pub mod login;
pub mod play;
pub mod status;

enum_packets!(
    ServerStatusPacket {
        0x00 = Response,
        0x01 = Pong,
    }
);

enum_packets!(
    ServerLoginPacket {
        0x00 = DisconnectLogin,
        0x01 = EncryptionRequest,
        0x02 = LoginSuccess,
        0x03 = SetCompression,
        0x04 = LoginPluginRequest,
    }
);

enum_packets!(
    ServerConfigurationPacket {
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
);

enum_packets!(
    ServerPlayPacket {
        0x2A = JoinGame,
    }
);
