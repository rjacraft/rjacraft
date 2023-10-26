use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_protocol::{
    packets::*,
    types::{self, Chat},
};
use tokio::net;
use tracing::*;

use self::network::*;

mod components;
mod events;
mod network;
mod systems;

pub use self::{components::*, events::*, systems::n2b_system};

#[derive(Resource)]
pub struct Runtime(pub tokio::runtime::Runtime);

pub struct UserSystems<Status, Auth, Brand> {
    pub status: Status,
    pub authenticate: Auth,
    pub brand: Brand,
}

pub enum AuthOutcome {
    Success(String, uuid::Uuid, Vec<s2c::login::LoginSuccessProperty>),
    Fail(Chat),
}

pub type BrandString = types::LenString<128>;

pub struct NetworkPlugin<A, S> {
    pub addr: A,
    pub n2b_system: S,
}

impl<A, S> Plugin for NetworkPlugin<A, S>
where
    Self: Send + Sync + 'static,
    A: net::ToSocketAddrs + Clone + Send + Sync,
    S: System<In = (), Out = ()> + Clone,
{
    fn build(&self, app: &mut App) {
        let (new_peer_tx, new_peer_rx) = flume::unbounded();

        let addr = self.addr.clone();
        let net_thread_system = move |rt: Res<Runtime>| {
            let addr = addr.clone();
            let new_peer_tx = new_peer_tx.clone();

            rt.0.spawn(async move {
                if let Err(e) = network_loop(addr.clone(), new_peer_tx.clone()).await {
                    error!("network thread crashed: {e}");
                }
            });
        };

        app.add_event::<PeerConnected>()
            .add_event::<PeerDisconnected>()
            .add_event::<DropPeer>()
            .add_event::<ConfigurationPacketOut>()
            .add_event::<PlayPacketIn>()
            .add_event::<PlayPacketOut>()
            .add_event::<ClientBrand>()
            .add_systems(PostStartup, net_thread_system)
            .add_systems(
                PreUpdate,
                (
                    systems::new_peer_system(new_peer_rx),
                    self.n2b_system.clone(),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    systems::b2n_event_system,
                    systems::delete_disconnects_system,
                ),
            );
    }
}
