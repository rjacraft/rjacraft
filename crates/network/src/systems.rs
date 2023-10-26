use bevy_ecs::{event, prelude::*, system};
use rjacraft_macro::*;
use rjacraft_protocol::{packets::*, ProtocolType};
use tracing::*;

use crate::{components::*, events::*, network::*};

pub fn new_peer_system(new_peer_rx: flume::Receiver<NewPeer>) -> impl FnMut(Commands) {
    move |mut commands: Commands| {
        for (addr, b2n, n2b) in new_peer_rx.try_iter() {
            debug!("adding new peer entity");
            commands.add(move |world: &mut World| {
                let entity = world.spawn(Peer { addr, n2b, b2n }).id();
                world.send_event(PeerConnected { peer: entity });
            });
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
    move |mut commands, query, mut pset| {
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
                            s2c::ConfigurationPacket::UpdateTags(
                                vec![
                                    s2c::TagType {
                                        name: id!("block"),
                                        tags: vec![].into(),
                                    },
                                    s2c::TagType {
                                        name: id!("entity_type"),
                                        tags: vec![].into(),
                                    },
                                    s2c::TagType {
                                        name: id!("fluid"),
                                        tags: vec![].into(),
                                    },
                                    s2c::TagType {
                                        name: id!("game_event"),
                                        tags: vec![].into(),
                                    },
                                    s2c::TagType {
                                        name: id!("item"),
                                        tags: vec![].into(),
                                    },
                                ]
                                .into(),
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
                        // todo registries
                        let _ = peer.b2n.send(B2nEvent::ConfigurationPacket(
                            s2c::ConfigurationPacket::FinishConfiguration,
                        ));
                    }
                    N2bEvent::Brand(brand) => {
                        commands.add(SendEvent(ClientBrand {
                            peer: entity,
                            brand,
                        }));
                    }
                    N2bEvent::PlayPacket(_packet) => {}
                }
            }
        }
    }
}

pub fn b2n_event_system(
    mut conf: EventReader<ConfigurationPacketOut>,
    mut play: EventReader<PlayPacketOut>,
    mut drop: EventReader<DropPeer>,
    world: &World,
) {
    for event in conf.into_iter() {
        let peer: &Peer = world.get(event.peer).unwrap();

        let _ = peer
            .b2n
            .send(B2nEvent::ConfigurationPacket(event.packet.clone()));
    }

    for event in play.into_iter() {
        let peer: &Peer = world.get(event.peer).unwrap();

        let _ = peer.b2n.send(B2nEvent::PlayPacket(event.packet.clone()));
    }

    for event in drop.into_iter() {
        let peer: &Peer = world.get(event.peer).unwrap();

        let _ = peer.b2n.send(B2nEvent::Drop);
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
