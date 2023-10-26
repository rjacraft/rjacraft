use std::net::SocketAddr;

use bevy_ecs::prelude::*;

use crate::network::*;

#[derive(Component)]
pub struct Peer {
    pub addr: SocketAddr,
    pub(crate) b2n: flume::Sender<B2nEvent>,
    pub(crate) n2b: flume::Receiver<N2bEvent>,
}

#[derive(Component)]
pub struct Handshaken {
    pub protocol_version: rjacraft_protocol::ProtocolVersion,
    pub server_address: String,
    pub server_port: u16,
}
