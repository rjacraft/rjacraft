use bevy_ecs::prelude::*;
use rjacraft_network::{packet::Movement, *};
use tracing::*;

pub fn brand_system(mut events: EventReader<C2sPacket<packet::ClientBrand>>) {
    for C2sPacket(_, data) in events.into_iter() {
        info!("client brand: {}", data.brand);
    }
}

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
