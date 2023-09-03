use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_network::*;
use rjacraft_protocol::types::{chat, status_object};
use tracing::*;

fn main() {
    tracing_subscriber::fmt::init();
    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio lifetime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                status: IntoSystem::into_system(status_system),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(Update, handle_disconnect)
        .run();
}

fn status_system(_peer: In<Entity>) -> status_object::StatusObject {
    status_object::StatusObject {
        version: status_object::Version {
            name: "Snapshot whatever".into(),
            protocol: rjacraft_protocol::SUPPORTED_PROTOCOL,
        },
        players: status_object::Players {
            max: 100,
            online: 0,
            sample: vec![],
        },
        description: chat::Chat {
            text: "Example: ".into(),
            attrs: Default::default(),
            extra: vec![Box::new(chat::Chat {
                text: "basic server".into(),
                attrs: chat::Attrs {
                    bold: true,
                    ..Default::default()
                },
                extra: vec![],
            })],
        },
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn handle_disconnect(mut events: EventReader<PeerDisconnected>) {
    for event in events.iter() {
        info!("disconnect: {:?}", event);
    }
}
