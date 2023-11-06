use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomRegistries {
    #[serde(rename = "minecraft:chat_type")]
    pub chat_type: Registry<ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_type: Registry<DamageType>,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: Registry<DimensionType>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: Registry<Biome>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatTypeChat {
    pub parameters: Vec<String>,
    pub translation_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<chat::Attrs>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatTypeNarration {
    pub translation_key: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatType {
    pub chat: ChatTypeChat,
    pub narration: ChatTypeNarration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathMessageType {
    IntentionalGameDesign,
    FallVariants,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageEffects {
    Burning,
    Drowning,
    Freezing,
    Poking,
    Thorns,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageScaling {
    Always,
    WhenCausedByLivingNonPlayer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DamageType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_message_type: Option<DeathMessageType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<DamageEffects>,
    pub exhaustion: f32,
    pub message_id: String,
    pub scaling: DamageScaling,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DimensionType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_time: Option<i64>,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub ultrawarm: bool,
    pub natural: bool,
    pub coordinate_scale: f64,
    pub bed_works: bool,
    pub respawn_anchor_works: bool,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: TagKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Identifier>,
    pub ambient_light: f32,

    pub piglin_safe: bool,
    pub has_raids: bool,
    pub monster_spawn_light_level: IntProvider,
    pub monster_spawn_block_light_limit: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureModifier {
    Frozen,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrassColorModifier {
    DarkForest,
    Swamp,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticleConfig {
    // todo
    pub options: valence_nbt::Compound,
    pub probability: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoodSound {
    pub sound: Identifier,
    pub tick_delay: i32,
    pub block_search_extent: i32,
    pub offset: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdditionsSound {
    pub sound: Identifier,
    pub tick_chance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Music {
    pub sound: Identifier,
    pub min_delay: i32,
    pub max_delay: i32,
    pub replace_current_music: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BiomeEffects {
    pub fog_color: u32,
    pub water_color: u32,
    pub water_fog_color: u32,
    pub sky_color: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foliage_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color_modifier: Option<GrassColorModifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub particle: Option<ParticleConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_sound: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood_sound: Option<MoodSound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additions_sound: Option<AdditionsSound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music: Option<Music>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Biome {
    pub has_precipitation: bool,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_modifier: Option<TemperatureModifier>,
    pub downfall: f32,
    pub effects: BiomeEffects,
}
