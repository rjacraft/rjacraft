use std::net::SocketAddr;

use rjacraft_protocol::{frame::*, packets::*, types::*, ProtocolVersion};
use tokio::{io, net};
use tracing::*;

use super::traced_error;
use crate::packet;

mod frames;
mod keepalive;
mod state;

pub type NewPeer = (
    SocketAddr,
    flume::Sender<B2nEvent>,
    flume::Receiver<N2bEvent>,
);

#[derive(Debug)]
pub enum B2nEvent {
    Drop,
    Status(ServerStatus),
    LoginSucceeded,
    LoginPacket(s2c::LoginPacket),
    ConfigurationPacket(s2c::ConfigurationPacket),
}

#[derive(Debug)]
pub enum N2bEvent {
    Disconnected,
    HandshakeComplete(ProtocolVersion, String, u16),
    NeedStatus,
    Authenticate(String, Uuid),
    NeedConfiguration,
    ConfigurationFinished(flume::Sender<bytes::Bytes>),
    Brand(packet::ClientBrand),
    // ConfigurationRpResponse(i32),
    Chat(packet::Chat),
    Interact(packet::Interact),
    Movement(packet::Movement),
    Input(packet::Input),
    ClientInfo(packet::ClientInfo),
    Container(packet::Window),
}

#[derive(Debug, thiserror::Error)]
enum PeerLoopError {
    #[error(transparent)]
    Reading(#[from] ReadFrameError),
    #[error(transparent)]
    Writing(#[from] WriteFrameError),
    #[error(transparent)]
    StateMachine(#[from] state::Error),
}

async fn peer_loop(
    stream: net::TcpStream,
    n2b: flume::Sender<N2bEvent>,
    b2n: flume::Receiver<B2nEvent>,
) -> Result<(), PeerLoopError> {
    let (read, write) = stream.into_split();
    let (c2s_tx, c2s_rx) = flume::unbounded();
    let (s2c_tx, s2c_rx) = flume::unbounded();
    let (to_ka_tx, to_ka_rx) = flume::unbounded();
    let (from_ka_tx, from_ka_rx) = flume::unbounded();

    tokio::spawn(frames::frame_write_loop(write, s2c_rx).in_current_span());

    let mut read_task = tokio::spawn(frames::frame_read_loop(read, c2s_tx).in_current_span());
    let mut state_task = tokio::spawn(
        state::state_machine_loop(n2b, b2n, to_ka_tx, from_ka_rx, s2c_tx, c2s_rx).in_current_span(),
    );
    let mut ka_task =
        tokio::spawn(keepalive::keepalive_loop(from_ka_tx, to_ka_rx).in_current_span());

    let result = tokio::select! {
        result = &mut read_task => {
            match result.unwrap() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into())
            }
        }
        result = &mut state_task => {
            match result.unwrap() {
                Ok(()) => Ok(()),
                Err(e) => Err(e.into())
            }
        }
        result = &mut ka_task => Ok(result.unwrap())
    };

    read_task.abort();
    state_task.abort();
    ka_task.abort();

    // the write task will wait until all the frames are finished

    result
}

pub async fn network_loop(
    addr: impl net::ToSocketAddrs,
    new_peer_tx: flume::Sender<NewPeer>,
) -> io::Result<()> {
    let listener = net::TcpListener::bind(addr).await?;

    info!("Listening at {}", listener.local_addr()?);

    loop {
        let (stream_in, addr_in) = listener.accept().await?;
        let (n2b_tx, n2b_rx) = flume::unbounded();
        let (b2n_tx, b2n_rx) = flume::unbounded();

        let _ = new_peer_tx.send((addr_in, b2n_tx.clone(), n2b_rx.clone()));

        tokio::spawn(
            async move {
                info!("Got a peer");

                if let Err(e) = peer_loop(stream_in, n2b_tx.clone(), b2n_rx).await {
                    info!("Peer loop failed:\n{}", traced_error::TracedError(e));
                } else {
                    info!("Peer loop ended");
                }

                let _ = n2b_tx.send(N2bEvent::Disconnected);
            }
            .instrument(info_span!("peer_task", ?addr_in)),
        );
    }
}
