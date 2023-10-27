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
mod traced_error;

pub mod prebuilt_registries;

pub use self::{components::*, events::*, systems::n2b_system};

#[derive(Resource)]
pub struct Runtime(pub tokio::runtime::Runtime);

#[derive(Resource)]
pub struct Registries(pub s2c::RegistryData);
#[derive(Resource)]
pub struct Tags(pub Vec<s2c::TagType>);

pub struct UserSystems<Status, Auth, Brand> {
    pub status: Status,
    pub authenticate: Auth,
    pub brand: Brand,
}

pub enum AuthOutcome {
    Success(String, uuid::Uuid, Vec<s2c::ProfileProperty>),
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
                    error!("Network thread crashed:\n{}", traced_error::TracedError(e));
                }
            });
        };

        app.add_event::<PeerDisconnected>()
            .add_event::<ClientBrand>()
            .add_event::<ChatMessageSent>()
            .add_systems(PostStartup, net_thread_system)
            .add_systems(
                PreUpdate,
                (
                    systems::new_peer_system(new_peer_rx),
                    self.n2b_system.clone(),
                )
                    .chain(),
            )
            .add_systems(PostUpdate, systems::delete_disconnects_system);
    }
}
