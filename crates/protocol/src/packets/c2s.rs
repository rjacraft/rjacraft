//! Server-bound packets

use bitfield_struct::bitfield;
use rjacraft_macro::ProtocolType;

use crate::{error, types::*, ProtocolType};

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum NextState {
    #[variant(1)]
    Status,
    #[variant(2)]
    Login,
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum HandshakePacket {
    #[variant(0x00)]
    Handshake {
        protocol_version: crate::ProtocolVersion,
        server_address: LenString<255>,
        server_port: Primitive<u16>,
        next_state: NextState,
    },
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum StatusPacket {
    #[variant(0x00)]
    Request,

    #[variant(0x01)]
    Ping { payload: Primitive<i64> },
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum LoginPacket {
    #[variant(0x00)]
    LoginStart { username: LenString<16>, uuid: Uuid },

    #[variant(0x01)]
    EncryptionResponse {
        shared_secret: LenVec<u8>,
        verify_token: LenVec<u8>,
    },

    #[variant(0x02)]
    LoginPluginResponse {
        message_id: VarInt<i32>,
        successful: Primitive<bool>,
        data: RemainingBytes<{ 1 << 20 }>,
    },

    #[variant(0x03)]
    ToConfig,
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum ConfigPacket {
    #[variant(0x00)]
    PluginMessage {
        channel: Identifier,
        data: RemainingBytes<{ 1 << 20 }>,
    },

    #[variant(0x01)]
    ToPlay,

    #[variant(0x02)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x03)]
    Pong { payload: Primitive<i64> },

    #[variant(0x04)]
    ResourcePack { result: VarInt<i32> },
}

#[derive(Debug, ProtocolType)]
pub struct ArgumentSignature {
    pub argument: LenString<16>,
    pub signature: [u8; 256],
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum ClientCommand {
    #[variant(0)]
    Respawn,
    #[variant(1)]
    StatsRequest,
}

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct SkinParts {
    pub cape: bool,
    pub jacket: bool,
    pub left_sleeve: bool,
    pub right_sleeve: bool,
    pub left_pants: bool,
    pub right_pants: bool,
    pub hat: bool,
    __: bool,
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum ChatMode {
    #[variant(0)]
    Enabled,
    #[variant(1)]
    Commands,
    #[variant(2)]
    Hidden,
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum HandAbs {
    #[variant(0)]
    Left,
    #[variant(1)]
    Right,
}

#[derive(Debug)]
pub enum OptionalSlot {
    Some(u16),
    None,
}

impl ProtocolType for OptionalSlot {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        let Primitive(value) = Primitive::<i16>::decode(buffer)?;

        if value == -999 {
            Ok(Self::None)
        } else {
            Ok(Self::Some(value as u16))
        }
    }

    fn encode(&self, _buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        todo!()
    }
}

impl From<OptionalSlot> for Option<u16> {
    fn from(value: OptionalSlot) -> Self {
        match value {
            OptionalSlot::Some(x) => Some(x),
            OptionalSlot::None => None,
        }
    }
}

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct PlayerAbilities {
    __: bool,
    pub flying: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum HandRel {
    #[variant(0)]
    Main,
    #[variant(1)]
    Offhand,
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum InteractKind {
    #[variant(0)]
    Interact { hand: HandRel },
    #[variant(1)]
    Attack,
    #[variant(2)]
    InteractAt {
        x: Primitive<f32>,
        y: Primitive<f32>,
        z: Primitive<f32>,
        hand: HandRel,
    },
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum PlayerAction {
    #[variant(0)]
    DigStart,
    #[variant(1)]
    DigCancel,
    #[variant(2)]
    DigFinish,
    #[variant(3)]
    DropStack,
    #[variant(4)]
    DropItem,
    #[variant(5)]
    ItemUpdate,
    #[variant(6)]
    SwapHands,
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(Primitive<i8>)]
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

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct PlayerInputFlags {
    pub jump: bool,
    pub dismount: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
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

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum RecipeBook {
    #[variant(0)]
    Crafting,
    #[variant(1)]
    Furnace,
    #[variant(2)]
    BlastFurnace,
    #[variant(3)]
    Smoker,
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum AdvancementCommand {
    #[variant(0)]
    OpenTab(Identifier),
    #[variant(1)]
    CloseScreen,
}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum PlayPacket {
    #[variant(0x00)]
    PlayerTeleportConfirm { id: VarInt<i32> },

    #[variant(0x04)]
    ChatCommand {
        command: LenString<256>,
        timestamp: Primitive<i64>,
        salt: Primitive<i64>,
        signatures: LenVec<ArgumentSignature>,
        message_count: VarInt<i32>,
        acknowledged: BitVec<u64>,
    },

    #[variant(0x05)]
    ChatMessage {
        message: LenString<256>,
        timestamp: Primitive<i64>,
        salt: Primitive<i64>,
        signature: BoolOption<[u8; 256]>,
        message_count: VarInt<i32>,
        acknowledged: BitVec<u64>,
    },

    #[variant(0x08)]
    ClientCommand(ClientCommand),

    #[variant(0x09)]
    ClientInfo {
        locale: LenString<16>,
        view_distance: Primitive<i8>,
        chat_mode: ChatMode,
        chat_colors: Primitive<bool>,
        skin_parts: SkinParts,
        main_hand: HandAbs,
        text_filtering: Primitive<bool>,
        show_on_listings: Primitive<bool>,
    },

    #[variant(0x0C)]
    ContainerButton {
        sync_id: Primitive<u8>,
        button_id: Primitive<u8>,
    },

    #[variant(0x0D)]
    ContainerClick {
        sync_id: Primitive<u8>,
        state_id: VarInt<i32>,
        slot: OptionalSlot,
        button: Primitive<u8>,
        mode: VarInt<i32>,
        new_slots: LenVec<(Primitive<u16>, BoolOption<ItemStackProto>)>,
        carried_item: BoolOption<ItemStackProto>,
    },

    #[variant(0x0E)]
    ContainerClose { sync_id: Primitive<u8> },

    #[variant(0x12)]
    InteractEntity {
        entity: VarInt<i32>,
        kind: InteractKind,
        sneaking: Primitive<bool>,
    },

    #[variant(0x14)]
    NetKeepAlive { id: Primitive<i64> },

    #[variant(0x16)]
    PlayerPosOng {
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x17)]
    PlayerPosRotOng {
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        yaw: Primitive<f32>,
        pitch: Primitive<f32>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x18)]
    PlayerRotOng {
        yaw: Primitive<f32>,
        pitch: Primitive<f32>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x19)]
    PlayerOnGround(Primitive<bool>),

    #[variant(0x1F)]
    PlayerAbilties(PlayerAbilities),

    #[variant(0x20)]
    PlayerAction {
        action: PlayerAction,
        position: BlockPos,
        face: Face,
        sequence: VarInt<i32>,
    },

    #[variant(0x21)]
    PlayerCommand {
        player: VarInt<i32>,
        action: PlayerCommand,
        extra: VarInt<i32>,
    },

    #[variant(0x22)]
    PlayerInput {
        sideways: Primitive<f32>,
        forward: Primitive<f32>,
        flags: PlayerInputFlags,
    },

    #[variant(0x24)]
    RecipeBookState {
        book: RecipeBook,
        open: Primitive<bool>,
        filter: Primitive<bool>,
    },

    #[variant(0x28)]
    AdvancementCommand(AdvancementCommand),

    #[variant(0x2B)]
    PlayerHotbarSlot(Primitive<u8>),

    #[variant(0x2E)]
    PlayerInventorySlot {
        slot: Primitive<u16>,
        stack: BoolOption<ItemStackProto>,
    },

    #[variant(0x32)]
    PlayerSwingArm(HandRel),

    #[variant(0x34)]
    InteractBlock {
        hand: HandRel,
        block_pos: BlockPos,
        block_face: Face,
        cursor_x: Primitive<f32>,
        cursor_y: Primitive<f32>,
        cursor_z: Primitive<f32>,
        head_buried: Primitive<bool>,
        sequence: VarInt<i32>,
    },

    #[variant(0x35)]
    InteractItem {
        hand: HandRel,
        sequence: VarInt<i32>,
    },
}
