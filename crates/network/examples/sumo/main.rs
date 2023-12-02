use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*, ProtocolType};

mod combat;
mod eid;
mod player_entity;

const SPAWN_X: f64 = 7.5;
const SPAWN_Y: f64 = 41.0;
const SPAWN_Z: f64 = 7.5;

#[derive(Resource)]
struct PlatformChunk(
    Encoded<Nbt<net_chunk::ColumnHeightmaps>>,
    Encoded<net_chunk::ColumnPalettes>,
    Encoded<net_chunk::ColumnLight>,
);

#[derive(Resource)]
struct EmptyChunk(
    Encoded<Nbt<net_chunk::ColumnHeightmaps>>,
    Encoded<net_chunk::ColumnPalettes>,
    Encoded<net_chunk::ColumnLight>,
);

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    let platform_chunk = {
        let mut column = chunk::Column::<16>::default();
        column.blockstates[2][8] = [[1; 16]; 16];
        column.sky_light.world = [[[[15; 16]; 16]; 16]; 16];
        let (heightmaps, palettes, light) = chunk::to_network(&column);
        PlatformChunk(
            Nbt(heightmaps).to_encoded_expect(),
            palettes.to_encoded_expect(),
            light.to_encoded_expect(),
        )
    };

    let empty_chunk = {
        let mut column = chunk::Column::<16>::default();
        column.sky_light.world = [[[[15; 16]; 16]; 16]; 16];
        let (heightmaps, palettes, light) = chunk::to_network(&column);
        EmptyChunk(
            Nbt(heightmaps).to_encoded_expect(),
            palettes.to_encoded_expect(),
            light.to_encoded_expect(),
        )
    };

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio runtime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                n2b_system: IntoSystem::into_system(n2b_system(UserSystems {
                    status: status_system,
                    authenticate: auth_system,
                    brand: brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        // .add_systems(PreUpdate, eid::assign_system)
        .add_systems(
            Update,
            (
                eid::assign_system.before(init_play_system),
                init_play_system,
                player_entity::accept_movement_system,
                player_entity::accept_input_system,
                player_entity::broadcast_movement_system.after(player_entity::join_system),
                player_entity::join_system,
                player_entity::leave_system,
                combat::accept_interact_system,
                combat::losing_system,
            ),
        )
        .add_systems(PostUpdate, eid::free_system)
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(prebuilt_registries::clean_tags()))
        .insert_resource(platform_chunk)
        .insert_resource(empty_chunk)
        .insert_resource(eid::EidMap::default())
        .run();
}

fn status_system(_peer: In<Entity>) -> server_status::ServerStatus {
    server_status::ServerStatus {
        version: server_status::Version {
            name: "Snapshot whatever".into(),
            protocol: rjacraft_protocol::SUPPORTED_PROTOCOL,
        },
        players: server_status::Players {
            max: 100,
            online: 0,
            sample: vec![],
        },
        description: text!("Example: " (b "sumo")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

#[derive(Component)]
pub struct Profile {
    pub username: UsernameString,
    pub uuid: Uuid,
}

fn auth_system(
    In((entity, username, uuid)): In<(Entity, UsernameString, Uuid)>,
    mut commands: Commands,
) -> AuthOutcome {
    commands.entity(entity).insert(Profile {
        username: username.clone(),
        uuid,
    });

    AuthOutcome::Success(
        uuid,
        player_info::Profile {
            username,
            properties: vec![].into(),
        },
    )
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn init_play_system(
    eids: Res<eid::EidMap>,
    platform: Res<PlatformChunk>,
    empty: Res<EmptyChunk>,
    players: Query<(Entity, &Play), Added<Play>>,
    mut commands: Commands,
) {
    for (entity, play) in players.iter() {
        play.send(
            s2c::PlayPacket::Login {
                entity_id: eids.eid_of(&entity),
                is_hardcore: false.into(),
                dimensions: vec![id!["overworld"]].into(),
                max_players: 20.into(),
                load_distance: 8.into(),
                simulation_distance: 8.into(),
                reduced_debug_info: false.into(),
                enable_respawn_screen: false.into(),
                dimension_type: id!("overworld"),
                dimension_name: id!("overworld"),
                hashed_seed: 0.into(),
                gamemode: s2c::GameMode::Adventure,
                previous_gamemode: s2c::PreviousGameMode::None,
                is_debug: false.into(),
                is_flat: false.into(),
                died: None.into(),
                portal_cooldown: 0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::WorldRespawn {
                position: BlockPos::new().with_x(0).with_y(0).with_z(0),
                pitch: 0.0.into(),
            }
            .to_encoded_expect(),
        );

        for x in -1..2 {
            for z in -1..2 {
                if x == 0 && z == 0 {
                    play.send(
                        s2c::PlayPacket::ChunkData {
                            chunk_x: x.into(),
                            chunk_z: z.into(),
                            heightmaps: platform.0.clone(),
                            palettes: platform.1.clone(),
                            block_entities: vec![].into(),
                            light: platform.2.clone(),
                        }
                        .to_encoded_expect(),
                    );
                } else {
                    play.send(
                        s2c::PlayPacket::ChunkData {
                            chunk_x: x.into(),
                            chunk_z: z.into(),
                            heightmaps: empty.0.clone(),
                            palettes: empty.1.clone(),
                            block_entities: vec![].into(),
                            light: empty.2.clone(),
                        }
                        .to_encoded_expect(),
                    );
                }
            }
        }

        commands.entity(entity).insert(player_entity::Movement {
            pos: (SPAWN_X, SPAWN_Y, SPAWN_Z),
            body_rot: (0.0, 30.0),
            head_rot: (0.0, 30.0),
            ong: true,
        });
    }
}
