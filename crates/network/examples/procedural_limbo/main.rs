use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{packets::s2c, types::*, ProtocolType};

mod chunks;
mod components;
mod generator;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio runtime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                n2b_system: IntoSystem::into_system(n2b_system(NetworkConfig {
                    compress: Some(256),
                    status_system,
                    auth_system,
                    brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(
            Update,
            (
                init_play_system,
                components::movement_system,
                components::send_position_system,
                components::client_info_system,
                chunks::init_filter_system,
                chunks::chunk_send_system
                    .after(init_play_system)
                    .after(components::movement_system)
                    .after(components::client_info_system)
                    .after(chunks::init_filter_system),
            ),
        )
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(prebuilt_registries::clean_tags()))
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
        description: text!("Example: " (b "procedural limbo")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn auth_system(In((_, username, uuid)): In<(Entity, UsernameString, Uuid)>) -> AuthOutcome {
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

fn init_play_system(players: Query<(Entity, &Play), Added<Play>>, mut commands: Commands) {
    for (entity, play) in players.iter() {
        play.send(
            s2c::PlayPacket::Login {
                entity_id: 0.into(),
                is_hardcore: false.into(),
                dimensions: vec![id!["overworld"]].into(),
                max_players: 20.into(),
                load_distance: VarInt(chunks::MAX_RADIUS as i32),
                simulation_distance: VarInt(chunks::MAX_RADIUS as i32),
                reduced_debug_info: false.into(),
                enable_respawn_screen: false.into(),
                dimension_type: id!("overworld"),
                dimension_name: id!("overworld"),
                hashed_seed: 0.into(),
                gamemode: s2c::GameMode::Adventure,
                previous_gamemode: s2c::PreviousGameMode::None,
                is_debug: false.into(),
                is_flat: true.into(), // to make the sky not black at y = 40
                died: None.into(),
                portal_cooldown: 0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::PlayerAbilities {
                flags: s2c::PlayerAbilities::new()
                    .with_flying(true)
                    .with_can_fly(true),
                flying_speed: 0.05.into(),
                fov_modifier: 0.1.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::WorldRespawn {
                position: BlockPos::new().with_x(0).with_y(45).with_z(0),
                pitch: 0.0.into(),
            }
            .to_encoded_expect(),
        );

        commands.entity(entity).insert(components::Position {
            x: 5000.0,
            y: 45.0,
            z: -1000.0,
        });
    }
}
