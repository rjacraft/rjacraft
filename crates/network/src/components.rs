use std::net::SocketAddr;

use bevy_ecs::prelude::*;

use crate::net_thread::{PeerMsgIn, PeerMsgOut};

#[derive(Component)]
pub struct Peer {
    pub addr: SocketAddr,
    pub(crate) msg_in: flume::Sender<PeerMsgIn>,
    pub(crate) msg_out: flume::Receiver<PeerMsgOut>,
}

#[derive(Component)]
pub struct Handshaken {
    pub protocol_version: rjacraft_protocol::ProtocolVersion,
    pub server_address: String,
    pub server_port: u16,
}
