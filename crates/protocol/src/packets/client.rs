pub use configuration::*;
pub use handshake::*;
pub use login::*;
pub use play::*;
pub use status::*;

use super::*;

mod configuration;
mod handshake;
mod login;
mod play;
mod status;

enum_packets!(
    ClientHandshakePacket {
        0x00 = Handshake,
    }
);

enum_packets!(
    ClientStatusPacket {
        0x00 = Request,
        0x01 = Ping,
    }
);

enum_packets!(
    ClientLoginPacket {
        0x00 = LoginStart,
        0x01 = EncryptionResponse,
        0x02 = LoginPluginResponse,
        0x03 = LoginAck,
    }
);

enum_packets!(
    ClientConfigurationPacket {
        0x00 = PluginMessageConfiguration,
        0x01 = FinishConfiguration,
        0x02 = KeepAlive,
        0x03 = Pong,
        0x04 = ResourcePack,
    }
);
