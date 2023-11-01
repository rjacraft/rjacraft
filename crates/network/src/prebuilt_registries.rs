use rjacraft_macro::*;
use rjacraft_protocol::packets::s2c::{registry::*, RegistryData};
use valence_nbt::*;

/// - `minecraft:chat_type`: Vanilla `minecraft:chat`
/// - `minecraft:damage_type`: Vanilla defaults
/// - `minecraft:dimension_type`: Vanilla overworld
/// - `minecraft:worldgen/biome`: Vanilla plains
pub fn simple() -> RegistryData {
    RegistryData {
        chat_type: Registry {
            r#type: id!("chat_type"),
            value: vec![Element {
                element: compound! {
                    "chat" => compound! {
                        "parameters" => List::String(vec!["sender".into(), "content".into()]),
                        "translation_key" => "chat.type.text",
                    },
                    "narration" => compound! {
                        "parameters" => List::String(vec!["sender".into(), "content".into()]),
                        "translation_key" => "chat.type.narrate",
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
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "arrow",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 0,
                    name: id!("arrow"),
                },
                Element {
                    element: compound! {
                        "death_message_type" => "intentional_game_design",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "badRespawnPoint",
                        "scaling" => "always",
                    },
                    id: 1,
                    name: id!("bad_respawn_point"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "cactus",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 2,
                    name: id!("cactus"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "cramming",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 3,
                    name: id!("cramming"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "dragonBreath",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 4,
                    name: id!("dragon_breath"),
                },
                Element {
                    element: compound! {
                        "effects" => "drowning",
                        "exhaustion" => 0_f32,
                        "message_id" => "drown",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 5,
                    name: id!("drown"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "dryout",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 6,
                    name: id!("dry_out"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "explosion",
                        "scaling" => "always",
                    },
                    id: 7,
                    name: id!("explosion"),
                },
                Element {
                    element: compound! {
                        "death_message_type" => "fall_variants",
                        "exhaustion" => 0_f32,
                        "message_id" => "fall",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 8,
                    name: id!("fall"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "anvil",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 9,
                    name: id!("falling_anvil"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "fallingBlock",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 10,
                    name: id!("falling_block"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "fallingStalactite",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 11,
                    name: id!("falling_stalactite"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "fireball",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 12,
                    name: id!("fireball"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "fireworks",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 13,
                    name: id!("fireworks"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "flyIntoWall",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 14,
                    name: id!("fly_into_wall"),
                },
                Element {
                    element: compound! {
                        "effects" => "freezing",
                        "exhaustion" => 0_f32,
                        "message_id" => "freeze",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 15,
                    name: id!("freeze"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "generic",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 16,
                    name: id!("generic"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "genericKill",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 17,
                    name: id!("generic_kill"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "hotFloor",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 18,
                    name: id!("hot_floor"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "inFire",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 19,
                    name: id!("in_fire"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "inWall",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 20,
                    name: id!("in_wall"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "indirectMagic",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 21,
                    name: id!("indirect_magic"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "lava",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 22,
                    name: id!("lava"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "lightningBolt",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 23,
                    name: id!("lightning_bolt"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "magic",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 24,
                    name: id!("magic"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "mob",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 25,
                    name: id!("mob_attack"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "mob",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 26,
                    name: id!("mob_attack_no_aggro"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "mob",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 27,
                    name: id!("mob_projectile"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0_f32,
                        "message_id" => "onFire",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 28,
                    name: id!("on_fire"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "outOfWorld",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 29,
                    name: id!("out_of_world"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "outsideBorder",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 30,
                    name: id!("outside_border"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "player",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 31,
                    name: id!("player_attack"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "explosion.player",
                        "scaling" => "always",
                    },
                    id: 32,
                    name: id!("player_explosion"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "sonic_boom",
                        "scaling" => "always",
                    },
                    id: 33,
                    name: id!("sonic_boom"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "stalagmite",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 34,
                    name: id!("stalagmite"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "starve",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 35,
                    name: id!("starve"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "sting",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 36,
                    name: id!("sting"),
                },
                Element {
                    element: compound! {
                        "effects" => "poking",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "sweetBerryBush",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 37,
                    name: id!("sweet_berry_bush"),
                },
                Element {
                    element: compound! {
                        "effects" => "thorns",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "thorns",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 38,
                    name: id!("thorns"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "thrown",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 39,
                    name: id!("thrown"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "trident",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 40,
                    name: id!("trident"),
                },
                Element {
                    element: compound! {
                        "effects" => "burning",
                        "exhaustion" => 0.1_f32,
                        "message_id" => "onFire",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 41,
                    name: id!("unattributed_fireball"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0_f32,
                        "message_id" => "wither",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 42,
                    name: id!("wither"),
                },
                Element {
                    element: compound! {
                        "exhaustion" => 0.1_f32,
                        "message_id" => "witherSkull",
                        "scaling" => "when_caused_by_living_non_player",
                    },
                    id: 43,
                    name: id!("wither_skull"),
                },
            ],
        },
        dimension_type: Registry {
            r#type: id!("dimension_type"),
            value: vec![Element {
                element: compound! {
                    "ambient_light" => 0_f32,
                    "bed_works" => 1_i8,
                    "coordinate_scale" => 1_f64,
                    "effects" => "minecraft:overworld",
                    "has_ceiling" => 0_i8,
                    "has_raids" => 1_i8,
                    "has_skylight" => 1_i8,
                    "height" => 256_i32,
                    "infiniburn" => "#minecraft:infiniburn_overworld",
                    "logical_height" => 256_i32,
                    "min_y" => 0_i32,
                    "monster_spawn_block_light_limit" => 0_i32,
                    "monster_spawn_light_level" => compound! {
                        "type" => "minecraft:uniform",
                        "value" => compound! {
                            "max_inclusive" => 7_i32,
                            "min_inclusive" => 0_i32,
                        },
                    },
                    "natural" => 1_i8,
                    "piglin_safe" => 0_i8,
                    "respawn_anchor_works" => 0_i8,
                    "ultrawarm" => 0_i8,
                },
                id: 0,
                name: id!("overworld"),
            }],
        },
        biome: Registry {
            r#type: id!("worldgen/biome"),
            value: vec![Element {
                element: compound! {
                    "downfall" => 0.4_f32,
                    "effects" => compound! {
                        "fog_color" => 12638463_i32,
                        "mood_sound" => compound! {
                            "block_search_extent" => 8_i32,
                            "offset" => 2_f32,
                            "sound" => "minecraft:ambient.cave",
                            "tick_delay" => 6000_i32,
                        },
                        "sky_color" => 7907327_i32,
                        "water_color" => 4159204_i32,
                        "water_fog_color" => 329011_i32,
                    },
                    "has_precipitation" => 1_i8,
                    "temperature" => 0.8_f32,
                },
                id: 0,
                name: id!("plains"),
            }],
        },
    }
}
