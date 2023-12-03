use rjacraft_macro::text;
use rjacraft_protocol::{
    frame,
    packets::{c2s, s2c},
    types::{self, *},
    ProtocolType,
};
use tracing::*;

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode packet")]
    DecodingHandshake(#[from] c2s::HandshakePacketDecodeError),
    #[error("Failed to decode packet")]
    DecodingStatus(#[from] c2s::StatusPacketDecodeError),
    #[error("Failed to decode packet")]
    DecodingLogin(#[from] c2s::LoginPacketDecodeError),
    #[error("Failed to decode packet")]
    DecodingConfig(#[from] c2s::ConfigPacketDecodeError),
    #[error("Failed to decode brand")]
    DecodingBrand(#[source] types::len_string::DecodeError<128>),
    #[error("Failed to decode packet")]
    DecodingPlay(#[from] c2s::PlayPacketDecodeError),
    #[error("Wrong protocol version: {0:?}")]
    WrongVersion(rjacraft_protocol::ProtocolVersion),
    #[error("Got login ack despite not being logged in")]
    FakeLoginAck,
    #[error("Couldn't write a raw packet")]
    Writer(#[from] frame::WriterError),
    #[error("Couldn't send a message to Bevy")]
    SendN2b(#[from] flume::SendError<N2bEvent>),
    #[error("Couldn't send a message to keep alive")]
    SendKa(#[from] flume::SendError<i64>),
}

pub enum Action {
    DropConnection,
    NewState(ConnectionState),
    Continue,
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login { completed: bool },
    Config,
    Play,
}

impl ConnectionState {
    pub async fn on_frame<S: io::AsyncWrite + Unpin + Send>(
        self,
        mut frame: bytes::Bytes,
        writer: &mut frame::Writer<S>,
        n2b: &flume::Sender<N2bEvent>,
        to_ka: &flume::Sender<i64>,
    ) -> Result<Action, Error> {
        // Handshake handles the handshake sequence
        // Status hands the response off to Bevy
        // Login and config handle low-level details and the switching
        // Play just translates the packets into neat Bevy events

        match self {
            ConnectionState::Handshake => {
                let c2s::HandshakePacket::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                } = c2s::HandshakePacket::decode(&mut frame)?;

                debug!("handshake complete, {}", protocol_version);

                n2b.send(N2bEvent::HandshakeComplete(
                    protocol_version,
                    server_address.into(),
                    server_port.into(),
                ))?;

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
                            writer
                                .write_frame(
                                    &s2c::LoginPacket::Disconnect {
                                        reason: JsonString(text!("Incompatible game version")),
                                    }
                                    .to_bytes_expect(),
                                )
                                .await?;

                            Err(Error::WrongVersion(protocol_version))?
                        }
                    }
                }
            }
            ConnectionState::Status => {
                let packet = c2s::StatusPacket::decode(&mut frame)?;

                match packet {
                    c2s::StatusPacket::Ping { payload } => {
                        writer
                            .write_frame(&s2c::StatusPacket::Pong { payload }.to_bytes_expect())
                            .await?;
                        return Ok(Action::DropConnection);
                    }
                    c2s::StatusPacket::Request => n2b.send(N2bEvent::NeedStatus)?,
                }
            }
            ConnectionState::Login { completed } => {
                let packet = c2s::LoginPacket::decode(&mut frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::LoginPacket::LoginStart { username, uuid } => {
                        n2b.send(N2bEvent::Authenticate(username, uuid))?
                    }
                    c2s::LoginPacket::EncryptionResponse { .. } => todo!(),
                    c2s::LoginPacket::LoginPluginResponse { .. } => todo!(),
                    c2s::LoginPacket::ToConfig => {
                        if completed {
                            n2b.send(N2bEvent::NeedConfig)?;
                            return Ok(Action::NewState(ConnectionState::Config));
                        } else {
                            Err(Error::FakeLoginAck)?
                        }
                    }
                }
            }
            ConnectionState::Config => {
                let packet = c2s::ConfigPacket::decode(&mut frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::ConfigPacket::PluginMessage { channel, data } => {
                        let mut data: bytes::Bytes = data.into();

                        if let ("minecraft", "brand") = channel.parts() {
                            let brand = types::LenString::<128>::decode(&mut data)
                                .map_err(Error::DecodingBrand)?;

                            n2b.send(N2bEvent::Brand(packet::ClientBrand {
                                brand: brand.into(),
                            }))?;
                        }
                    }
                    c2s::ConfigPacket::ToPlay => {
                        n2b.send(N2bEvent::ConfigFinished)?;

                        return Ok(Action::NewState(ConnectionState::Play));
                    }
                    c2s::ConfigPacket::KeepAlive { id } => to_ka.send(id.into())?,
                    c2s::ConfigPacket::Pong { .. } => todo!(),
                    c2s::ConfigPacket::ResourcePack { .. } => todo!(),
                }
            }
            ConnectionState::Play => {
                let packet = c2s::PlayPacket::decode(&mut frame)?;

                trace!("{packet:?}");

                match packet {
                    c2s::PlayPacket::PlayerTeleportConfirm { id } => {
                        n2b.send(N2bEvent::TeleportConfirm(id.into()))?
                    }
                    c2s::PlayPacket::ChatCommand { command, .. } => {
                        n2b.send(N2bEvent::Command(command.into()))?
                    }
                    c2s::PlayPacket::ChatMessage { message, .. } => {
                        n2b.send(N2bEvent::ChatMessage(packet::ChatMessage {
                            content: message.into(),
                        }))?
                    }
                    c2s::PlayPacket::NetKeepAlive { id } => to_ka.send(id.into())?,
                    c2s::PlayPacket::ClientCommand(c2s::ClientCommand::Respawn) => {
                        n2b.send(N2bEvent::Input(packet::Input::Respawn))?
                    }
                    c2s::PlayPacket::ClientCommand(c2s::ClientCommand::StatsRequest) => {
                        n2b.send(N2bEvent::Input(packet::Input::StatsRequest))?
                    }
                    c2s::PlayPacket::ClientInfo {
                        locale,
                        view_distance,
                        chat_mode,
                        chat_colors,
                        skin_parts,
                        main_hand,
                        text_filtering,
                        show_on_listings,
                    } => n2b.send(N2bEvent::ClientInfo(packet::ClientInfo {
                        locale: locale.into(),
                        view_distance: view_distance.into(),
                        chat_mode,
                        chat_colors: chat_colors.into(),
                        main_hand,
                        skin_parts: skin_parts.into(),
                        text_filtering: text_filtering.into(),
                        show_on_listings: show_on_listings.into(),
                    }))?,
                    c2s::PlayPacket::ContainerButton { sync_id, button_id } => {
                        n2b.send(N2bEvent::Window(packet::Window::ContainerButton {
                            sync_id: sync_id.into(),
                            button: button_id.into(),
                        }))?
                    }
                    c2s::PlayPacket::ContainerClick {
                        sync_id,
                        slot,
                        button,
                        mode,
                        new_slots,
                        carried_item,
                        ..
                    } => n2b.send(N2bEvent::Window(packet::Window::ContainerClick {
                        sync_id: sync_id.into(),
                        slot: slot.into(),
                        button: button.into(),
                        mode: mode.0 as u32,
                        new_slots: new_slots
                            .0
                            .into_iter()
                            .map(|(n, s)| (n.into(), s.into()))
                            .collect(),
                        carried_item: carried_item.into(),
                    }))?,
                    c2s::PlayPacket::ContainerClose { sync_id } => {
                        n2b.send(N2bEvent::Window(packet::Window::ContainerClose {
                            sync_id: sync_id.into(),
                        }))?
                    }
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::Interact { hand },
                        ..
                    } => n2b.send(N2bEvent::Interact(packet::Interact::TouchEntity {
                        entity: entity.0,
                        at: None,
                        hand,
                    }))?,
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::Attack,
                        ..
                    } => n2b.send(N2bEvent::Interact(packet::Interact::AttackEntity(entity.0)))?,
                    c2s::PlayPacket::InteractEntity {
                        entity,
                        kind: c2s::InteractKind::InteractAt { x, y, z, hand },
                        ..
                    } => n2b.send(N2bEvent::Interact(packet::Interact::TouchEntity {
                        entity: entity.0,
                        at: Some((x.into(), y.into(), z.into())),
                        hand,
                    }))?,
                    c2s::PlayPacket::PlayerPosOng { x, y, z, on_ground } => {
                        n2b.send(N2bEvent::Movement(packet::Movement::Position(
                            x.into(),
                            y.into(),
                            z.into(),
                        )))?;
                        n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )))?;
                    }
                    c2s::PlayPacket::PlayerPosRotOng {
                        x,
                        y,
                        z,
                        yaw,
                        pitch,
                        on_ground,
                    } => {
                        n2b.send(N2bEvent::Movement(packet::Movement::Position(
                            x.into(),
                            y.into(),
                            z.into(),
                        )))?;
                        n2b.send(N2bEvent::Movement(packet::Movement::Rotation(
                            pitch.into(),
                            yaw.into(),
                        )))?;
                        n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )))?;
                    }
                    c2s::PlayPacket::PlayerRotOng {
                        yaw,
                        pitch,
                        on_ground,
                    } => {
                        n2b.send(N2bEvent::Movement(packet::Movement::Rotation(
                            pitch.into(),
                            yaw.into(),
                        )))?;
                        n2b.send(N2bEvent::Movement(packet::Movement::OnGround(
                            on_ground.into(),
                        )))?;
                    }
                    c2s::PlayPacket::PlayerOnGround(on_ground) => n2b.send(N2bEvent::Movement(
                        packet::Movement::OnGround(on_ground.into()),
                    ))?,
                    c2s::PlayPacket::PlayerAbilties(abilities) => {
                        n2b.send(N2bEvent::Input(packet::Input::Flight(abilities.flying())))?
                    }
                    c2s::PlayPacket::PlayerCommand { action, extra, .. } => {
                        n2b.send(N2bEvent::Input(match action {
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
                        }))?
                    }
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DigStart,
                        position,
                        face,
                        ..
                    } => n2b.send(N2bEvent::Digging(packet::Digging::Start(position, face)))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DigCancel,
                        position,
                        face,
                        ..
                    } => n2b.send(N2bEvent::Digging(packet::Digging::Cancel(position, face)))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DigFinish,
                        position,
                        face,
                        ..
                    } => n2b.send(N2bEvent::Digging(packet::Digging::Finish(position, face)))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DropStack,
                        ..
                    } => n2b.send(N2bEvent::Input(packet::Input::DropStack))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::DropItem,
                        ..
                    } => n2b.send(N2bEvent::Input(packet::Input::DropItem))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::ItemUpdate,
                        ..
                    } => n2b.send(N2bEvent::Input(packet::Input::ItemUpdate))?,
                    c2s::PlayPacket::PlayerAction {
                        action: c2s::PlayerAction::SwapHands,
                        ..
                    } => n2b.send(N2bEvent::Input(packet::Input::SwapHands))?,
                    c2s::PlayPacket::PlayerInput {
                        sideways,
                        forward,
                        flags,
                    } => {
                        n2b.send(N2bEvent::Input(packet::Input::Move(
                            sideways.into(),
                            forward.into(),
                        )))?;

                        if flags.jump() {
                            n2b.send(N2bEvent::Input(packet::Input::Jump))?
                        }
                        if flags.dismount() {
                            n2b.send(N2bEvent::Input(packet::Input::Dismount))?
                        }
                    }
                    c2s::PlayPacket::PlayerInventorySlot { slot, stack } => n2b.send(
                        N2bEvent::Window(packet::Window::InventorySlot(slot.into(), stack.into())),
                    )?,
                    c2s::PlayPacket::RecipeBookState { book, open, filter } => {
                        n2b.send(N2bEvent::Window(packet::Window::RecipeBook {
                            book,
                            open: open.into(),
                            filter: filter.into(),
                        }))?
                    }
                    c2s::PlayPacket::AdvancementCommand(c2s::AdvancementCommand::OpenTab(id)) => {
                        n2b.send(N2bEvent::Window(packet::Window::AdvancementsTab(Some(id))))?
                    }
                    c2s::PlayPacket::AdvancementCommand(c2s::AdvancementCommand::CloseScreen) => {
                        n2b.send(N2bEvent::Window(packet::Window::AdvancementsTab(None)))?
                    }
                    c2s::PlayPacket::PlayerHotbarSlot(n) => {
                        n2b.send(N2bEvent::Input(packet::Input::HotbarSlot(n.into())))?
                    }
                    c2s::PlayPacket::PlayerSwingArm(hand) => {
                        n2b.send(N2bEvent::Input(packet::Input::SwingArm(hand)))?
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
                    } => n2b.send(N2bEvent::Interact(packet::Interact::Block {
                        hand,
                        block_pos,
                        block_face,
                        cursor_x: cursor_x.into(),
                        cursor_y: cursor_y.into(),
                        cursor_z: cursor_z.into(),
                        head_buried: head_buried.into(),
                    }))?,
                    c2s::PlayPacket::InteractItem { hand, .. } => {
                        n2b.send(N2bEvent::Interact(packet::Interact::Item(hand)))?
                    }
                }
            }
        }

        Ok(Action::Continue)
    }

    pub fn keepalive_packet(self, id: i64) -> Option<bytes::Bytes> {
        match self {
            ConnectionState::Config => {
                Some(s2c::ConfigPacket::KeepAlive { id: id.into() }.to_bytes_expect())
            }
            ConnectionState::Play => {
                Some(s2c::PlayPacket::NetKeepAlive { id: id.into() }.to_bytes_expect())
            }
            _ => None,
        }
    }
}
