use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::types::server_status;
use tracing::*;

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
                    compress: None,
                    status_system,
                    auth_system,
                    brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(Update, handle_disconnect)
        .run();
}

fn status_system(In(entity): In<Entity>, world: &World) -> server_status::ServerStatus {
    let peer: &Peer = world.get(entity).unwrap();

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
        description: text!(
            ("Example: " (b "basic server"))
            ("\nYour IP: " (b,c["#22ff22"] "{}", peer.addr.ip()))
        ),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn auth_system(In(_): In<(Entity, UsernameString, uuid::Uuid)>) -> AuthOutcome {
    AuthOutcome::Fail(text!("Logging in is not supported"))
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    None
}

fn handle_disconnect(mut events: EventReader<PeerDisconnected>) {
    for event in events.iter() {
        info!("disconnect: {:?}", event);
    }
}
