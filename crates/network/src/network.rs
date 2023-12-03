use std::net::SocketAddr;

use rjacraft_protocol::{frame::*, packets::*, types::*, ProtocolType, ProtocolVersion};
use tokio::{io, net, pin};
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
    NeedConfiguration,
    ConfigurationFinished,
    TeleportConfirm(i32),
    Brand(packet::ClientBrand),
    // ConfigurationRpResponse(i32),
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
    #[error("Keep alive error")]
    KeepAlive(#[from] keepalive::Error),
}

async fn peer_loop(
    stream: net::TcpStream,
    n2b: flume::Sender<N2bEvent>,
    b2n: flume::Receiver<B2nEvent>,
) -> Result<(), PeerLoopError> {
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

    let mut read_task = Box::pin(reader.read_frame());

    let (to_ka_tx, to_ka_rx) = flume::unbounded::<i64>();
    let (from_ka_tx, from_ka_rx) = flume::unbounded::<i64>();
    let ka_task = keepalive::keepalive_loop(from_ka_tx, to_ka_rx);
    pin!(ka_task);

    loop {
        tokio::select! {
            result = &mut read_task => {
                match state
                    .on_frame(result?, &mut writer, &n2b, &to_ka_tx)
                    .instrument(info_span!("conn_state", ?state))
                    .await?
                {
                    state::Action::DropConnection => return Ok(()),
                    state::Action::NewState(x) => {
                        debug!("{state:?} -> {x:?}");
                        state = x
                    },
                    state::Action::Continue => {}
                }

                drop(read_task);
                read_task = Box::pin(reader.read_frame());
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

                        drop(read_task);
                        reader.compress = threshold.is_some();
                        read_task = Box::pin(reader.read_frame());
                        writer.compress = threshold;

                        debug!("set compression to {threshold:?}");
                    }
                }
            },
            Ok(command) = from_ka_rx.recv_async() => if let Some(x) = state.keepalive_packet(command) {
                writer.write_frame(&x).await?;
            },

            error = &mut ka_task => return Err(error.into()),
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
