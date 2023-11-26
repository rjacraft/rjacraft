use bevy_ecs::prelude::*;
use rjacraft_network::*;
use rjacraft_protocol::{
    entity_properties,
    packets::{c2s, s2c},
    types::*,
};

use crate::eid;

#[derive(Debug, Clone, Copy, Component)]
pub struct Old<T>(pub T);

#[derive(Debug, Clone, Copy, Component)]
pub struct Movement {
    pub pos: (f64, f64, f64),
    pub body_rot: (f32, f32),
    pub head_rot: (f32, f32),
    pub ong: bool,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Input {
    pub sneak: bool,
}

pub fn accept_movement_system(
    mut query: Query<&mut Movement>,
    mut events: EventReader<C2sPacket<packet::Movement>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        match data {
            &packet::Movement::Position(x, y, z) => query.get_mut(*entity).unwrap().pos = (x, y, z),
            &packet::Movement::Rotation(pitch, yaw) => {
                let mut mov = query.get_mut(*entity).unwrap();

                mov.body_rot = (pitch % 360.0, yaw % 360.0);
                mov.head_rot = (pitch % 360.0, yaw % 360.0);
            }
            &packet::Movement::OnGround(value) => query.get_mut(*entity).unwrap().ong = value,
        }
    }
}

pub fn accept_input_system(
    eids: Res<eid::EidMap>,
    mut query: Query<&mut Input>,
    others: Query<(Entity, &Play)>,
    mut events: EventReader<C2sPacket<packet::Input>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        let mut input = query.get_mut(*entity).unwrap();
        let eid = eids.eid_of(entity);

        match data {
            &packet::Input::Sneak(value) => {
                input.sneak = value;

                let packet = s2c::PlayPacket::EntityData {
                    id: eid,
                    values: EntityDataValues(vec![(
                        entity_properties::entity::POSE,
                        if value {
                            entity_data::Pose::Sneaking
                        } else {
                            entity_data::Pose::Standing
                        }
                        .into(),
                    )]),
                };

                for (e_other, play_other) in others.iter() {
                    if *entity != e_other {
                        play_other.send_packet(&packet).unwrap();
                    }
                }
            }
            &packet::Input::SwingArm(hand) => {
                let packet = s2c::PlayPacket::EntityAnimation {
                    id: eid,
                    animation: match hand {
                        c2s::HandRel::Main => s2c::EntityAnimation::SwingMainHand,
                        c2s::HandRel::Offhand => s2c::EntityAnimation::SwingOffhand,
                    },
                };

                for (e_other, play_other) in others.iter() {
                    if *entity != e_other {
                        play_other.send_packet(&packet).unwrap();
                    }
                }
            }
            _ => (),
        }
    }
}

// accepts degrees
fn angle_to_proto(mut angle: f32) -> u8 {
    const PRECISION: f32 = 256.0;

    if angle < 0.0 {
        angle += 360.0;
    }

    (angle / 360.0 * PRECISION) as u8
}

fn pos_to_proto_dpos(new: (f64, f64, f64), old: (f64, f64, f64)) -> Option<(i16, i16, i16)> {
    const PRECISION: f64 = 4096.0;
    const LIMIT: f64 = 8.0;

    if f64::abs(new.0 - old.0) > LIMIT
        || f64::abs(new.1 - old.1) > LIMIT
        || f64::abs(new.2 - old.2) > LIMIT
    {
        None
    } else {
        Some((
            ((new.0 - old.0) * PRECISION) as i16,
            ((new.1 - old.1) * PRECISION) as i16,
            ((new.2 - old.2) * PRECISION) as i16,
        ))
    }
}

pub fn broadcast_movement_system(
    eids: Res<eid::EidMap>,
    others: Query<(Entity, &Play)>,
    mut moving: Query<(Entity, &Movement, &mut Old<Movement>)>,
) {
    for (e_in, mov_new, mut mov_old) in moving.iter_mut() {
        let eid_in = eids.eid_of(&e_in);
        let dpos = pos_to_proto_dpos(mov_new.pos, mov_old.0.pos);

        let whatever_packet = s2c::PlayPacket::EntityPosRotOng {
            id: eid_in,
            x: mov_new.pos.0.into(),
            y: mov_new.pos.1.into(),
            z: mov_new.pos.2.into(),
            yaw: angle_to_proto(mov_new.body_rot.1).into(),
            pitch: angle_to_proto(mov_new.body_rot.0).into(),
            on_ground: mov_new.ong.into(),
        };

        let body_packet = if mov_new.pos != mov_old.0.pos && mov_new.body_rot != mov_old.0.body_rot
        {
            if let Some((dx, dy, dz)) = dpos {
                Some(s2c::PlayPacket::EntityDposRotOng {
                    id: eid_in,
                    dx: dx.into(),
                    dy: dy.into(),
                    dz: dz.into(),
                    yaw: angle_to_proto(mov_new.body_rot.1).into(),
                    pitch: angle_to_proto(mov_new.body_rot.0).into(),
                    on_ground: mov_new.ong.into(),
                })
            } else {
                Some(whatever_packet)
            }
        } else if mov_new.pos != mov_old.0.pos {
            if let Some((dx, dy, dz)) = dpos {
                Some(s2c::PlayPacket::EntityDposOng {
                    id: eid_in,
                    dx: dx.into(),
                    dy: dy.into(),
                    dz: dz.into(),
                    on_ground: mov_new.ong.into(),
                })
            } else {
                Some(whatever_packet)
            }
        } else if mov_new.body_rot != mov_old.0.body_rot {
            Some(s2c::PlayPacket::EntityRotOng {
                id: eid_in,
                yaw: angle_to_proto(mov_new.body_rot.1).into(),
                pitch: angle_to_proto(mov_new.body_rot.0).into(),
                on_ground: mov_new.ong.into(),
            })
        } else {
            None
        };

        let head_packet = bool::then(mov_new.head_rot != mov_old.0.head_rot, || {
            s2c::PlayPacket::EntityHeadYaw {
                id: eid_in,
                yaw: angle_to_proto(mov_new.body_rot.1).into(),
            }
        });

        *mov_old = Old(*mov_new);

        for (e_out, play_out) in others.iter() {
            if e_in != e_out {
                play_out
                    .send_packet_option(body_packet.as_ref())
                    .unwrap()
                    .send_packet_option(head_packet.as_ref())
                    .unwrap();
            }
        }
    }
}

pub fn join_system(
    eids: Res<eid::EidMap>,
    others: Query<(Entity, &Play, &super::Profile, &Movement)>,
    joined: Query<(Entity, &Play, &super::Profile, &Movement), Added<Movement>>,
    mut commands: Commands,
) {
    for (entity, play, profile, movement) in joined.iter() {
        let eid = eids.eid_of(&entity);

        commands
            .entity(entity)
            .insert((Input { sneak: false }, Old(*movement)));

        play.send_packet(&s2c::PlayPacket::PlayerTeleport {
            x: movement.pos.0.into(),
            y: movement.pos.1.into(),
            z: movement.pos.2.into(),
            yaw: movement.body_rot.1.into(),
            pitch: movement.body_rot.0.into(),
            relative: s2c::TeleportRelative::new(),
            id: 0.into(),
        })
        .unwrap();

        for (entity_other, play_other, profile_other, movement_other) in others.iter() {
            let eid_other = eids.eid_of(&entity_other);

            play_other
                .send_packet(&s2c::PlayPacket::ServerPlayerInfo(s2c::PlayerInfoUpdates {
                    players: vec![profile.uuid],
                    profile: Some(vec![s2c::PlayerProfile {
                        username: profile.username.clone().try_into().unwrap(),
                        properties: vec![].into(),
                    }]),
                    gamemode: None,
                    listed: None,
                    ping: None,
                    nickname: None,
                }))
                .unwrap();

            if entity_other != entity {
                play_other
                    .send_packet(&s2c::PlayPacket::ServerPlayerInfo(s2c::PlayerInfoUpdates {
                        players: vec![profile.uuid],
                        profile: Some(vec![s2c::PlayerProfile {
                            username: profile.username.clone().try_into().unwrap(),
                            properties: vec![].into(),
                        }]),
                        gamemode: None,
                        listed: None,
                        ping: None,
                        nickname: None,
                    }))
                    .unwrap()
                    .send_packet(&s2c::PlayPacket::EntitySpawnPlayer {
                        id: eid,
                        uuid: profile.uuid,
                        x: movement.pos.0.into(),
                        y: movement.pos.1.into(),
                        z: movement.pos.2.into(),
                        yaw: angle_to_proto(movement_other.body_rot.1).into(),
                        pitch: angle_to_proto(movement_other.body_rot.0).into(),
                    })
                    .unwrap();

                play.send_packet(&s2c::PlayPacket::ServerPlayerInfo(s2c::PlayerInfoUpdates {
                    players: vec![profile_other.uuid],
                    profile: Some(vec![s2c::PlayerProfile {
                        username: profile_other.username.clone().try_into().unwrap(),
                        properties: vec![].into(),
                    }]),
                    gamemode: None,
                    listed: None,
                    ping: None,
                    nickname: None,
                }))
                .unwrap()
                .send_packet(&s2c::PlayPacket::EntitySpawnPlayer {
                    id: eid_other,
                    uuid: profile_other.uuid,
                    x: movement_other.pos.0.into(),
                    y: movement_other.pos.1.into(),
                    z: movement_other.pos.2.into(),
                    yaw: angle_to_proto(movement_other.body_rot.1).into(),
                    pitch: angle_to_proto(movement_other.body_rot.0).into(),
                })
                .unwrap()
                .send_packet(&s2c::PlayPacket::EntityData {
                    id: eid_other,
                    values: vec![].into(),
                })
                .unwrap();
            }
        }
    }
}

pub fn leave_system(
    eid_map: Res<eid::EidMap>,
    players: Query<(&super::Profile, &Play)>,
    mut events: EventReader<PeerDisconnected>,
) {
    let mut uuids = Vec::new();
    let mut eids = Vec::<VarInt<i32>>::new();

    for &PeerDisconnected { peer } in events.iter() {
        if let Ok((profile, _)) = players.get(peer) {
            uuids.push(profile.uuid);
            eids.push(eid_map.eid_of(&peer))
        }
    }

    let uuid_packet = bool::then(!uuids.is_empty(), || {
        s2c::PlayPacket::ServerPlayerRemove(uuids.into())
    });
    let eid_packet = bool::then(!eids.is_empty(), || {
        s2c::PlayPacket::EntityRemove(eids.into())
    });

    for (_, play) in players.iter() {
        play.send_packet_option(uuid_packet.as_ref())
            .unwrap()
            .send_packet_option(eid_packet.as_ref())
            .unwrap();
    }
}
