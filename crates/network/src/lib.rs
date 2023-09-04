use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_protocol::packets::*;
use tokio::net;
use tracing::*;

use self::net_thread::{PeerMsgIn, PeerMsgOut};

mod components;
mod events;
mod net_thread;

pub use self::{components::*, events::*};

#[derive(Resource)]
pub struct Runtime(pub tokio::runtime::Runtime);

pub struct NetworkPlugin<Addr, SStatus> {
    pub addr: Addr,
    pub status: SStatus,
    // TODO an auth system
}

impl<Addr, SStatus> Plugin for NetworkPlugin<Addr, SStatus>
where
    Addr: net::ToSocketAddrs + Clone + Send + Sync + 'static,
    SStatus: ReadOnlySystem<In = Entity, Out = rjacraft_protocol::types::StatusObject> + Clone,
{
    fn build(&self, app: &mut App) {
        let (new_peer_tx, new_peer_rx) = flume::unbounded();

        let addr = self.addr.clone();
        let net_thread_system = move |rt: Res<Runtime>| {
            let addr = addr.clone();
            let new_peer_tx = new_peer_tx.clone();

            rt.0.spawn(async move {
                if let Err(e) = net_thread::network_loop(addr.clone(), new_peer_tx.clone()).await {
                    error!("network thread crashed: {e}");
                }
            });
        };

        let new_peer_system = move |mut commands: Commands| {
            for (addr, msg_in, msg_out) in new_peer_rx.try_iter() {
                debug!("adding new peer entity");
                commands.add(move |world: &mut World| {
                    let entity = world
                        .spawn(Peer {
                            addr,
                            msg_in,
                            msg_out,
                        })
                        .id();
                    world.send_event(PeerConnected { peer: entity });
                });
            }
        };

        let mut status = self.status.clone();
        status.initialize(&mut app.world);

        let event_tx_system =
            move |mut commands: Commands, world: &World, peers: Query<(Entity, &Peer)>| {
                for (entity, peer) in peers.iter() {
                    for msg in peer.msg_out.try_iter() {
                        match msg {
                            PeerMsgOut::Disconnected => commands.add(move |world: &mut World| {
                                // gets deleted later on
                                world.send_event(PeerDisconnected { peer: entity });
                            }),
                            PeerMsgOut::HandshakeComplete(
                                protocol_version,
                                server_address,
                                server_port,
                            ) => {
                                commands.spawn(Handshaken {
                                    protocol_version,
                                    server_address,
                                    server_port,
                                });
                            }
                            PeerMsgOut::NeedStatus => {
                                peer.msg_in
                                    .send(PeerMsgIn::StatusPacket(cb::StatusPacket::Response(
                                        cb::status::Response {
                                            response: status.run_readonly(entity, world).into(),
                                        },
                                    )))
                                    .unwrap();
                            }
                        }
                    }
                }
            };

        app.add_event::<PeerConnected>()
            .add_event::<PeerDisconnected>()
            .add_event::<DropPeer>()
            .add_event::<ConfigurationPacketOut>()
            // .add_event::<PlayPacketIn>()
            .add_event::<PlayPacketOut>()
            .add_systems(PostStartup, net_thread_system)
            .add_systems(PreUpdate, new_peer_system)
            .add_systems(Update, (event_tx_system, drop_system, event_rx_sysetm))
            .add_systems(PostUpdate, delete_disconnects_system);
    }
}

fn event_rx_sysetm(
    mut conf: EventReader<ConfigurationPacketOut>,
    mut play: EventReader<PlayPacketOut>,
    world: &World,
) {
    for event in conf.into_iter() {
        let peer = world.get::<Peer>(event.peer).unwrap();
        peer.msg_in
            .send(net_thread::PeerMsgIn::ConfigurationPacket(
                event.packet.clone(),
            ))
            .unwrap();
    }

    for event in play.into_iter() {
        let peer = world.get::<Peer>(event.peer).unwrap();
        peer.msg_in
            .send(net_thread::PeerMsgIn::PlayPacket(event.packet.clone()))
            .unwrap();
    }
}

fn delete_disconnects_system(mut events: EventReader<PeerDisconnected>, mut commands: Commands) {
    for event in events.into_iter() {
        debug!("despawning the peer entity");
        commands.entity(event.peer).despawn();
    }
}

fn drop_system(mut events: EventReader<DropPeer>, world: &World) {
    for event in events.into_iter() {
        let peer = world.get::<Peer>(event.peer).unwrap();
        peer.msg_in.send(net_thread::PeerMsgIn::Drop).unwrap();
    }
}
