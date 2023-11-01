use std::{collections::HashMap, ops::Range};

use bevy_ecs::prelude::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*};
use tracing::*;

use crate::{c2s, generator};

pub const MIN_RADIUS: u32 = 3;
pub const MAX_RADIUS: u32 = 32;

#[derive(Component, Debug, PartialEq)]
pub enum ViewFilter {
    Nothing,
    Square {
        center_x: i32,
        center_z: i32,
        radius: u32,
    },
}

impl ViewFilter {
    fn contains(&self, chunk_x: i32, chunk_z: i32) -> bool {
        match self {
            ViewFilter::Nothing => false,
            &ViewFilter::Square {
                center_x,
                center_z,
                radius,
            } => {
                i32::abs_diff(chunk_x, center_x) <= radius
                    && i32::abs_diff(chunk_z, center_z) <= radius
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let (range, dx, dz) = match self {
            ViewFilter::Nothing => (0..0, 0, 0),
            &ViewFilter::Square {
                center_x,
                center_z,
                radius,
            } => (
                Range {
                    start: -(radius as i32),
                    end: radius as i32 + 1,
                },
                center_x,
                center_z,
            ),
        };

        range
            .clone()
            .flat_map(move |x| range.clone().map(move |z| (x + dx, z + dz)))
    }
}

#[derive(Default)]
pub struct ChunkCache {
    generator_output: chunk::Column<16>,
    net_chunks: HashMap<
        (i32, i32),
        (
            net_chunk::ColumnHeightmaps,
            net_chunk::ColumnPalettes,
            net_chunk::ColumnLight,
        ),
    >,
}

impl ChunkCache {
    fn load_player(&mut self, x: i32, z: i32, play: &Play) {
        let (heightmaps, palettes, light) = self.net_chunks.entry((x, z)).or_insert_with(|| {
            self.generator_output = Default::default();
            generator::generate(&mut self.generator_output, x, z);
            chunk::to_network(&self.generator_output)
        });

        play.send_packet(&s2c::PlayPacket::ChunkData {
            chunk_x: x.into(),
            chunk_z: z.into(),
            heightmaps: Nbt(heightmaps.clone()),
            palettes: palettes.clone(),
            block_entities: vec![].into(),
            light: light.clone(),
        })
        .unwrap();
    }
}

pub fn chunk_send_system(
    mut query: Query<
        (
            &Play,
            &c2s::Position,
            Option<&c2s::ClientInfo>,
            &mut ViewFilter,
        ),
        Or<(Changed<c2s::Position>, Changed<c2s::ClientInfo>)>,
    >,
    mut chunk_cache: Local<ChunkCache>,
) {
    for (play, position, client_info, mut filter_old) in query.iter_mut() {
        let center_x = f64::floor(position.x / 16.0) as i32;
        let center_z = f64::floor(position.z / 16.0) as i32;
        let filter_new = ViewFilter::Square {
            center_x,
            center_z,
            // when the client asks for n you give it n + 1
            radius: if let Some(c2s::ClientInfo(info)) = client_info {
                u32::clamp(info.view_distance as u32 + 1, MIN_RADIUS, MAX_RADIUS)
            } else {
                MIN_RADIUS
            },
        };

        if filter_old.as_ref() != &filter_new {
            info!(
                "filter changed from {:?} to {filter_new:?}",
                filter_old.as_ref()
            );

            play.send_packet(&s2c::PlayPacket::ChunkCenter {
                chunk_x: center_x.into(),
                chunk_z: center_z.into(),
            })
            .unwrap();

            for (x, z) in filter_old.iter() {
                if !filter_new.contains(x, z) {
                    play.send_packet(&s2c::PlayPacket::ChunkUnload {
                        chunk_z: z.into(),
                        chunk_x: x.into(),
                    })
                    .unwrap();
                }
            }

            for (x, z) in filter_new.iter() {
                if !filter_old.contains(x, z) {
                    chunk_cache.load_player(x, z, play);
                }
            }

            *filter_old = filter_new;
        }
    }
}

pub fn init_filter_system(players: Query<Entity, Added<Play>>, mut commands: Commands) {
    for entity in players.iter() {
        commands.entity(entity).insert(ViewFilter::Nothing);
    }
}
