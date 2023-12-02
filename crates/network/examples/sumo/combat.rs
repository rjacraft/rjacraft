use bevy_ecs::prelude::*;
use rjacraft_network::*;
use rjacraft_protocol::{packets::s2c, types::*, ProtocolType};
use tracing::info;

use crate::{eid, player_entity};

// accepts m/s
fn velo_to_proto(velo: f64) -> i16 {
    const RATIO: f64 = 400.0;

    (velo * RATIO) as i16
}

const DAMAGE_TYPE_PLAYER: i32 = 31;
const KNOCKBACK_XZ: f64 = 11.0;
const KNOCKBACK_Y: f64 = 7.0;

pub fn accept_interact_system(
    eids: Res<eid::EidMap>,
    players: Query<(&Play, &player_entity::Movement)>,
    mut events: EventReader<C2sPacket<packet::Interact>>,
) {
    for C2sPacket(e_source, data) in events.into_iter() {
        let (_, &mov_source) = players.get(*e_source).unwrap();

        if let &packet::Interact::AttackEntity(eid_taker) = data {
            let (play_taker, _) = players.get(eids.entity_of(eid_taker)).unwrap();

            play_taker.send(
                s2c::PlayPacket::EntityDamageTilt {
                    id: VarInt(eid_taker),
                    yaw: Primitive(mov_source.head_rot.1 - 180.0),
                }
                .to_encoded_expect(),
            );

            let damage_packet = s2c::PlayPacket::EntityDamage {
                taker_id: VarInt(eid_taker),
                damage_type: DAMAGE_TYPE_PLAYER.into(),
                source_id: eids.eid_of(e_source),
                means_id: eids.eid_of(e_source),
                source_pos: None.into(),
            }
            .to_encoded_expect();

            let (sin, cos) = (mov_source.head_rot.1 as f64).to_radians().sin_cos();
            let velo_packet = s2c::PlayPacket::EntityVelocity {
                id: VarInt(eid_taker),
                x: velo_to_proto(KNOCKBACK_XZ * -sin).into(),
                y: velo_to_proto(KNOCKBACK_Y).into(),
                z: velo_to_proto(KNOCKBACK_XZ * cos).into(),
            }
            .to_encoded_expect();

            for (play_other, _) in players.iter() {
                play_other
                    .send(damage_packet.clone())
                    .send(velo_packet.clone());
            }
        }
    }
}

pub fn losing_system(mut players: Query<(&Play, &crate::Profile, &mut player_entity::Movement)>) {
    for (play, profile, mut movement) in players.iter_mut() {
        if movement.pos.1 < 40.0 {
            info!("{} just lost", profile.username);

            movement.pos = (crate::SPAWN_X, crate::SPAWN_Y, crate::SPAWN_Z);

            play.send(
                s2c::PlayPacket::PlayerTeleport {
                    x: movement.pos.0.into(),
                    y: movement.pos.1.into(),
                    z: movement.pos.2.into(),
                    yaw: movement.body_rot.1.into(),
                    pitch: movement.body_rot.0.into(),
                    relative: s2c::TeleportRelative::new(),
                    id: 0.into(),
                }
                .to_encoded_expect(),
            );
        }
    }
}
