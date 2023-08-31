use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bevy_app::{App, Plugin, PostStartup, PreUpdate, Update};
use bevy_ecs::event::EventWriter;
use bevy_ecs::prelude::{Commands, Component, Entity, Event, EventReader, Query, Res, World};
use bevy_ecs::system::{Resource, SystemState};
use bytes::{Bytes, BytesMut};
use flume::{Receiver, Sender, TryRecvError, TrySendError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use rjacraft_protocol::packets::client::{
    ClientHandshakePacket, ClientStatusPacket, HandshakeState,
};
use rjacraft_protocol::{Decoder, Encoder};

use crate::decode::PacketDecoder;
use crate::encode::PacketEncoder;

mod decode;
mod encode;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        if let Err(e) = build_plugin(app) {
            error!("Failed to build network plugin: {e:#}");
        }
    }
}

fn build_plugin(app: &mut App) -> anyhow::Result<()> {
    let runtime = Runtime::new()?;

    let (new_connections_send, new_connections_recv) = flume::bounded::<RemoteConnection>(64);
    let shared = SharedNetworkState(Arc::new(SharedNetworkStateInner {
        new_connections_send,
        new_connections_recv,
    }));

    app.insert_resource(shared.clone());

    let accept_loop_system = move |shared: Res<SharedNetworkState>| {
        let _guard = runtime.handle().enter();
        tokio::spawn(accept_loop(shared.clone()));
    };

    let spawn_new_connections = move |world: &mut World| {
        while let Ok(connection) = shared.0.new_connections_recv.try_recv() {
            world.spawn(connection);
        }
    };

    app.add_systems(PostStartup, accept_loop_system);
    app.add_systems(PreUpdate, spawn_new_connections);
    app.add_event::<PacketReceivedEvent>();
    app.add_event::<DisconnectEvent>();
    app.add_event::<HandshakeEvent>();
    app.add_event::<ClientStatusRequestEvent>();
    app.add_systems(Update, run_packet_event_loop);
    app.add_systems(Update, connection_event_handler);

    Ok(())
}

async fn accept_loop(shared: SharedNetworkState) {
    let addr = "0.0.0.0:25565";
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Listening at: {addr}");
            listener
        }
        Err(e) => {
            error!("Failed to start TCP listener: {e}");
            return;
        }
    };

    let timeout = Duration::from_secs(5);

    loop {
        match listener.accept().await {
            Ok((stream, remote_addr)) => {
                let shared = shared.clone();

                tokio::spawn(async move {
                    if let Err(e) = tokio::time::timeout(
                        timeout,
                        handle_connection(shared, stream, remote_addr),
                    )
                    .await
                    {
                        error!("Connection timed out: {e}");
                    }
                });
            }
            Err(e) => {
                println!("Failed to accept connection: {e}");
            }
        }
    }
}

const READ_BUF_SIZE: usize = 4096;

#[derive(Clone, Debug)]
pub struct PacketFrame {
    pub timestamp: Instant,
    pub payload: Bytes,
}

async fn handle_connection(shared: SharedNetworkState, stream: TcpStream, remote_addr: SocketAddr) {
    if let Err(e) = stream.set_nodelay(true) {
        error!("Failed to set TCP_NODELAY: {e}");
    }
    let (mut reader, mut writer) = stream.into_split();

    let (incoming_sender, incoming_receiver) = flume::unbounded::<PacketFrame>();
    let recv_task = tokio::spawn(async move {
        let mut buf = BytesMut::new();
        let mut decoder = PacketDecoder::new();

        loop {
            let payload = match decoder.try_next_frame() {
                Ok(Some(frame)) => frame,
                Ok(None) => {
                    // Incomplete packet
                    buf.reserve(READ_BUF_SIZE);
                    match reader.read_buf(&mut buf).await {
                        Ok(0) => {
                            break;
                        } // Reader is at EOF.
                        Ok(_) => {}
                        Err(e) => {
                            error!("error reading data from stream: {e}");
                            break;
                        }
                    }
                    decoder.queue_bytes(buf.split());
                    continue;
                }
                Err(e) => {
                    error!("error decoding packet frame: {e:#}");
                    break;
                }
            };
            let timestamp = Instant::now();
            let frame = PacketFrame {
                timestamp,
                payload: payload.freeze(),
            };

            if let Err(e) = incoming_sender.try_send(frame) {
                error!("error sending packet frame: {e:#}");
            }
        }
    });

    let (outgoing_sender, outgoing_receiver) = flume::unbounded::<BytesMut>();
    let send_task = tokio::spawn(async move {
        loop {
            let bytes = match outgoing_receiver.try_recv() {
                Ok(frame) => frame,
                Err(e) => match e {
                    TryRecvError::Empty => continue,
                    TryRecvError::Disconnected => break,
                },
            };

            if let Err(e) = writer.write_all(&bytes).await {
                warn!("error writing data to stream: {e}")
            }
        }
    });

    let connection = RemoteConnection {
        remote_addr,
        recv: incoming_receiver,
        send: outgoing_sender,
        encoder: PacketEncoder::new(),
        recv_task,
        send_task,
        state: HandshakeState::Handshaking,
    };

    let _ = shared.0.new_connections_send.send_async(connection).await;
}

#[allow(clippy::type_complexity)]
fn run_packet_event_loop(
    world: &mut World,
    state: &mut SystemState<(
        Query<(Entity, &mut RemoteConnection)>,
        EventWriter<PacketReceivedEvent>,
        EventWriter<DisconnectEvent>,
        Commands,
    )>,
) {
    let (mut connections, mut packet_events, mut disconnect_events, mut commands) =
        state.get_mut(world);

    for (entity, mut connection) in &mut connections {
        let result = connection.try_recv();

        match result {
            Ok(frame) => {
                packet_events.send(PacketReceivedEvent {
                    connection: entity,
                    frame,
                });
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                debug!("Client disconnected");
                disconnect_events.send(DisconnectEvent { connection: entity });
                commands.entity(entity).remove::<RemoteConnection>();
            }
        }
    }

    state.apply(world);
}

fn connection_event_handler(
    mut packets: EventReader<PacketReceivedEvent>,
    mut handshake_events: EventWriter<HandshakeEvent>,
    mut client_status_request_events: EventWriter<ClientStatusRequestEvent>,
    mut connections: Query<&mut RemoteConnection>,
) {
    for packet in packets.iter() {
        let payload = &packet.frame.payload;
        if let Ok(mut connection) = connections.get_mut(packet.connection) {
            match &connection.state {
                HandshakeState::Handshaking => {
                    match ClientHandshakePacket::decode(&mut std::io::Cursor::new(payload)) {
                        Ok(p) => match p {
                            ClientHandshakePacket::Handshake(p) => {
                                handshake_events.send(HandshakeEvent {
                                    connection: packet.connection,
                                    protocol_version: p.protocol_version,
                                    server_address: p.server_address,
                                    server_port: p.server_port,
                                    next_state: p.next_state,
                                });
                                connection.state = p.next_state;
                            }
                        },
                        Err(e) => {
                            error!("error: {:?}", e)
                        }
                    };
                }
                HandshakeState::Status => {
                    match ClientStatusPacket::decode(&mut std::io::Cursor::new(payload)) {
                        Ok(p) => match p {
                            ClientStatusPacket::Request(_) => {
                                client_status_request_events.send(ClientStatusRequestEvent {
                                    connection: packet.connection,
                                })
                            }
                            ClientStatusPacket::Ping(ping) => {
                                connection.send_packet(
                                        rjacraft_protocol::packets::server::ServerStatusPacket::Pong(
                                            rjacraft_protocol::packets::server::Pong {
                                                payload: ping.payload
                                            }
                                        )
                                    ).unwrap();
                                connection.flush_packets();
                            }
                        },
                        Err(e) => {
                            error!("error: {:?}", e)
                        }
                    };
                }
                HandshakeState::Login => {}
                HandshakeState::Configuration => {}
                HandshakeState::Play => {}
            }
        }
    }
}

#[derive(Resource, Clone)]
pub struct SharedNetworkState(Arc<SharedNetworkStateInner>);

struct SharedNetworkStateInner {
    new_connections_send: Sender<RemoteConnection>,
    new_connections_recv: Receiver<RemoteConnection>,
}

#[derive(Component)]
pub struct RemoteConnection {
    pub remote_addr: SocketAddr,
    recv: Receiver<PacketFrame>,
    send: Sender<BytesMut>,
    encoder: PacketEncoder,
    recv_task: JoinHandle<()>,
    send_task: JoinHandle<()>,
    pub state: HandshakeState,
}

impl RemoteConnection {
    pub fn try_recv(&mut self) -> Result<PacketFrame, TryRecvError> {
        self.recv.try_recv()
    }

    pub fn write_packet_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.encoder.append_frame(bytes)
    }

    pub fn flush_packets(&mut self) -> Result<(), TrySendError<BytesMut>> {
        let bytes = self.encoder.take();
        if !bytes.is_empty() {
            self.send.try_send(bytes)
        } else {
            Ok(())
        }
    }

    pub fn send_packet<T>(&mut self, packet: T) -> anyhow::Result<()>
    where
        T: Encoder + Debug,
    {
        self.encoder.append_packet(packet)
    }
}

impl Drop for RemoteConnection {
    fn drop(&mut self) {
        println!("dropping connection {}", self.remote_addr);
        _ = self.flush_packets();
        self.recv_task.abort();
        self.send_task.abort();
    }
}

#[derive(Event, Clone, Debug)]
pub struct PacketReceivedEvent {
    pub connection: Entity,
    pub frame: PacketFrame,
}

#[derive(Event, Clone, Debug)]
pub struct DisconnectEvent {
    pub connection: Entity,
}

#[derive(Event, Clone, Debug)]
pub struct HandshakeEvent {
    connection: Entity,
    protocol_version: i32,
    server_address: String,
    server_port: u16,
    next_state: HandshakeState,
}

impl HandshakeEvent {
    pub fn connection(&self) -> Entity {
        self.connection
    }

    pub fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    pub fn server_address(&self) -> &str {
        &self.server_address
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn next_state(&self) -> HandshakeState {
        self.next_state
    }
}

#[derive(Event, Clone, Debug)]
pub struct ClientStatusRequestEvent {
    pub connection: Entity,
}
