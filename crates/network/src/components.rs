use std::net::SocketAddr;

use bevy_ecs::prelude::*;
use rjacraft_protocol::{packets::s2c, types};
use tracing::*;

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

#[derive(Component)]
pub struct Play {
    pub(crate) b2n: flume::Sender<B2nEvent>,
}

impl Play {
    pub fn send(&self, packet: types::Encoded<s2c::PlayPacket>) -> &Self {
        trace!("{packet:?}");

        let _ = self.b2n.send(B2nEvent::Packet(packet.data()));

        self
    }

    pub fn send_option(&self, packet: Option<types::Encoded<s2c::PlayPacket>>) -> &Self {
        if let Some(x) = packet {
            self.send(x);
        }

        self
    }
}
