use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{
    packets::s2c,
    types::{self, server_status},
};
use tracing::*;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio lifetime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                n2b_system: IntoSystem::into_system(n2b_system(UserSystems {
                    status: status_system,
                    authenticate: auth_system,
                    brand: server_brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(Update, (brand_system, init_play_system, chat_system))
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

#[derive(Component)]
struct Login {
    username: String,
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
        description: chat!("Example: " (b "chat")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn auth_system(
    In((entity, username, uuid)): In<(Entity, String, uuid::Uuid)>,
    mut commands: Commands,
) -> AuthOutcome {
    commands.entity(entity).insert(Login {
        username: username.clone(),
    });

    AuthOutcome::Success(username, uuid, vec![])
}

fn server_brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn brand_system(mut events_in: EventReader<C2sPacket<packet::ClientBrand>>) {
    for C2sPacket(_, data) in events_in.into_iter() {
        info!("client brand: {}", data.brand);
    }
}

fn init_play_system(players: Query<(Entity, &Play), Added<Play>>) {
    for (entity, play) in players.iter() {
        play.send_packet(&s2c::PlayPacket::Login {
            entity_id: entity.index().into(),
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
            gamemode: s2c::GameMode::Survival,
            previous_gamemode: s2c::PreviousGameMode::None,
            is_debug: false.into(),
            is_flat: false.into(),
            died: None.into(),
            portal_cooldown: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerAbilities {
            flags: 0b00000010.into(),
            flying_speed: 0.05.into(),
            fov_modifier: 0.1.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::WorldRespawn {
            position: types::Position(0, 40, 0),
            pitch: 0.0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerTeleport {
            x: 0.0.into(),
            y: 40.0.into(),
            z: 0.0.into(),
            yaw: 0.0.into(),
            pitch: 0.0.into(),
            flags: 0b00000000.into(),
            id: 0.into(),
        })
        .unwrap();
    }
}

fn chat_system(
    world: &World,
    players: Query<&Play>,
    mut events: EventReader<C2sPacket<packet::ChatMessage>>,
) {
    const GRAY: &str = "#555555";

    for C2sPacket(from, data) in events.iter() {
        let login: &Login = world.get(*from).unwrap();
        let formatted: types::JsonChat =
            chat!(("{}", login.username) (c[GRAY] " > ") ("{}", data.content)).into();

        for play in players.iter() {
            play.send_packet(&s2c::PlayPacket::ChatSystemMessage {
                content: formatted.clone(),
                overlay: false.into(),
            })
            .unwrap();
        }
    }
}
