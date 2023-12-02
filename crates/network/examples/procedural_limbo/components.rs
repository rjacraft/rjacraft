use bevy_ecs::prelude::*;
use rjacraft_network::{packet::Movement, *};
use rjacraft_protocol::{packets::s2c, ProtocolType};

#[derive(Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn movement_system(
    mut query: Query<&mut Position>,
    mut events: EventReader<C2sPacket<packet::Movement>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        if let &Movement::Position(x, y, z) = data {
            *query.get_mut(*entity).unwrap() = Position { x, y, z };
        }
    }
}

pub fn send_position_system(query: Query<(&Play, &Position), Added<Position>>) {
    for (play, position) in query.iter() {
        play.send(
            s2c::PlayPacket::PlayerTeleport {
                x: position.x.into(),
                y: position.y.into(),
                z: position.z.into(),
                yaw: 180.0.into(),
                pitch: 0.0.into(),
                relative: s2c::TeleportRelative::new(),
                id: 0.into(),
            }
            .to_encoded_expect(),
        );
    }
}

#[derive(Component)]
pub struct ClientInfo(pub packet::ClientInfo);

pub fn client_info_system(
    mut commands: Commands,
    mut events: EventReader<C2sPacket<packet::ClientInfo>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        commands.entity(*entity).insert(ClientInfo(data.clone()));
    }
}
