use rjacraft_protocol::{
    packets::{c2s, s2c},
    types::{self, *},
    ProtocolType,
};
use tracing::*;

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read packet")]
    DecodingHandshake(#[from] c2s::HandshakePacketDecodeError),
    #[error("Failed to read packet")]
    DecodingStatus(#[from] c2s::StatusPacketDecodeError),
    #[error("Failed to read packet")]
    EncodingStatus(#[from] s2c::StatusPacketEncodeError),
    #[error("Failed to read packet")]
    DecodingLogin(#[from] c2s::LoginPacketDecodeError),
    #[error("Failed to read packet")]
    EncodingLogin(#[from] s2c::LoginPacketEncodeError),
    #[error("Failed to read packet")]
    DecodingConfiguration(#[from] c2s::ConfigurationPacketDecodeError),
    #[error("Failed to decode brand")]
    DecodingBrand(#[source] types::len_string::DecodeError<128>),
    #[error("Failed to read packet")]
    EncodingConfiguration(#[from] s2c::ConfigurationPacketEncodeError),
    #[error("Failed to read packet")]
    DecodingPlay(#[from] c2s::PlayPacketDecodeError),
    #[error("Failed to read packet")]
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
        s2c: &flume::Sender<bytes::Bytes>,
        command: B2nEvent,
    ) -> Result<Action, Error> {
        match command {
            B2nEvent::Drop => Ok(Action::DropConnection),
            B2nEvent::Status(status) => {
                let _ = s2c.send(s2c::StatusPacket::Response(status.into()).encode_owned()?);
                Ok(Action::Continue)
            }
            B2nEvent::LoginSucceeded => {
                Ok(Action::NewState(ConnectionState::Login { completed: true }))
            }
            B2nEvent::LoginPacket(packet) => {
                let _ = s2c.send(packet.encode_owned()?);
                Ok(Action::Continue)
            }
            B2nEvent::ConfigurationPacket(packet) => {
                let _ = s2c.send(packet.encode_owned()?);
                Ok(Action::Continue)
            }
        }
    }

    async fn on_frame(
        self,
        frame: &mut bytes::Bytes,
        s2c: &flume::Sender<bytes::Bytes>,
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
                        return Ok(Action::NewState(ConnectionState::Status));
                    }
                    c2s::NextState::Login => {
                        if protocol_version == rjacraft_protocol::SUPPORTED_PROTOCOL {
                            return Ok(Action::NewState(ConnectionState::Login {
                                completed: false,
                            }));
                        } else {
                            let _ = s2c.send(
                                s2c::LoginPacket::Disconnect {
                                    reason: JsonString(chat::Chat {
                                        text: "Incompatible game version".into(),
                                        attrs: Default::default(),
                                        extra: vec![],
                                    }),
                                }
                                .encode_owned()?,
                            );

                            Err(Error::WrongVersion(protocol_version))?
                        }
                    }
                }
            }
            ConnectionState::Status => {
                let packet = c2s::StatusPacket::decode(frame)?;

                match packet {
                    c2s::StatusPacket::Ping { payload } => {
                        let _ = s2c.send(s2c::StatusPacket::Pong { payload }.encode_owned()?);

                        return Ok(Action::DropConnection);
                    }
                    c2s::StatusPacket::Request => {
                        let _ = n2b.send(N2bEvent::NeedStatus);
                    }
                }
            }
            ConnectionState::Login { completed } => {
                let packet = c2s::LoginPacket::decode(frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::LoginPacket::LoginStart { username, uuid } => {
                        let _ = n2b.send(N2bEvent::Authenticate(username.into(), uuid));
                    }
                    c2s::LoginPacket::EncryptionResponse { .. } => todo!(),
                    c2s::LoginPacket::LoginPluginResponse { .. } => todo!(),
                    c2s::LoginPacket::SuccessAck => {
                        if completed {
                            let _ = n2b.send(N2bEvent::NeedConfiguration);

                            return Ok(Action::NewState(ConnectionState::Configuration));
                        } else {
                            Err(Error::FakeLoginAck)?
                        }
                    }
                }
            }
            ConnectionState::Configuration => {
                let packet = c2s::ConfigurationPacket::decode(frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::ConfigurationPacket::PluginMessage { channel, data } => {
                        let mut data: bytes::Bytes = data.into();

                        if let ("minecraft", "brand") = channel.parts() {
                            let brand = types::LenString::<128>::decode(&mut data)
                                .map_err(Error::DecodingBrand)?;

                            let _ = n2b.send(N2bEvent::Brand(packet::ClientBrand {
                                brand: brand.into(),
                            }));
                        }
                    }
                    c2s::ConfigurationPacket::FinishConfiguration => {
                        let _ = n2b.send(N2bEvent::ConfigurationFinished(s2c.clone()));

                        return Ok(Action::NewState(ConnectionState::Play));
                    }
                    c2s::ConfigurationPacket::KeepAlive { id } => {
                        let _ = to_ka.send(id.into());
                    }
                    c2s::ConfigurationPacket::Pong { .. } => todo!(),
                    c2s::ConfigurationPacket::ResourcePack { .. } => todo!(),
                }
            }
            ConnectionState::Play => {
                let packet = c2s::PlayPacket::decode(frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::PlayPacket::PlayerTeleport { .. } => {}
                    c2s::PlayPacket::ChatCommand { command, .. } => {
                        let _ = n2b.send(N2bEvent::Command(command.into()));
                    }
                    c2s::PlayPacket::ChatMessage { message, .. } => {
                        let _ = n2b.send(N2bEvent::ChatMessage(packet::ChatMessage {
                            content: message.into(),
                        }));
                    }
                    c2s::PlayPacket::NetKeepAlive { id } => {
                        let _ = to_ka.send(id.into());
                    }
                    c2s::PlayPacket::ClientCommand(_) => {}
                    c2s::PlayPacket::ClientInfo {
                        locale,
                        view_distance,
                        chat_mode,
                        chat_colors,
                        skin_parts,
                        main_hand,
                        text_filtering,
                        show_on_listings,
                    } => {
                        let _ = n2b.send(N2bEvent::ClientInfo(packet::ClientInfo {
                            locale: locale.into(),
                            view_distance: view_distance.into(),
                            chat_mode,
                            chat_colors: chat_colors.into(),
                            main_hand,
                            skin_parts: skin_parts.into(),
                            text_filtering: text_filtering.into(),
                            show_on_listings: show_on_listings.into(),
                        }));
                    }
                    c2s::PlayPacket::ContainerButton { sync_id, button_id } => {
                        let _ = n2b.send(N2bEvent::Container(packet::Window {
                            sync_id: sync_id.into(),
                            action: packet::WindowAction::Button(button_id.into()),
                        }));
                    }
                    c2s::PlayPacket::ContainerClick {
                        sync_id,
                        slot,
                        button,
                        mode,
                        new_slots,
                        carried_item,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Container(packet::Window {
                            sync_id: sync_id.into(),
                            action: packet::WindowAction::Click {
                                slot: slot,
                                button: button.into(),
                                mode: mode.0 as u32,
                                new_slots: new_slots.0,
                                carried_item,
                            },
                        }));
                    }
                    c2s::PlayPacket::ContainerClose { sync_id } => {
                        let _ = n2b.send(N2bEvent::Container(packet::Window {
                            sync_id: sync_id.into(),
                            action: packet::WindowAction::Close,
                        }));
                    }
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::Interact { hand },
                        ..
                    } => {
                        // sneaking is a useless field
                        let _ = n2b.send(N2bEvent::Interact(packet::Interact::TouchEntity {
                            entity: entity.0,
                            at: None,
                            hand,
                        }));
                    }
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::Attack,
                        ..
                    } => {
                        let _ =
                            n2b.send(N2bEvent::Interact(packet::Interact::AttackEntity(entity.0)));
                    }
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::InteractAt { x, y, z, hand },
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Interact(packet::Interact::TouchEntity {
                            entity: entity.0,
                            at: Some((x.into(), y.into(), z.into())),
                            hand,
                        }));
                    }
                    c2s::PlayPacket::PlayerPosOng { x, y, z, on_ground } => {
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::Position(
                            x.into(),
                            y.into(),
                            z.into(),
                        )));
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )));
                    }
                    c2s::PlayPacket::PlayerPosRotOng {
                        x,
                        y,
                        z,
                        yaw,
                        pitch,
                        on_ground,
                    } => {
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::Position(
                            x.into(),
                            y.into(),
                            z.into(),
                        )));
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::Rotation(
                            pitch.into(),
                            yaw.into(),
                        )));
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )));
                    }
                    c2s::PlayPacket::PlayerRotOng {
                        yaw,
                        pitch,
                        on_ground,
                    } => {
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::Rotation(
                            pitch.into(),
                            yaw.into(),
                        )));
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )));
                    }
                    c2s::PlayPacket::PlayerOnGround(on_ground) => {
                        let _ = n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )));
                    }
                    c2s::PlayPacket::PlayerAbilties(_) => {}
                    c2s::PlayPacket::PlayerCommand { action, extra, .. } => {
                        let _ = n2b.send(N2bEvent::Input(match action {
                            c2s::PlayerCommand::SneakDown => packet::Input::Sneak(true),
                            c2s::PlayerCommand::SneakUp => packet::Input::Sneak(false),
                            c2s::PlayerCommand::LeaveBed => packet::Input::LeaveBed,
                            c2s::PlayerCommand::SprintUp => packet::Input::Sprint(true),
                            c2s::PlayerCommand::SprintDown => packet::Input::Sprint(false),
                            c2s::PlayerCommand::HorseJumpDown => {
                                packet::Input::HorseJumpStart(extra.0 as u8)
                            }
                            c2s::PlayerCommand::HorseJumpUp => packet::Input::HorseJumpEnd,
                            c2s::PlayerCommand::HorseInventory => packet::Input::HorseInventory,
                            c2s::PlayerCommand::Elytra => packet::Input::Elytra,
                        }));
                    }
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DropStack,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::DropStack));
                    }
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DropItem,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::DropItem));
                    }
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::ItemUpdate,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::ItemUpdate));
                    }
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::SwapHands,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::SwapHands));
                    }
                    c2s::PlayPacket::PlayerAction { action, .. } => {}
                    c2s::PlayPacket::PlayerInput {
                        sideways,
                        forward,
                        flags,
                    } => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::Move(
                            sideways.into(),
                            forward.into(),
                        )));

                        if flags.jump() {
                            let _ = n2b.send(N2bEvent::Input(packet::Input::Jump));
                        }
                        if flags.dismount() {
                            let _ = n2b.send(N2bEvent::Input(packet::Input::Dismount));
                        }
                    }
                    c2s::PlayPacket::PlayerInventorySlot { .. } => {}
                    c2s::PlayPacket::RecipeBookState { .. } => {}
                    c2s::PlayPacket::AdvancementCommand(_) => {}
                    c2s::PlayPacket::PlayerHotbarSlot(_) => {}
                    c2s::PlayPacket::PlayerSwingArm(hand) => {
                        let _ = n2b.send(N2bEvent::Input(packet::Input::SwingArm(hand)));
                    }
                    c2s::PlayPacket::InteractBlock {
                        hand,
                        block_pos,
                        block_face,
                        cursor_x,
                        cursor_y,
                        cursor_z,
                        head_buried,
                        ..
                    } => {
                        let _ = n2b.send(N2bEvent::Interact(packet::Interact::Block {
                            hand,
                            block_pos,
                            block_face,
                            cursor_x: cursor_x.into(),
                            cursor_y: cursor_y.into(),
                            cursor_z: cursor_z.into(),
                            head_buried: head_buried.into(),
                        }));
                    }
                    c2s::PlayPacket::InteractItem { hand, .. } => {
                        let _ = n2b.send(N2bEvent::Interact(packet::Interact::Item(hand)));
                    }
                }
            }
        }

        Ok(Action::Continue)
    }

    async fn on_keepalive(
        self,
        s2c: &flume::Sender<bytes::Bytes>,
        command: keepalive::Message,
    ) -> Result<Action, Error> {
        match (command, self) {
            (keepalive::Message::Packet(id), ConnectionState::Configuration) => {
                let _ =
                    s2c.send(s2c::ConfigurationPacket::KeepAlive { id: id.into() }.encode_owned()?);
                Ok(Action::Continue)
            }
            (keepalive::Message::Packet(id), ConnectionState::Play) => {
                let _ = s2c.send(s2c::PlayPacket::NetKeepAlive { id: id.into() }.encode_owned()?);
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
    s2c: flume::Sender<bytes::Bytes>,
    c2s: flume::Receiver<bytes::Bytes>,
) -> Result<(), Error> {
    let mut state = ConnectionState::Handshake;

    loop {
        tokio::select! {
            Ok(command) = b2n.recv_async() => {
                match state.on_b2n(&s2c, command).await? {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => {
                        debug!("{state:?} -> {x:?}");
                        state = x
                    },
                    Action::Continue => {}
                }
            },
            Ok(mut frame) = c2s.recv_async() => {
                match state
                    .on_frame(&mut frame, &s2c, &n2b, &to_ka)
                    .instrument(info_span!("conn_state", ?state))
                    .await?
                {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => {
                        debug!("{state:?} -> {x:?}");
                        state = x
                    },
                    Action::Continue => {}
                }
            }
            Ok(command) = from_ka.recv_async() => {
                match state.on_keepalive(&s2c, command).await? {
                    Action::DropConnection => return Ok(()),
                    Action::NewState(x) => {
                        debug!("{state:?} -> {x:?}");
                        state = x
                    },
                    Action::Continue => {}
                }
            },
        }
    }
}
