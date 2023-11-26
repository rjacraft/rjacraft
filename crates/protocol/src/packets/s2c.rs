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
pub enum EntityAnimation {
    #[variant(0)]
    SwingMainHand,
    #[variant(2)]
    LeaveBed,
    #[variant(3)]
    SwingOffhand,
    #[variant(4)]
    Crit,
    #[variant(5)]
    MagicCrit,
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
pub struct GlobalPos {
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
#[variant(Primitive<i8>)]
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

#[derive(Debug, Clone, ProtocolType)]
pub struct PlayerProfile {
    pub username: LenString<16>,
    pub properties: LenVec<ProfileProperty>,
}

/// About the fields: There's no way to express this nicely in Rust. The lengths of each present
/// vector have to stay the same for this to be decodable.
#[derive(Debug, Clone)]
pub struct PlayerInfoUpdates {
    pub players: Vec<Uuid>,
    pub profile: Option<Vec<PlayerProfile>>,
    // todo signature
    pub gamemode: Option<Vec<GameMode>>,
    pub listed: Option<Vec<Primitive<bool>>>,
    pub ping: Option<Vec<VarInt<i32>>>,
    pub nickname: Option<Vec<JsonChat>>,
}

impl ProtocolType for PlayerInfoUpdates {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        #[derive(ProtocolType)]
        #[bitfield(u8)]
        struct UsedProperties {
            profile: bool,
            signature: bool,
            gamemode: bool,
            listed: bool,
            ping: bool,
            nickname: bool,
            __: bool,
            __: bool,
        }

        UsedProperties::new()
            .with_profile(self.profile.is_some())
            .with_gamemode(self.gamemode.is_some())
            .with_listed(self.listed.is_some())
            .with_ping(self.ping.is_some())
            .with_nickname(self.nickname.is_some())
            .encode(buffer)?;

        VarInt(self.players.len() as i32).encode(buffer)?;

        for uuid in &self.players {
            uuid.encode(buffer)?;
            // there are no encode errors

            for x in self.profile.iter().flatten() {
                x.encode(buffer).unwrap();
            }

            for x in self.gamemode.iter().flatten() {
                x.encode(buffer).unwrap();
            }

            for x in self.listed.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.ping.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.nickname.iter().flatten() {
                x.encode(buffer).unwrap();
            }
        }

        Ok(())
    }
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

#[derive(Debug, Clone)]
pub enum SoundId {
    Protocol(u32),
    Identifier { id: Identifier, range: Option<f32> },
}

impl ProtocolType for SoundId {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        match self {
            &Self::Protocol(id) => {
                VarInt(id as i32 + 1).encode(buffer)?;
            }
            Self::Identifier { id, range } => {
                VarInt(0).encode(buffer)?;
                id.encode(buffer)?;
                BoolOption(range.map(|x| Primitive(x))).encode(buffer)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum SoundCategory {
    #[variant(0)]
    Master,
    #[variant(1)]
    Music,
    #[variant(2)]
    Record,
    #[variant(3)]
    Weather,
    #[variant(4)]
    Block,
    #[variant(5)]
    Hostile,
    #[variant(6)]
    Neutral,
    #[variant(7)]
    Player,
    #[variant(8)]
    Ambient,
    #[variant(9)]
    Voice,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum PlayPacket {
    #[variant(0x01)]
    EntitySpawn {
        id: VarInt<i32>,
        uuid: Uuid,
        kind: VarInt<i32>,
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        pitch: Primitive<u8>,
        yaw: Primitive<u8>,
        head_yaw: Primitive<u8>,
        int_data: VarInt<i32>,
        velocity: (Primitive<i16>, Primitive<i16>, Primitive<i16>),
    },

    #[variant(0x03)]
    EntitySpawnPlayer {
        id: VarInt<i32>,
        uuid: Uuid,
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        yaw: Primitive<u8>,
        pitch: Primitive<u8>,
    },

    #[variant(0x04)]
    EntityAnimation {
        id: VarInt<i32>,
        animation: EntityAnimation,
    },

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

    #[variant(0x12)]
    ChatCommands {
        nodes: LenVec<command::Node>,
        root: VarInt<i32>,
    },

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

    #[variant(0x1A)]
    EntityDamage {
        taker_id: VarInt<i32>,
        damage_type: VarInt<i32>,
        /// 0 or n + 1
        source_id: VarInt<i32>,
        /// 0 or n + 1
        means_id: VarInt<i32>,
        source_pos: BoolOption<(Primitive<f64>, Primitive<f64>, Primitive<f64>)>,
    },

    /// The order of X and Z is flipped, because Minecraft encodes this as some kind of MSB
    /// bitfield. Despite this, the order in [`PlayPacket::ChunkData`] still remains normal.
    #[variant(0x20)]
    ChunkUnload {
        chunk_z: Primitive<i32>,
        chunk_x: Primitive<i32>,
    },

    #[variant(0x23)]
    EntityDamageTilt {
        id: VarInt<i32>,
        yaw: Primitive<f32>,
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
        entity_id: Primitive<i32>,
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
        died: BoolOption<GlobalPos>,
        portal_cooldown: VarInt<i32>,
    },

    #[variant(0x2D)]
    EntityDposOng {
        id: VarInt<i32>,
        dx: Primitive<i16>,
        dy: Primitive<i16>,
        dz: Primitive<i16>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x2E)]
    EntityDposRotOng {
        id: VarInt<i32>,
        dx: Primitive<i16>,
        dy: Primitive<i16>,
        dz: Primitive<i16>,
        yaw: Primitive<u8>,
        pitch: Primitive<u8>,
        on_ground: Primitive<bool>,
    },

    #[variant(0x2F)]
    EntityRotOng {
        id: VarInt<i32>,
        yaw: Primitive<u8>,
        pitch: Primitive<u8>,
        on_ground: Primitive<bool>,
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

    #[variant(0x3C)]
    ServerPlayerRemove(LenVec<Uuid>),

    #[variant(0x3D)]
    ServerPlayerInfo(PlayerInfoUpdates),

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

    #[variant(0x41)]
    EntityRemove(LenVec<VarInt<i32>>),

    #[variant(0x45)]
    EntityHeadYaw { id: VarInt<i32>, yaw: Primitive<u8> },

    #[variant(0x48)]
    ServerMetadata {
        description: JsonChat,
        favicon: BoolOption<LenVec<u8>>,
        enforces_secure_chat: Primitive<bool>,
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

    #[variant(0x55)]
    EntityData {
        id: VarInt<i32>,
        values: EntityDataValues,
    },

    #[variant(0x57)]
    EntityVelocity {
        id: VarInt<i32>,
        x: Primitive<i16>,
        y: Primitive<i16>,
        z: Primitive<i16>,
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

    #[variant(0x65)]
    SoundPositioned {
        id: SoundId,
        category: SoundCategory,
        x: Primitive<i32>,
        y: Primitive<i32>,
        z: Primitive<i32>,
        volume: Primitive<f32>,
        pitch: Primitive<f32>,
        seed: Primitive<i64>,
    },

    #[variant(0x68)]
    ChatUnsignedMessage {
        content: JsonChat,
        overlay: Primitive<bool>,
    },

    #[variant(0x6C)]
    EntityPosRotOng {
        id: VarInt<i32>,
        x: Primitive<f64>,
        y: Primitive<f64>,
        z: Primitive<f64>,
        yaw: Primitive<u8>,
        pitch: Primitive<u8>,
        on_ground: Primitive<bool>,
    },
}
