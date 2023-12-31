use core::fmt;
use std::{hint, ops, sync};

use bevy_ecs::{component, prelude::*, system, world};
use tokio::sync::oneshot;

/// Like any other mutex guard, this struct must be dropped after you're done using it. Preferably,
/// you should use it in small bursts so as to not bring down the tick time. Definitely do not hold
/// this across other awaits.
pub struct WorldGuard {
    world: *mut World,
    finished: sync::Arc<sync::atomic::AtomicBool>,
}

impl ops::Deref for WorldGuard {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.world }
    }
}

impl ops::DerefMut for WorldGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.world }
    }
}

impl Drop for WorldGuard {
    fn drop(&mut self) {
        self.finished.store(true, sync::atomic::Ordering::Release);
    }
}

impl fmt::Debug for WorldGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WorldGuard")
    }
}

#[derive(Resource, Clone)]
pub struct WorldMutex {
    tx: flume::Sender<oneshot::Sender<WorldGuard>>,
    rx: flume::Receiver<oneshot::Sender<WorldGuard>>,
}

impl WorldMutex {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();

        Self { tx, rx }
    }

    pub async fn lock(&self) -> WorldGuard {
        let (tx, rx) = oneshot::channel();
        self.tx.send(tx).unwrap();
        rx.await.expect("channel error")
    }
}

unsafe impl Send for WorldGuard {}

unsafe impl system::SystemParam for WorldMutex {
    type State = ();
    type Item<'world, 'state> = Self;

    fn init_state(_world: &mut World, _system_meta: &mut system::SystemMeta) -> Self::State {
        ()
    }

    unsafe fn get_param<'world, 'state>(
        _state: &'state mut Self::State,
        _system_meta: &system::SystemMeta,
        world: world::unsafe_world_cell::UnsafeWorldCell<'world>,
        _change_tick: component::Tick,
    ) -> Self::Item<'world, 'state> {
        world.get_resource::<Self>().unwrap().clone()
    }
}

pub fn accept_locks(world: &mut World) {
    let mutex: &WorldMutex = world.resource();
    let rx = mutex.rx.clone();

    while let Ok(tx) = rx.try_recv() {
        let finished = sync::Arc::new(sync::atomic::AtomicBool::new(false));

        tx.send(WorldGuard {
            world,
            finished: finished.clone(),
        })
        .expect("channel error");

        while !finished.load(sync::atomic::Ordering::Acquire) {
            hint::spin_loop();
        }
    }
}
