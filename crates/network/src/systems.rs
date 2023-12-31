use std::future;

use bevy_ecs::{event, prelude::*, system};
use rjacraft_macro::*;
use rjacraft_protocol::{packets::*, ProtocolType};
use tracing::*;

use crate::{auth, components::*, events::*, network::*};

pub fn new_peer_system(new_peer_rx: flume::Receiver<NewPeer>) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        for (addr, b2n, n2b) in new_peer_rx.try_iter() {
            debug!("adding new peer entity");
            commands.spawn(Peer { addr, n2b, b2n });
        }
    }
}

struct SendEvent<E>(pub E);

impl<E: event::Event> system::Command for SendEvent<E> {
    fn apply(self, world: &mut World) {
        world.send_event(self.0);
    }
}

pub fn n2b_system<SStatus, MStatus, SAuth, MAuth, FAuth, SBrand, MBrand>(
    mut config: crate::NetworkConfig<SStatus, SAuth, SBrand>,
) -> impl FnMut(
    &World,
    Commands,
    Query<(Entity, &Peer)>,
    Res<crate::Runtime>,
    ParamSet<(SStatus::Param, SAuth::Param, SBrand::Param)>,
) + Clone
where
    SStatus: SystemParamFunction<MStatus, In = Entity, Out = rjacraft_protocol::types::ServerStatus>
        + Clone,
    SAuth: SystemParamFunction<MAuth, In = auth::Handle, Out = FAuth> + Clone,
    FAuth: future::Future<Output = auth::Result> + Send + 'static,
    SBrand: SystemParamFunction<MBrand, In = Entity, Out = Option<crate::BrandString>> + Clone,
{
    move |world, mut commands, query, rt, mut pset| {
        for (entity, peer) in query.iter() {
            for msg in peer.n2b.try_iter() {
                match msg {
                    N2bEvent::Disconnected => {
                        // gets deleted later on
                        commands.add(SendEvent(PeerDisconnected { peer: entity }));
                    }
                    N2bEvent::HandshakeComplete(protocol_version, server_address, server_port) => {
                        commands.spawn(Handshaken {
                            protocol_version,
                            server_address,
                            server_port,
                        });
                    }
                    N2bEvent::NeedStatus => {
                        let _ = peer.b2n.send(B2nEvent::Packet(
                            s2c::StatusPacket::Response(
                                config.status_system.run(entity, pset.p0()).into(),
                            )
                            .to_bytes_expect(),
                        ));
                    }
                    N2bEvent::Authenticate(uuid_in, username_in) => {
                        let future = config.auth_system.run(
                            auth::Handle {
                                peer: entity,
                                uuid: uuid_in,
                                username: username_in,
                                b2n: peer.b2n.clone(),
                            },
                            pset.p1(),
                        );

                        let b2n = peer.b2n.clone();
                        rt.0.spawn(async move {
                            match future.await {
                                Ok((uuid_out, username_out)) => {
                                    let _ =
                                        b2n.send(B2nEvent::LoginSuccess(uuid_out, username_out));
                                }
                                Err(reason) => {
                                    let _ = b2n.send(B2nEvent::Packet(
                                        s2c::LoginPacket::Disconnect {
                                            reason: reason.into(),
                                        }
                                        .to_bytes_expect(),
                                    ));
                                    let _ = b2n.send(B2nEvent::Drop);
                                }
                            };
                        });
                    }
                    N2bEvent::NeedConfig => {
                        let _ = peer.b2n.send(B2nEvent::Packet(
                            s2c::ConfigPacket::RegistryData(
                                world.resource::<crate::Registries>().0.clone(),
                            )
                            .to_bytes_expect(),
                        ));

                        let _ = peer.b2n.send(B2nEvent::Packet(
                            s2c::ConfigPacket::UpdateTags(
                                world.resource::<crate::Tags>().0.clone(),
                            )
                            .to_bytes_expect(),
                        ));

                        if let Some(brand_string) = config.brand_system.run(entity, pset.p2()) {
                            let _ = peer.b2n.send(B2nEvent::Packet(
                                s2c::ConfigPacket::PluginMessage {
                                    channel: id!("brand"),
                                    data: crate::BrandString::to_bytes(&brand_string)
                                        .expect("failed to encode brand string")
                                        .try_into()
                                        .expect("brand string too long"),
                                }
                                .to_bytes_expect(),
                            ));
                        }

                        let _ = peer.b2n.send(B2nEvent::Packet(
                            s2c::ConfigPacket::ToPlayRequest.to_bytes_expect(),
                        ));
                    }
                    N2bEvent::ConfigFinished => {
                        commands.entity(entity).insert(Play {
                            b2n: peer.b2n.clone(),
                        });
                    }
                    N2bEvent::TeleportConfirm(_) => {} // todo
                    N2bEvent::Brand(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::ChatMessage(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Command(content) => commands.add(SendEvent(C2sPacket(
                        entity,
                        packet::Command {
                            tokens: content.split(' ').map(str::to_string).collect(),
                        },
                    ))),
                    N2bEvent::Interact(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Movement(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Input(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::Digging(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::ClientInfo(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Window(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                }
            }
        }
    }
}

pub fn delete_disconnects_system(
    mut events: EventReader<PeerDisconnected>,
    mut commands: Commands,
) {
    for event in events.into_iter() {
        debug!("despawning the peer entity");
        commands.entity(event.peer).despawn();
    }
}
