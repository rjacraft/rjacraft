use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{packets::s2c, types::*};

mod c2s;
mod chunks;
mod generator;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

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
        .add_systems(
            Update,
            (
                init_play_system,
                c2s::brand_system,
                c2s::movement_system,
                c2s::client_info_system,
                chunks::init_filter_system,
                chunks::chunk_send_system
                    .after(init_play_system)
                    .after(c2s::movement_system)
                    .after(c2s::client_info_system)
                    .after(chunks::init_filter_system),
            ),
        )
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(vec![
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
        ]))
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
        description: chat!("Example: " (b "procedural limbo")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn auth_system(In((_, username, uuid)): In<(Entity, String, Uuid)>) -> AuthOutcome {
    AuthOutcome::Success(username, uuid, vec![])
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn init_play_system(players: Query<(Entity, &Play), Added<Play>>, mut commands: Commands) {
    for (entity, play) in players.iter() {
        play.send_packet(&s2c::PlayPacket::Login {
            entity_id: entity.index().into(),
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
            gamemode: s2c::GameMode::Survival,
            previous_gamemode: s2c::PreviousGameMode::None,
            is_debug: false.into(),
            is_flat: true.into(), // to make the sky not black at y = 40
            died: None.into(),
            portal_cooldown: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerAbilities {
            flags: 0b00001110.into(),
            flying_speed: 0.05.into(),
            fov_modifier: 0.1.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerTeleport {
            x: 0.0.into(),
            y: 45.0.into(),
            z: 0.0.into(),
            yaw: 180.0.into(),
            pitch: 0.0.into(),
            flags: 0.into(),
            id: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::WorldRespawn {
            position: Position(0, 45, 0),
            pitch: 0.0.into(),
        })
        .unwrap();

        commands.entity(entity).insert((c2s::Position {
            x: 0.0,
            y: 45.0,
            z: 0.0,
        },));
    }
}
