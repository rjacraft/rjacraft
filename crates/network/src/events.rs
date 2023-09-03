use bevy_ecs::prelude::*;
use rjacraft_protocol::packets::*;

#[derive(Debug, Event)]
pub struct PeerConnected {
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct PeerDisconnected {
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct DropPeer {
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct ConfigurationPacketOut {
    pub peer: Entity,
    pub packet: cb::ConfigurationPacket,
}

#[derive(Debug, Event)]
pub struct PlayPacketIn {
    pub peer: Entity,
    pub packet: sb::PlayPacket,
}

#[derive(Debug, Event)]
pub struct PlayPacketOut {
    pub peer: Entity,
    pub packet: cb::PlayPacket,
}
