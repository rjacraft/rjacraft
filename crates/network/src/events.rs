use bevy_ecs::prelude::*;
use rjacraft_protocol::packets::*;

#[derive(Debug, Event)]
pub struct PeerConnected {
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct PeerDisconnected {
    /// Might not exist in [`bevy_app::PostUpdate`]
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct DropPeer {
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct ConfigurationPacketOut {
    pub peer: Entity,
    pub packet: s2c::ConfigurationPacket,
}

#[derive(Debug, Event)]
pub struct PlayPacketIn {
    pub peer: Entity,
    pub packet: c2s::PlayPacket,
}

#[derive(Debug, Event)]
pub struct PlayPacketOut {
    pub peer: Entity,
    pub packet: s2c::PlayPacket,
}
