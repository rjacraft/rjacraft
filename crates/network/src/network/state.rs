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
            B2nEvent::Status(packet) => {
                write_frame(write, &s2c::StatusPacket::Response(packet).encode_owned()?).await?;

                Ok(Action::Continue)
            }
            B2nEvent::AuthSuccess(packet) => {
                write_frame(
                    write,
                    &s2c::LoginPacket::LoginSuccess(packet).encode_owned()?,
                )
                .await?;

                Ok(Action::NewState(ConnectionState::Login { completed: true }))
            }
            B2nEvent::AuthFail(packet) => {
                write_frame(
                    write,
                    &s2c::LoginPacket::DisconnectLogin(packet).encode_owned()?,
                )
                .await?;

                Ok(Action::DropConnection)
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
                let c2s::HandshakePacket::Handshake(hs) = c2s::HandshakePacket::decode(frame)?;

                debug!("handshake complete, {}", hs.protocol_version);

                let _ = n2b.send(N2bEvent::HandshakeComplete(
                    hs.protocol_version,
                    hs.server_address.into(),
                    hs.server_port.into(),
                ));

                match hs.next_state {
                    c2s::NextState::Status => {
                        // yeah ok whatever
                        Ok(Action::NewState(ConnectionState::Status))
                    }
                    c2s::NextState::Login => {
                        if hs.protocol_version == rjacraft_protocol::SUPPORTED_PROTOCOL {
                            Ok(Action::NewState(ConnectionState::Login {
                                completed: false,
                            }))
                        } else {
                            write_frame(
                                write,
                                &s2c::LoginPacket::DisconnectLogin(s2c::login::DisconnectLogin {
                                    reason: JsonString(chat::Chat {
                                        text: "Incompatible game version".into(),
                                        attrs: Default::default(),
                                        extra: vec![],
                                    }),
                                })
                                .encode_owned()?,
                            )
                            .await?;

                            Err(Error::WrongVersion(hs.protocol_version))
                        }
                    }
                }
            }
            ConnectionState::Status => {
                let packet = c2s::StatusPacket::decode(frame)?;

                match packet {
                    c2s::StatusPacket::Ping(ping) => {
                        write_frame(
                            write,
                            &s2c::StatusPacket::Pong(s2c::status::Pong {
                                payload: ping.payload,
                            })
                            .encode_owned()?,
                        )
                        .await?;

                        Ok(Action::DropConnection)
                    }
                    c2s::StatusPacket::Request(_) => {
                        let _ = n2b.send(N2bEvent::NeedStatus);

                        Ok(Action::Continue)
                    }
                }
            }
            ConnectionState::Login { completed } => {
                let packet = c2s::LoginPacket::decode(frame)?;

                debug!("{packet:?}");

                match packet {
                    c2s::LoginPacket::LoginStart(login_start) => {
                        let _ = n2b.send(N2bEvent::Authenticate(
                            login_start.username.into(),
                            login_start.uuid,
                        ));

                        Ok(Action::Continue)
                    }
                    c2s::LoginPacket::EncryptionResponse(_) => todo!(),
                    c2s::LoginPacket::LoginPluginResponse(_) => todo!(),
                    c2s::LoginPacket::LoginAck(_) => {
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
                    c2s::ConfigurationPacket::PluginMessageConfiguration(x) => {
                        let mut data: bytes::Bytes = x.data.into();

                        if let ("minecraft", "brand") = x.channel.parts() {
                            let brand = types::LenString::<128>::decode(&mut data)
                                .map_err(Error::DecodingBrand)?;

                            let _ = n2b.send(N2bEvent::Brand(brand.into()));
                        }

                        Ok(Action::Continue)
                    }
                    c2s::ConfigurationPacket::FinishConfiguration(_) => {
                        Ok(Action::NewState(ConnectionState::Play))
                    }
                    c2s::ConfigurationPacket::KeepAlive(x) => {
                        let _ = to_ka.send(x.id.into());

                        Ok(Action::Continue)
                    }
                    c2s::ConfigurationPacket::Pong(_) => todo!(),
                    c2s::ConfigurationPacket::ResourcePack(_) => todo!(),
                }
            }
            ConnectionState::Play => {
                let packet = c2s::PlayPacket::decode(frame)?;

                debug!("{packet:?}");

                match packet {
                    c2s::PlayPacket::KeepAlive(x) => {
                        let _ = to_ka.send(x.id.into());

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
                    &s2c::ConfigurationPacket::KeepAlive(s2c::configuration::KeepAlive {
                        id: id.into(),
                    })
                    .encode_owned()?,
                )
                .await?;
                Ok(Action::Continue)
            }
            (keepalive::Message::Packet(id), ConnectionState::Play) => {
                write_frame(
                    write,
                    &s2c::PlayPacket::KeepAlive(s2c::configuration::KeepAlive { id: id.into() })
                        .encode_owned()?,
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
