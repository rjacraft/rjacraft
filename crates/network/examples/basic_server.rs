use bevy_app::{App, RunMode, ScheduleRunnerPlugin, Update};
use bevy_ecs::prelude::{EventReader, Query};
use rjacraft_network::*;
use rjacraft_protocol::packets::server::*;
use serde_json::json;

fn main() {
    tracing_subscriber::fmt::init();
    App::new()
        .add_plugins((
            NetworkPlugin("0.0.0.0:25565"),
            ScheduleRunnerPlugin {
                run_mode: RunMode::Loop { wait: None },
            },
        ))
        .add_systems(Update, handle_status_requests)
        .run();
}

fn handle_status_requests(
    mut connections: Query<&mut RemoteConnection>,
    mut events: EventReader<ClientStatusRequestEvent>,
) {
    for event in events.iter() {
        if let Ok(mut connection) = connections.get_mut(event.connection) {
            println!("status for connection: {:?}", connection.remote_addr);
            let packet = ServerStatusPacket::Response(Response {
                response: json!({
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
                .to_string(),
            });

            let _ = connection.send_packet(packet);
            if let Err(e) = connection.flush_packets() {
                println!("Can't flush: {}", e)
            };
        }
    }
}

fn handle_disconnect(mut events: EventReader<DisconnectEvent>) {
    for event in events.iter() {
        println!("disconnect: {:?}", event);
    }
}
