use bevy_ecs::{event, prelude::*, system};
use rjacraft_macro::*;
use rjacraft_protocol::{packets::*, types, ProtocolType};
use tracing::*;

use crate::{components::*, events::*, network::*};

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

pub fn n2b_system<SStatus, MStatus, SAuth, MAuth, SBrand, MBrand>(
    mut systems: crate::UserSystems<SStatus, SAuth, SBrand>,
) -> impl FnMut(
    &World,
    Commands,
    Query<(Entity, &Peer)>,
    ParamSet<(SStatus::Param, SAuth::Param, SBrand::Param)>,
) + Clone
where
    SStatus: SystemParamFunction<MStatus, In = Entity, Out = rjacraft_protocol::types::ServerStatus>
        + Clone,
    SAuth: SystemParamFunction<MAuth, In = (Entity, String, uuid::Uuid), Out = crate::AuthOutcome>
        + Clone,
    SBrand: SystemParamFunction<MBrand, In = Entity, Out = Option<crate::BrandString>> + Clone,
{
    move |world, mut commands, query, mut pset| {
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
                        let _ = peer
                            .b2n
                            .send(B2nEvent::Status(systems.status.run(entity, pset.p0())));
                    }
                    N2bEvent::Authenticate(username_in, uuid_in) => {
                        let outcome = systems
                            .authenticate
                            .run((entity, username_in, uuid_in), pset.p1());

                        match outcome {
                            crate::AuthOutcome::Success(username_out, uuid_out, props_out) => {
                                let _ = peer.b2n.send(B2nEvent::LoginSucceeded);
                                let _ = peer.b2n.send(B2nEvent::LoginPacket(
                                    s2c::LoginPacket::Success {
                                        username: username_out
                                            .try_into()
                                            .expect("authenticated username is too long"),
                                        uuid: uuid_out,
                                        properties: props_out.into(),
                                    },
                                ));
                            }
                            crate::AuthOutcome::Fail(reason) => {
                                let _ = peer.b2n.send(B2nEvent::LoginPacket(
                                    s2c::LoginPacket::Disconnect {
                                        reason: reason.into(),
                                    },
                                ));
                                let _ = peer.b2n.send(B2nEvent::Drop);
                            }
                        };
                    }
                    N2bEvent::NeedConfiguration => {
                        let _ = peer.b2n.send(B2nEvent::ConfigurationPacket(
                            s2c::ConfigurationPacket::RegistryData(types::Nbt(
                                world.resource::<crate::Registries>().0.clone(),
                            )),
                        ));

                        let _ = peer.b2n.send(B2nEvent::ConfigurationPacket(
                            s2c::ConfigurationPacket::UpdateTags(
                                world.resource::<crate::Tags>().0.clone().into(),
                            ),
                        ));

                        if let Some(brand_string) = systems.brand.run(entity, pset.p2()) {
                            let _ = peer
                                .b2n
                                .send(B2nEvent::ConfigurationPacket(
                                    s2c::ConfigurationPacket::PluginMessage {
                                        channel: id!("brand"),
                                        data: crate::BrandString::encode_owned(&brand_string)
                                            .unwrap()
                                            .try_into()
                                            .unwrap(),
                                    },
                                ))
                                .unwrap();
                        }

                        let _ = peer.b2n.send(B2nEvent::ConfigurationPacket(
                            s2c::ConfigurationPacket::FinishConfiguration,
                        ));
                    }
                    N2bEvent::ConfigurationFinished(play_tx) => {
                        commands.entity(entity).insert(Play {
                            tx: play_tx.clone(),
                        });
                    }
                    N2bEvent::Brand(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::Chat(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::Interact(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Movement(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Input(packet) => commands.add(SendEvent(C2sPacket(entity, packet))),
                    N2bEvent::ClientInfo(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
                    N2bEvent::Container(packet) => {
                        commands.add(SendEvent(C2sPacket(entity, packet)))
                    }
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
