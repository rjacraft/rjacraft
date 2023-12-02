use rjacraft_macro::*;
use rjacraft_protocol::{
    packets::s2c,
    types::{registry::*, *},
    ProtocolType,
};

/// - `minecraft:chat_type`: Vanilla `minecraft:chat`
/// - `minecraft:damage_type`: Vanilla defaults
/// - `minecraft:dimension_type`: Vanilla overworld
/// - `minecraft:worldgen/biome`: Vanilla plains
pub fn simple() -> Encoded<Nbt<CustomRegistries>> {
    let regs = CustomRegistries {
        chat_type: Registry {
            r#type: id!("chat_type"),
            value: vec![Element {
                element: ChatType {
                    chat: ChatTypeText {
                        parameters: vec!["sender".into(), "content".into()],
                        translation_key: "chat.type.text".into(),
                        style: None,
                    },
                    narration: ChatTypeNarration {
                        parameters: vec!["sender".into(), "content".into()],
                        translation_key: "chat.type.narrate".into(),
                    },
                },
                id: 0,
                name: id!("chat"),
            }],
        },
        damage_type: Registry {
            r#type: id!("damage_type"),
            value: vec![
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "arrow".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 0,
                    name: id!("arrow"),
                },
                Element {
                    element: DamageType {
                        effects: None,
                        death_message_type: Some(DeathMessageType::IntentionalGameDesign),
                        exhaustion: 0.1,
                        message_id: "badRespawnPoint".into(),
                        scaling: DamageScaling::Always,
                    },
                    id: 1,
                    name: id!("bad_respawn_point"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "cactus".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 2,
                    name: id!("cactus"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "cramming".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 3,
                    name: id!("cramming"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "dragonBreath".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 4,
                    name: id!("dragon_breath"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Drowning),
                        exhaustion: 0.0,
                        message_id: "drown".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 5,
                    name: id!("drown"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "dryout".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 6,
                    name: id!("dry_out"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "explosion".into(),
                        scaling: DamageScaling::Always,
                    },
                    id: 7,
                    name: id!("explosion"),
                },
                Element {
                    element: DamageType {
                        death_message_type: Some(DeathMessageType::FallVariants),
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "fall".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 8,
                    name: id!("fall"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "anvil".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 9,
                    name: id!("falling_anvil"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "fallingBlock".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 10,
                    name: id!("falling_block"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "fallingStalactite".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 11,
                    name: id!("falling_stalactite"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.1,
                        message_id: "fireball".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 12,
                    name: id!("fireball"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "fireworks".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 13,
                    name: id!("fireworks"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "flyIntoWall".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 14,
                    name: id!("fly_into_wall"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Freezing),
                        exhaustion: 0.0,
                        message_id: "freeze".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 15,
                    name: id!("freeze"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "generic".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 16,
                    name: id!("generic"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "genericKill".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 17,
                    name: id!("generic_kill"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.1,
                        message_id: "hotFloor".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 18,
                    name: id!("hot_floor"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.1,
                        message_id: "inFire".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 19,
                    name: id!("in_fire"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "inWall".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 20,
                    name: id!("in_wall"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "indirectMagic".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 21,
                    name: id!("indirect_magic"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.1,
                        message_id: "lava".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 22,
                    name: id!("lava"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "lightningBolt".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 23,
                    name: id!("lightning_bolt"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "magic".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 24,
                    name: id!("magic"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "mob".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 25,
                    name: id!("mob_attack"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "mob".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 26,
                    name: id!("mob_attack_no_aggro"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "mob".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 27,
                    name: id!("mob_projectile"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.0,
                        message_id: "onFire".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 28,
                    name: id!("on_fire"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "outOfWorld".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 29,
                    name: id!("out_of_world"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "outsideBorder".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 30,
                    name: id!("outside_border"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "player".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 31,
                    name: id!("player_attack"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "explosion.player".into(),
                        scaling: DamageScaling::Always,
                    },
                    id: 32,
                    name: id!("player_explosion"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "sonic_boom".into(),
                        scaling: DamageScaling::Always,
                    },
                    id: 33,
                    name: id!("sonic_boom"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "stalagmite".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 34,
                    name: id!("stalagmite"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "starve".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 35,
                    name: id!("starve"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "sting".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 36,
                    name: id!("sting"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Poking),
                        exhaustion: 0.1,
                        message_id: "sweetBerryBush".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 37,
                    name: id!("sweet_berry_bush"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Thorns),
                        exhaustion: 0.1,
                        message_id: "thorns".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 38,
                    name: id!("thorns"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "thrown".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 39,
                    name: id!("thrown"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "trident".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 40,
                    name: id!("trident"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: Some(DamageEffects::Burning),
                        exhaustion: 0.1,
                        message_id: "onFire".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 41,
                    name: id!("unattributed_fireball"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.0,
                        message_id: "wither".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 42,
                    name: id!("wither"),
                },
                Element {
                    element: DamageType {
                        death_message_type: None,
                        effects: None,
                        exhaustion: 0.1,
                        message_id: "witherSkull".into(),
                        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
                    },
                    id: 43,
                    name: id!("wither_skull"),
                },
            ],
        },
        dimension_type: Registry {
            r#type: id!("dimension_type"),
            value: vec![Element {
                element: DimensionType {
                    fixed_time: None,
                    has_skylight: true,
                    has_ceiling: false,
                    ultrawarm: false,
                    natural: true,
                    coordinate_scale: 1.0,
                    bed_works: true,
                    respawn_anchor_works: false,
                    min_y: 0,
                    height: 256,
                    logical_height: 256,
                    infiniburn: TagKey(id!("infiniburn_overworld")),
                    effects: Some(id!("overworld")),
                    ambient_light: 0.0,
                    piglin_safe: false,
                    has_raids: true,
                    monster_spawn_block_light_limit: 0,
                    monster_spawn_light_level: rjacraft_protocol::types::IntProvider::Uniform {
                        value: provider::Bounds {
                            min_inclusive: 0,
                            max_inclusive: 7,
                        },
                    },
                },
                id: 0,
                name: id!("overworld"),
            }],
        },
        biome: Registry {
            r#type: id!("worldgen/biome"),
            value: vec![Element {
                element: Biome {
                    has_precipitation: true,
                    temperature: 0.8,
                    temperature_modifier: None,
                    downfall: 0.4,
                    effects: BiomeEffects {
                        fog_color: 12638463,
                        water_color: 4159204,
                        water_fog_color: 329011,
                        sky_color: 7907327,
                        foliage_color: None,
                        grass_color: None,
                        grass_color_modifier: None,
                        particle: None,
                        mood_sound: Some(MoodSound {
                            block_search_extent: 8,
                            offset: 2.0,
                            sound: id!("ambient.cave"),
                            tick_delay: 6000,
                        }),
                        ambient_sound: None,
                        additions_sound: None,
                        music: None,
                    },
                },
                id: 0,
                name: id!("plains"),
            }],
        },
    };

    Nbt(regs).to_encoded_expect()
}

pub fn clean_tags() -> Encoded<LenVec<s2c::TagType>> {
    LenVec(vec![
        s2c::TagType {
            name: id!("block"),
            tags: vec![].into(),
        },
        s2c::TagType {
            name: id!("entity_type"),
            tags: vec![].into(),
        },
        s2c::TagType {
            name: id!("fluid"),
            tags: vec![].into(),
        },
        s2c::TagType {
            name: id!("game_event"),
            tags: vec![].into(),
        },
        s2c::TagType {
            name: id!("item"),
            tags: vec![].into(),
        },
    ])
    .to_encoded_expect()
}
