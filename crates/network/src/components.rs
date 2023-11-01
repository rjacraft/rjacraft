use std::net::SocketAddr;

use bevy_ecs::prelude::*;
use rjacraft_protocol::{packets::s2c, ProtocolType};

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
    pub(crate) tx: flume::Sender<bytes::Bytes>,
}

impl Play {
    pub fn send_packet(
        &self,
        packet: &s2c::PlayPacket,
    ) -> Result<&Self, s2c::PlayPacketEncodeError> {
        let _ = self.tx.send(packet.encode_owned()?);

        Ok(self)
    }
}
