//! Client-bound packets

use bitfield_struct::bitfield;
use rjacraft_macro::ProtocolType;

use crate::{error, types::*, ProtocolType};

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum StatusPacket {
    #[variant(0x00)]
    Response(JsonString<{ 1 << 18 }, ServerStatus>),

    #[variant(0x01)]
    Pong { payload: Primitive<i64> },
}

#[derive(Debug, Clone, ProtocolType)]
pub struct ProfileProperty {
    pub name: LenString<{ 1 << 15 }>,
    pub value: LenString<{ 1 << 15 }>,
    pub signature: BoolOption<LenString<{ 1 << 15 }>>,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum LoginPacket {
    #[variant(0x00)]
    Disconnect { reason: JsonChat },

    #[variant(0x01)]
    EncryptionRequest {
        server_id: LenString<20>,
        public_key: LenVec<u8>,
        verify_token: LenVec<u8>,
    },

    #[variant(0x02)]
    Success {
        uuid: Uuid,
        username: LenString<16>,
        /// See [Mojang's API](https://wiki.vg/Mojang_API#UUID_to_Profile_and_Skin.2FCape) for the
        /// meaning of these
        properties: LenVec<ProfileProperty>,
    },

    #[variant(0x03)]
    SetCompression { threshold: VarInt<i32> },

    #[variant(0x04)]
    PluginRequest {
        message_id: VarInt<i32>,
        channel: Identifier,
        data: RemainingBytes<{ 1 << 20 }>,
    },
}

#[derive(Debug, Clone, ProtocolType)]
pub struct Tag {
    pub name: Identifier,
    pub entries: LenVec<VarInt<i32>>,
}

#[derive(Debug, Clone, ProtocolType)]
pub struct TagType {
    pub name: Identifier,
    pub tags: LenVec<Tag>,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum ConfigurationPacket {
    #[variant(0x00)]
    PluginMessage {
        channel: Identifier,
        data: RemainingBytes<{ 1 << 20 }>,
    },

    #[variant(0x01)]
    Disconnect { reason: JsonChat },

    #[variant(0x02)]
    FinishConfiguration,

    #[variant(0x03)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x04)]
    Ping { payload: Primitive<i64> },

    #[variant(0x05)]
    RegistryData(Nbt<CustomRegistries>),

    #[variant(0x06)]
    ResourcePack {
        url: LenString<{ 1 << 15 }>,
        hash: LenString<40>,
        forced: Primitive<bool>,
        prompt_message: BoolOption<JsonChat>,
    },

    #[variant(0x07)]
    FeatureFlags(LenVec<Identifier>),

    #[variant(0x08)]
    UpdateTags(LenVec<TagType>),
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(Primitive<u8>)]
pub enum Difficulty {
    #[variant(0)]
    Peaceful,
    #[variant(1)]
    Easy,
    #[variant(2)]
    Normal,
    #[variant(3)]
    Hard,
}

#[derive(Debug, Clone, ProtocolType)]
pub struct ChunkBiomeData {
    pub chunk_x: Primitive<i32>,
    pub chunk_z: Primitive<i32>,
    palettes: net_chunk::ColumnPalettes<net_chunk::Paletted>,
}

#[derive(Debug, Clone, ProtocolType)]
pub struct WorldPos {
    pub dimension_name: Identifier,
    pub position: BlockPos,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(Primitive<u8>)]
pub enum GameMode {
    #[variant(0)]
    Survival,
    #[variant(1)]
    Creative,
    #[variant(2)]
    Adventure,
    #[variant(3)]
    Spectator,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(Primitive::<i8>)]
pub enum PreviousGameMode {
    #[variant(-1)]
    None,
    #[variant(0)]
    Survival,
    #[variant(1)]
    Creative,
    #[variant(2)]
    Adventure,
    #[variant(3)]
    Spectator,
}

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub instant_break: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct TeleportRelative {
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub pitch: bool,
    pub yaw: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum PlayPacket {
    #[variant(0x08)]
    ChunkBlockEntity {
        position: BlockPos,
        block_entity: BlockEntity,
    },

    #[variant(0x09)]
    ChunkBlockEvent {
        position: BlockPos,
        event: BlockEvent,
    },

    #[variant(0x0C)]
    WorldDifficulty {
        difficulty: Difficulty,
        locked: Primitive<bool>,
    },

    /// Not tested
    #[variant(0x0F)]
    ChunkBiomes(LenVec<ChunkBiomeData>),

    #[variant(0x13)]
    ContainerClose { window_id: Primitive<u8> },

    #[variant(0x14)]
    ContainerSlots {
        sync_id: Primitive<u8>,
        state_id: VarInt<i32>,
        slots: LenVec<ItemStackProto>,
        carried_item: ItemStackProto,
    },

    #[variant(0x15)]
    ContainerProperty {
        sync_id: Primitive<u8>,
        property: Primitive<u16>,
        value: Primitive<i16>,
    },

    #[variant(0x16)]
    ContainerSlot {
        sync_id: Primitive<i8>,
        state_id: VarInt<i32>,
        number: Primitive<i8>,
        slot: ItemStackProto,
    },

    /// The order of X and Z is flipped, because Minecraft encodes this as some kind of MSB
    /// bitfield. Despite this, the order in [`PlayPacket::ChunkData`] still remains normal.
    #[variant(0x20)]
    ChunkUnload {
        chunk_z: Primitive<i32>,
        chunk_x: Primitive<i32>,
    },

    #[variant(0x24)]
    BorderInit {
        center_x: Primitive<f64>,
        center_z: Primitive<f64>,
        side_old: Primitive<f64>,
        side_new: Primitive<f64>,
        interp_time: VarInt<i64>,
        portal_boundary: VarInt<i32>,
        warning_distance: VarInt<i32>,
        warning_tme: VarInt<i32>,
    },

    #[variant(0x25)]
    NetKeepAlive { id: Primitive<i64> },

    #[variant(0x26)]
    ChunkData {
        chunk_x: Primitive<i32>,
        chunk_z: Primitive<i32>,
        heightmaps: Nbt<net_chunk::ColumnHeightmaps>,
        palettes: net_chunk::ColumnPalettes<net_chunk::FullPalettes>,
        block_entities: LenVec<(BlockPosColumn, BlockEntity)>,
        light: net_chunk::ColumnLight,
    },

    /// Not tested
    #[variant(0x29)]
    ChunkLighting {
        chunk_x: Primitive<i32>,
        chunk_z: Primitive<i32>,
        light: net_chunk::ColumnLight,
    },

    #[variant(0x2A)]
    Login {
        entity_id: Primitive<u32>,
        is_hardcore: Primitive<bool>,
        dimensions: LenVec<Identifier>,
        max_players: VarInt<i32>,
        // not to be confused with view distance, which is how far the client chooses to render
        load_distance: VarInt<i32>,
        simulation_distance: VarInt<i32>,
        reduced_debug_info: Primitive<bool>,
        enable_respawn_screen: Primitive<bool>,
        dimension_type: Identifier,
        dimension_name: Identifier,
        hashed_seed: Primitive<i64>,
        gamemode: GameMode,
        previous_gamemode: PreviousGameMode,
        is_debug: Primitive<bool>,
        is_flat: Primitive<bool>,
        died: BoolOption<WorldPos>,
        portal_cooldown: VarInt<i32>,
    },

    #[variant(0x32)]
    ContainerOpen {
        sync_id: VarInt<i32>,
        kind: VarInt<i32>,
        title: JsonChat,
    },

    #[variant(0x37)]
    PlayerAbilities {
        flags: PlayerAbilities,
        flying_speed: Primitive<f32>,
        fov_modifier: Primitive<f32>,
    },

    #[variant(0x3F)]
    PlayerTeleport {
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        yaw: Primitive<f32>,
        pitch: Primitive<f32>,
        relative: TeleportRelative,
        id: VarInt<i32>,
    },

    #[variant(0x4F)]
    PlayerHotbarSlot(Primitive<u8>),

    #[variant(0x51)]
    ChunkCenter {
        chunk_x: VarInt<i32>,
        chunk_z: VarInt<i32>,
    },

    #[variant(0x53)]
    WorldRespawn {
        position: BlockPos,
        pitch: Primitive<f32>,
    },

    #[variant(0x59)]
    PlayerExperience {
        fill_bar: Primitive<f32>,
        exp: VarInt<i32>,
        level: VarInt<i32>,
    },

    #[variant(0x5A)]
    PlayerHealth {
        health: Primitive<f32>,
        hunger: VarInt<i32>,
        saturation: Primitive<f32>,
    },

    #[variant(0x61)]
    WorldTime {
        world_age: Primitive<i64>,
        time: Primitive<i64>,
    },

    #[variant(0x68)]
    ChatSystemMessage {
        content: JsonChat,
        overlay: Primitive<bool>,
    },
}
