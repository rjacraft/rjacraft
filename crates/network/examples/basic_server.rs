use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_network::*;
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

fn status_system(_peer: In<Entity>) -> serde_json::Value {
    serde_json::json!({
        "version": {
            "protocol": 763,
            "name": "Rjacraft 1.20.1",
        },
        "players": {
            "online": 0,
            "max": 100,
            "sample": []
        },
        "description": {
            "text": "Rjacraft"
        }
    })
}

fn handle_disconnect(mut events: EventReader<PeerDisconnected>) {
    for event in events.iter() {
        info!("disconnect: {:?}", event);
    }
}
