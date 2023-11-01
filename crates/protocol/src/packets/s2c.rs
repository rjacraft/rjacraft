//! Client-bound packets

use rjacraft_macro::ProtocolType;
use serde::{Deserialize, Serialize};

use crate::{types::*, ProtocolType};

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
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
#[variant(VarInt)]
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
        uuid: ::uuid::Uuid,
        username: LenString<16>,
        /// See [Mojang's API](https://wiki.vg/Mojang_API#UUID_to_Profile_and_Skin.2FCape) for the
        /// meaning of these
        properties: LenVec<ProfileProperty>,
    },

    #[variant(0x03)]
    SetCompression { threshold: VarInt },

    #[variant(0x04)]
    PluginRequest {
        message_id: VarInt,
        channel: Identifier,
        data: RemainingBytes<{ 1 << 20 }>,
    },
}

pub mod registry {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Element<T> {
        pub element: T,
        pub id: i32,
        pub name: Identifier,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Registry<T> {
        pub r#type: Identifier,
        pub value: Vec<Element<T>>,
    }
}

/// In theory, there could be more registries than this, but we don't care at all.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryData {
    #[serde(rename = "minecraft:chat_type")]
    pub chat_type: registry::Registry<valence_nbt::Compound>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_type: registry::Registry<valence_nbt::Compound>,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: registry::Registry<valence_nbt::Compound>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: registry::Registry<valence_nbt::Compound>,
}

#[derive(Debug, Clone, ProtocolType)]
pub struct Tag {
    pub name: Identifier,
    pub entries: LenVec<VarInt>,
}

#[derive(Debug, Clone, ProtocolType)]
pub struct TagType {
    pub name: Identifier,
    pub tags: LenVec<Tag>,
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
    Disconnect { reason: JsonChat },

    #[variant(0x02)]
    FinishConfiguration,

    #[variant(0x03)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x04)]
    Ping { payload: Primitive<i64> },

    #[variant(0x05)]
    RegistryData(Nbt<RegistryData>),

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
#[variant(Primitive::<u8>)]
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
pub struct BlockEntity {
    // todo
}

#[derive(Debug, Clone, ProtocolType)]
pub struct WorldPos {
    pub dimension_name: Identifier,
    pub position: Position,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(Primitive::<u8>)] // fixme
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

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayPacket {
    #[variant(0x0C)]
    WorldDifficulty {
        difficulty: Difficulty,
        locked: Primitive<bool>,
    },

    /// Not tested
    #[variant(0x0F)]
    ChunkBiomes(LenVec<ChunkBiomeData>),

    /// The order of X and Z is flipped, because Minecraft reads this as a weird long bitfield.
    /// Despite this, the order in [`PlayPacket::ChunkData`] still remains normal.
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
        interp_time: VarInt, // todo varlong
        portal_boundary: VarInt,
        warning_distance: VarInt,
        warning_tme: VarInt,
    },

    #[variant(0x25)]
    NetKeepAlive { id: Primitive<i64> },

    #[variant(0x26)]
    ChunkData {
        chunk_x: Primitive<i32>,
        chunk_z: Primitive<i32>,
        heightmaps: Nbt<net_chunk::ColumnHeightmaps>,
        palettes: net_chunk::ColumnPalettes<net_chunk::FullPalettes>,
        block_entities: LenVec<BlockEntity>,
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
        max_players: VarInt,
        // not to be confused with view distance, which is how far the client chooses to render
        load_distance: VarInt,
        simulation_distance: VarInt,
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
        portal_cooldown: VarInt,
    },

    #[variant(0x37)]
    PlayerAbilities {
        flags: Primitive<u8>, // todo bitfield
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
        flags: Primitive<u8>,
        id: VarInt,
    },

    #[variant(0x4F)]
    PlayerHotbarSlot(Primitive<i8>),

    #[variant(0x51)]
    ChunkCenter { chunk_x: VarInt, chunk_z: VarInt },

    #[variant(0x53)]
    WorldRespawn {
        position: Position,
        pitch: Primitive<f32>,
    },

    #[variant(0x59)]
    PlayerExperience {
        fill_bar: Primitive<f32>,
        exp: VarInt,
        level: VarInt,
    },

    #[variant(0x5A)]
    PlayerHealth {
        health: Primitive<f32>,
        hunger: VarInt,
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
