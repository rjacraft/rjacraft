use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use rjacraft_network::*;

#[derive(Default, Resource)]
pub struct EidMap {
    e2id: BTreeMap<Entity, i32>,
    id2e: BTreeMap<i32, Entity>,
}

impl EidMap {
    pub fn eid_of<Id: From<i32>>(&self, entity: &Entity) -> Id {
        self.e2id[entity].into()
    }

    pub fn entity_of(&self, eid: impl Into<i32>) -> Entity {
        self.id2e[&eid.into()]
    }
}

pub fn assign_system(mut eids: ResMut<EidMap>, players: Query<Entity, Added<Play>>) {
    let mut free_eids = 0..;

    for entity in players.iter() {
        // yeah this could be a really nice iterator but borrowck won't let me
        let eid = loop {
            let n = free_eids.next().unwrap();
            if !eids.id2e.contains_key(&n) {
                break n;
            }
        };

        eids.e2id.insert(entity, eid);
        eids.id2e.insert(eid, entity);
    }
}

pub fn free_system(mut eids: ResMut<EidMap>, mut events: EventReader<PeerDisconnected>) {
    for &PeerDisconnected { peer } in events.iter() {
        if let Some(eid) = eids.e2id.remove(&peer) {
            eids.id2e.remove(&eid);
        }
    }
}
