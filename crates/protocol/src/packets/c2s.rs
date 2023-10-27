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
pub struct ArgumentSignature {
    pub argument: LenString<16>,
    pub signature: [u8; 256],
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum ClientCommand {
    #[variant(0)]
    Respawn,
    #[variant(1)]
    StatsRequest,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum ChatMode {
    #[variant(0)]
    Enabled,
    #[variant(1)]
    Commands,
    #[variant(2)]
    Hidden,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum HandAbs {
    #[variant(0)]
    Left,
    #[variant(1)]
    Right,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum HandRel {
    #[variant(0)]
    Main,
    #[variant(1)]
    Offhand,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayerAction {
    #[variant(0)]
    DigStart,
    #[variant(1)]
    DigCancel,
    #[variant(2)]
    DigEnd,
    #[variant(3)]
    DropStack,
    #[variant(4)]
    DropItem,
    #[variant(5)]
    ItemUpdate,
    #[variant(6)]
    SwapHands,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(Primitive::<i8>)]
pub enum Face {
    #[variant(0)]
    Bottom,
    #[variant(1)]
    Top,
    #[variant(2)]
    North,
    #[variant(3)]
    South,
    #[variant(4)]
    West,
    #[variant(5)]
    East,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayerCommand {
    #[variant(0)]
    SneakDown,
    #[variant(1)]
    SneakUp,
    #[variant(2)]
    LeaveBed,
    #[variant(3)]
    SprintUp,
    #[variant(4)]
    SprintDown,
    #[variant(5)]
    HorseJumpDown,
    #[variant(6)]
    HorseJumpUp,
    #[variant(7)]
    HorseInventory,
    #[variant(8)]
    Elytra,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum AdvancementCommand {
    #[variant(0)]
    OpenTab(Identifier),
    #[variant(1)]
    CloseScreen,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayPacket {
    #[variant(0x00)]
    ConfirmTeleport { id: VarInt },

    #[variant(0x04)]
    ChatCommand {
        command: LenString<256>,
        timestamp: Primitive<i64>,
        salt: Primitive<i64>,
        signatures: LenVec<ArgumentSignature>,
        message_count: VarInt,
        acknowledged: RemainingBytes<{ usize::MAX }>, // not sure what this is about
    },

    #[variant(0x05)]
    ChatMessage {
        message: LenString<256>,
        timestamp: Primitive<i64>,
        salt: Primitive<i64>,
        signature: BoolOption<[u8; 256]>,
        message_count: VarInt,
        acknowledged: RemainingBytes<{ usize::MAX }>, // not sure what this is about
    },

    #[variant(0x08)]
    ClientCommand(ClientCommand),

    #[variant(0x09)]
    ClientInfo {
        locale: LenString<16>,
        view_distance: Primitive<i8>,
        chat_mode: ChatMode,
        chat_colors: Primitive<bool>,
        skin_parts: Primitive<u8>, // todo bitfield
        main_hand: HandAbs,
        text_filtering: Primitive<bool>,
        show_on_listings: Primitive<bool>,
    },

    #[variant(0x14)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x16)]
    Position {
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x17)]
    PositionRotation {
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        yaw: Primitive<f32>,
        pitch: Primitive<f32>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x18)]
    Rotation {
        yaw: Primitive<f32>,
        pitch: Primitive<f32>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x19)]
    OnGround(Primitive<bool>),

    #[variant(0x20)]
    PlayerAction {
        action: PlayerAction,
        position: Position,
        face: Face,
        sequence: VarInt,
    },

    #[variant(0x21)]
    PlayerCommand {
        player: VarInt,
        action: PlayerCommand,
        extra: VarInt,
    },

    #[variant(0x22)]
    PlayerInput {
        sideways: Primitive<f32>,
        forward: Primitive<f32>,
        flags: Primitive<u8>, // todo bitfield
    },

    #[variant(0x28)]
    AdvancementCommand(AdvancementCommand),

    #[variant(0x2B)]
    SwitchSlots(Primitive<u8>),

    #[variant(0x32)]
    SwingArm(HandRel),
}
