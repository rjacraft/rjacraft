use std::{net::SocketAddr, pin};

use rjacraft_protocol::{frame::*, packets::*, types::*, ProtocolType, ProtocolVersion};
use tokio::{io, net, time};
use tracing::*;

use super::traced_error;
use crate::packet;

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
    Packet(bytes::Bytes),
    LoginSucceeded,
    Compression(Option<u32>),
}

#[derive(Debug)]
pub enum N2bEvent {
    Disconnected,
    HandshakeComplete(ProtocolVersion, String, u16),
    NeedStatus,
    Authenticate(crate::UsernameString, Uuid),
    NeedConfig,
    ConfigFinished,
    TeleportConfirm(i32),
    Brand(packet::ClientBrand),
    // ConfigRpResponse(i32),
    ChatMessage(packet::ChatMessage),
    Command(String),
    Interact(packet::Interact),
    Movement(packet::Movement),
    Input(packet::Input),
    Digging(packet::Digging),
    ClientInfo(packet::ClientInfo),
    Window(packet::Window),
}

#[derive(Debug, thiserror::Error)]
enum PeerLoopError {
    #[error(transparent)]
    Reading(#[from] ReaderError),
    #[error(transparent)]
    Writing(#[from] WriterError),
    #[error(transparent)]
    StateMachine(#[from] state::Error),
    #[error("Keepalive ID mismatch. Expected: {expected:?}, got: {got}")]
    KaMismatch { expected: Option<i64>, got: i64 },
    #[error("Keepalive timeout exceeded")]
    Timeout,
}

async fn peer_loop(
    stream: net::TcpStream,
    n2b: flume::Sender<N2bEvent>,
    b2n: flume::Receiver<B2nEvent>,
) -> Result<(), PeerLoopError> {
    // the point of this pinning mess is that we want to avoid spawning any tasks and storing any
    // messages

    let start_time = time::Instant::now();
    let mut state = state::ConnectionState::Handshake;
    let (read, write) = stream.into_split();
    let mut reader = Reader {
        source: read,
        compress: false,
    };
    let mut writer = Writer {
        sink: write,
        compress: None,
    };

    // TODO: eliminating this allocation would be the last optimization we can do.
    //   unfortunately, rust does not have a way to drop something and then replace it such that
    //   whatever it references opens up again
    let mut reading = Box::pin(reader.read_frame());
    let mut keepalive = pin::pin!(keepalive::KeepAlive::start(start_time));
    let mut keepalive_id = None;

    loop {
        tokio::select! {
            result = &mut reading => {
                match state
                    .on_frame(result?, &mut writer, &n2b)
                    .instrument(info_span!("conn_state", ?state))
                    .await?
                {
                    state::Action::DropConnection => return Ok(()),
                    state::Action::NewState(x) => {
                        debug!("{state:?} -> {x:?}");
                        state = x
                    },
                    state::Action::Continue => {}
                    state::Action::RefreshKa(id_in) => 'a: {
                        if let Some(id_out) = keepalive_id {
                            if id_out == id_in {
                                keepalive.set(keepalive::KeepAlive::start(start_time));
                                break 'a;
                            }
                        }

                        return Err(PeerLoopError::KaMismatch { expected: keepalive_id, got: id_in });
                    }
                }

                drop(reading);
                reading = Box::pin(reader.read_frame());
            },
            Ok(command) = b2n.recv_async() => {
                match command {
                    B2nEvent::Drop => return Ok(()),
                    B2nEvent::Packet(packet) => writer.write_frame(&packet).await?,
                    B2nEvent::LoginSucceeded => {
                        state = state::ConnectionState::Login { completed: true }
                    }
                    B2nEvent::Compression(threshold) => {
                        writer.write_frame(&
                            s2c::LoginPacket::SetCompression {
                                threshold: threshold.map(|x| x as i32).unwrap_or(-1).into(),
                            }
                            .to_bytes_expect(),
                        ).await?;

                        drop(reading);
                        reader.compress = threshold.is_some();
                        reading = Box::pin(reader.read_frame());
                        writer.compress = threshold;

                        debug!("set compression to {threshold:?}");
                    }
                }
            },
            element = &mut keepalive => {
                if let Some(id) = element {
                    if let Some(x) = state.keepalive_packet(id) {
                        keepalive_id = Some(id);
                        writer.write_frame(&x).await?;
                    }
                } else {
                    return Err(PeerLoopError::Timeout);
                }
            }
        }
    }
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
