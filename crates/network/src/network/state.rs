use rjacraft_protocol::{
    frame::*,
    packets::{c2s, s2c},
    types::{self, *},
    ProtocolType,
};
use tokio::io;
use tracing::*;

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Writing(#[from] WritePacketError),
    #[error(transparent)]
    DecodingHandshake(#[from] c2s::HandshakePacketDecodeError),
    #[error(transparent)]
    DecodingStatus(#[from] c2s::StatusPacketDecodeError),
    #[error(transparent)]
    EncodingStatus(#[from] s2c::StatusPacketEncodeError),
    #[error(transparent)]
    DecodingLogin(#[from] c2s::LoginPacketDecodeError),
    #[error(transparent)]
    EncodingLogin(#[from] s2c::LoginPacketEncodeError),
    #[error(transparent)]
    DecodingConfiguration(#[from] c2s::ConfigurationPacketDecodeError),
    #[error("Failed to decode brand")]
    DecodingBrand(#[source] types::len_string::DecodeError<128>),
    #[error(transparent)]
    EncodingConfiguration(#[from] s2c::ConfigurationPacketEncodeError),
    #[error(transparent)]
    DecodingPlay(#[from] c2s::PlayPacketDecodeError),
    #[error(transparent)]
    EncodingPlay(#[from] s2c::PlayPacketEncodeError),
    #[error("Wrong protocol version: {0:?}")]
    WrongVersion(rjacraft_protocol::ProtocolVersion),
    #[error("Got login ack despite not being logged in")]
    FakeLoginAck,
}

enum Action {
    DropConnection,
    NewState(ConnectionState),
    Continue,
}

#[derive(Debug, Clone, Copy)]
enum ConnectionState {
    Handshake,
    Status,
    Login { completed: bool },
    Configuration,
    Play,
}

impl ConnectionState {
    async fn on_b2n(
        self,
        write: &mut (impl io::AsyncWrite + Unpin + Send),
        command: B2nEvent,
    ) -> Result<Action, Error> {
        match command {
            B2nEvent::Drop => Ok(Action::DropConnection),
            B2nEvent::Status(status) => {
                write_frame(
                    write,
                    &s2c::StatusPacket::Response(status.into()).encode_owned()?,
                )
                .await?;

                Ok(Action::Continue)
            }
            B2nEvent::LoginSucceeded => {
                Ok(Action::NewState(ConnectionState::Login { completed: true }))
            }
            B2nEvent::LoginPacket(packet) => {
                write_frame(write, &packet.encode_owned()?).await?;

                Ok(Action::Continue)
            }
            B2nEvent::ConfigurationPacket(packet) => {
                write_frame(write, &packet.encode_owned()?).await?;

                Ok(Action::Continue)
            }
            B2nEvent::PlayPacket(packet) => {
                write_frame(write, &packet.encode_owned()?).await?;

                Ok(Action::Continue)
            }
        }
    }

    async fn on_frame(
        self,
        frame: &mut bytes::Bytes,
        write: &mut (impl io::AsyncWrite + Unpin + Send),
        n2b: &flume::Sender<N2bEvent>,
        to_ka: &flume::Sender<i64>,
    ) -> Result<Action, Error> {
        match self {
            ConnectionState::Handshake => {
                let c2s::HandshakePacket::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                } = c2s::HandshakePacket::decode(frame)?;

                debug!("handshake complete, {}", protocol_version);

                let _ = n2b.send(N2bEvent::HandshakeComplete(
                    protocol_version,
                    server_address.into(),
                    server_port.into(),
                ));

                match next_state {
                    c2s::NextState::Status => {
                        // yeah ok whatever
                        Ok(Action::NewState(ConnectionState::Status))
                    }
                    c2s::NextState::Login => {
                        if protocol_version == rjacraft_protocol::SUPPORTED_PROTOCOL {
                            Ok(Action::NewState(ConnectionState::Login {
                                completed: false,
                            }))
                        } else {
                            write_frame(
                                write,
                                &s2c::LoginPacket::Disconnect {
                                    reason: JsonString(chat::Chat {
                                        text: "Incompatible game version".into(),
                                        attrs: Default::default(),
                                        extra: vec![],
                                    }),
                                }
                                .encode_owned()?,
                            )
                            .await?;

                            Err(Error::WrongVersion(protocol_version))
                        }
                    }
                }
            }
            ConnectionState::Status => {
                let packet = c2s::StatusPacket::decode(frame)?;

                match packet {
                    c2s::StatusPacket::Ping { payload } => {
                        write_frame(write, &s2c::StatusPacket::Pong { payload }.encode_owned()?)
                            .await?;

                        Ok(Action::DropConnection)
                    }
                    c2s::StatusPacket::Request => {
                        let _ = n2b.send(N2bEvent::NeedStatus);

                        Ok(Action::Continue)
                    }
                }
            }
            ConnectionState::Login { completed } => {
                let packet = c2s::LoginPacket::decode(frame)?;

                debug!("{packet:?}");

                match packet {
                    c2s::LoginPacket::LoginStart { username, uuid } => {
                        let _ = n2b.send(N2bEvent::Authenticate(username.into(), uuid));

                        Ok(Action::Continue)
                    }
                    c2s::LoginPacket::EncryptionResponse { .. } => todo!(),
                    c2s::LoginPacket::LoginPluginResponse { .. } => todo!(),
                    c2s::LoginPacket::SuccessAck => {
                        if completed {
                            let _ = n2b.send(N2bEvent::NeedConfiguration);

                            Ok(Action::NewState(ConnectionState::Configuration))
                        } else {
                            Err(Error::FakeLoginAck)
                        }
                    }
                }
            }
            ConnectionState::Configuration => {
                let packet = c2s::ConfigurationPacket::decode(frame)?;

                debug!("{packet:?}");

                match packet {
                    c2s::ConfigurationPacket::PluginMessage { channel, data } => {
                        let mut data: bytes::Bytes = data.into();

                        if let ("minecraft", "brand") = channel.parts() {
                            let brand = types::LenString::<128>::decode(&mut data)
                                .map_err(Error::DecodingBrand)?;

                            let _ = n2b.send(N2bEvent::Brand(brand.into()));
                        }

                        Ok(Action::Continue)
                    }
                    c2s::ConfigurationPacket::FinishConfiguration => {
                        Ok(Action::NewState(ConnectionState::Play))
                    }
                    c2s::ConfigurationPacket::KeepAlive { id } => {
                        let _ = to_ka.send(id.into());

                        Ok(Action::Continue)
                    }
                    c2s::ConfigurationPacket::Pong { .. } => todo!(),
                    c2s::ConfigurationPacket::ResourcePack { .. } => todo!(),
                }
            }
            ConnectionState::Play => {
                let packet = c2s::PlayPacket::decode(frame)?;

                debug!("{packet:?}");

                match packet {
                    c2s::PlayPacket::KeepAlive { id } => {
                        let _ = to_ka.send(id.into());

                        Ok(Action::Continue)
                    }
                }
            }
        }
    }

    async fn on_keepalive(
        self,
        write: &mut (impl io::AsyncWrite + Unpin + Send),
        command: keepalive::Message,
    ) -> Result<Action, Error> {
        match (command, self) {
            (keepalive::Message::Packet(id), ConnectionState::Configuration) => {
                write_frame(
                    write,
                    &s2c::ConfigurationPacket::KeepAlive { id: id.into() }.encode_owned()?,
                )
                .await?;
                Ok(Action::Continue)
            }
            (keepalive::Message::Packet(id), ConnectionState::Play) => {
                write_frame(
                    write,
                    &s2c::PlayPacket::KeepAlive { id: id.into() }.encode_owned()?,
                )
                .await?;
                Ok(Action::Continue)
            }
            (keepalive::Message::Mismatch, _) => Ok(Action::DropConnection),
            (keepalive::Message::Timeout, _) => Ok(Action::DropConnection),
            _ => Ok(Action::Continue),
        }
    }
}

pub async fn state_machine_loop(
    n2b: flume::Sender<N2bEvent>,
    b2n: flume::Receiver<B2nEvent>,
    to_ka: flume::Sender<i64>,
    from_ka: flume::Receiver<keepalive::Message>,
    mut write: impl io::AsyncWrite + Unpin + Send,
    frames: flume::Receiver<bytes::Bytes>,
) -> Result<(), Error> {
    let mut state = ConnectionState::Handshake;

    loop {
        tokio::select! {
            Ok(command) = b2n.recv_async() => {
                match state.on_b2n(&mut write, command).await? {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => state = x,
                    Action::Continue => {}
                }
            },
            Ok(mut frame) = frames.recv_async() => {
                match state
                    .on_frame(&mut frame, &mut write, &n2b, &to_ka)
                    .instrument(info_span!("conn_state", ?state))
                    .await?
                {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => state = x,
                    Action::Continue => {}
                }
            }
            Ok(command) = from_ka.recv_async() => {
                match state.on_keepalive(&mut write, command).await? {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => state = x,
                    Action::Continue => {}
                }
            },
        }
    }
}
