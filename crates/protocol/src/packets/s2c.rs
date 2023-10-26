//! Client-bound packets

use rjacraft_macro::ProtocolType;

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
    RegistryData {
        todo: RemainingBytes<{ 1 << 20 }>, // TODO
    },

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
pub struct JoinGameDeathInfo {
    pub dimension_name: Identifier,
    pub position: Primitive<u64>,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt)]
pub enum PlayPacket {
    #[variant(0x25)]
    KeepAlive { id: Primitive<i64> },

    #[variant(0x2A)]
    JoinGame {
        entity_id: Primitive<i32>,
        is_hardcore: Primitive<bool>,
        dimensions: LenVec<Identifier>,
        max_players: VarInt,
        view_distance: VarInt,
        simulation_distance: VarInt,
        reduced_debug_info: Primitive<bool>,
        enable_respawn_screen: Primitive<bool>,
        dimension_type: Identifier,
        dimension_name: Identifier,
        hashed_seed: Primitive<i64>,
        gamemde: Primitive<u8>,
        previous_gamemode: Primitive<i8>,
        is_debug: Primitive<bool>,
        is_flat: Primitive<bool>,
        death_info: BoolOption<JoinGameDeathInfo>,
        portal_cooldown: VarInt,
    },
}
