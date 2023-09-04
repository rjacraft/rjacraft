use std::net::SocketAddr;

use rjacraft_protocol::{frame::*, packets::*, ProtocolType, ProtocolVersion};
use tokio::{io, net, task};
use tracing::*;

// Some of these are invalid at some states
pub enum PeerMsgIn {
    Drop,
    StatusPacket(s2c::StatusPacket),
    // LoginPacket(cb::LoginPacket),
    ConfigurationPacket(s2c::ConfigurationPacket),
    PlayPacket(s2c::PlayPacket),
}

pub enum PeerMsgOut {
    Disconnected,
    HandshakeComplete(ProtocolVersion, String, u16),
    NeedStatus,
    // Authenticate(String, Option<uuid::Uuid>),
    // NeedConfiguration,
    // PlayPacket(sb::PlayPacket),
}

#[derive(Debug, Clone, Copy)]
enum ConnectionState {
    Handshake,
    Status,
    Login(ProtocolVersion),
    // Configuration,
    // Play,
}

#[derive(Debug, thiserror::Error)]
enum PeerLoopError {
    #[error(transparent)]
    Reading(#[from] ReadPacketError),
    #[error(transparent)]
    Writing(#[from] WritePacketError),
    #[error(transparent)]
    DecodingHandshake(#[from] c2s::HandshakePacketDecodeError),
    #[error(transparent)]
    DecodingStatus(#[from] c2s::StatusPacketDecodeError),
    #[error(transparent)]
    EncodingStatus(#[from] s2c::StatusPacketEncodeError),
    #[error("Wrong protocol version: {0:?}")]
    WrongVersion(rjacraft_protocol::ProtocolVersion),
}

async fn peer_loop(
    stream: net::TcpStream,
    msg_in: flume::Receiver<PeerMsgIn>,
    msg_out: flume::Sender<PeerMsgOut>,
) -> Result<(), PeerLoopError> {
    let (mut read, mut write) = stream.into_split();

    // reading varints isn't cancellable so here we are
    let (frames_tx, frames_rx) = flume::unbounded();
    let mut read_task: task::JoinHandle<Result<(), ReadPacketError>> = tokio::spawn(async move {
        loop {
            let packet = read_frame(&mut read).await?;

            let _ = frames_tx.send(packet);
        }
    });

    let mut state = ConnectionState::Handshake;

    debug!("got a peer");

    loop {
        tokio::select! {
            result = &mut read_task => {
                // if the task panics we panic with it
                return match result.unwrap() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e.into())
                }
            }
            Ok(msg) = msg_in.recv_async() => {
                match msg {
                    PeerMsgIn::Drop => return Ok(()),
                    PeerMsgIn::StatusPacket(packet) => {
                        write_frame(&mut write, &packet.encode_owned()?).await?;
                    }
                    _ => {}
                }
            }
            Ok(mut frame) = frames_rx.recv_async() => {
                match state {
                    ConnectionState::Handshake => {
                        let c2s::HandshakePacket::Handshake(hs) = c2s::HandshakePacket::decode(&mut frame)?;

                        state = match hs.next_state {
                            c2s::NextState::Status => ConnectionState::Status,
                            c2s::NextState::Login => ConnectionState::Login(hs.protocol_version),
                        };

                        debug!("handshake complete, {}", hs.protocol_version);
                        debug!("next state: {state:?}");

                        let _ = msg_out
                            .send(PeerMsgOut::HandshakeComplete(
                                hs.protocol_version,
                                hs.server_address.into(),
                                hs.server_port.into(),
                            ))
                            ;
                    }
                    ConnectionState::Status => {
                        let packet = c2s::StatusPacket::decode(&mut frame)?;

                        debug!("{packet:#?}");

                        match packet {
                            c2s::StatusPacket::Ping(ping) => {
                                write_frame(
                                    &mut write,
                                    &s2c::StatusPacket::Pong(s2c::status::Pong {
                                        payload: ping.payload,
                                    }).encode_owned()?,
                                )
                                .await?;

                                return Ok(());
                            }
                            c2s::StatusPacket::Request(_) => {
                                let _ = msg_out.send(PeerMsgOut::NeedStatus);
                            }
                        }
                    }
                    ConnectionState::Login(protocol) => {
                        if protocol != rjacraft_protocol::SUPPORTED_PROTOCOL {
                            Err(PeerLoopError::WrongVersion(protocol))?;
                        }
                    }
                }
            }
        }
    }
}

pub async fn network_loop(
    addr: impl net::ToSocketAddrs,
    new_peer_tx: flume::Sender<(
        SocketAddr,
        flume::Sender<PeerMsgIn>,
        flume::Receiver<PeerMsgOut>,
    )>,
) -> io::Result<()> {
    let listener = net::TcpListener::bind(addr).await?;

    info!("listening at {}", listener.local_addr()?);

    loop {
        let (stream_in, addr_in) = listener.accept().await?;

        let (msg_in_tx, msg_in_rx) = flume::unbounded();
        let (msg_out_tx, msg_out_rx) = flume::unbounded();

        let _ = new_peer_tx.send((addr_in, msg_in_tx, msg_out_rx));

        tokio::spawn(
            async move {
                if let Err(e) = peer_loop(stream_in, msg_in_rx, msg_out_tx.clone()).await {
                    info!("peer loop failed: {e}");
                } else {
                    info!("peer loop ended");
                }

                let _ = msg_out_tx.send(PeerMsgOut::Disconnected);
            }
            .instrument(tracing::info_span!("peer_loop", ?addr_in)),
        );
    }
}
