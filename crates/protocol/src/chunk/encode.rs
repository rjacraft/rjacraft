use std::collections::HashMap;

use bytes::BufMut;

use super::*;
use crate::types::*;

fn build_heightmap<const SECTIONS: usize>(
    sections: &[Section<BlockId, SECTION_SIDE_BLOCKS>; SECTIONS],
    predicate: impl Fn(BlockId) -> bool,
) -> Vec<i64> {
    let bits_per_entry = (SECTIONS * SECTION_SIDE_BLOCKS).ilog2() + 1;
    let mut result = vec![0];
    let mut n_bit = 0;

    for x in 0..SECTION_SIDE_BLOCKS {
        for z in 0..SECTION_SIDE_BLOCKS {
            'column: for s in (0..SECTIONS).rev() {
                for y in (0..SECTION_SIDE_BLOCKS).rev() {
                    if n_bit + bits_per_entry > i64::BITS {
                        result.push(0);
                        n_bit = 0;
                    }

                    let id = sections[s][y][z][x];
                    let height = s * SECTION_SIDE_BLOCKS + y;

                    if predicate(id) {
                        *result.last_mut().unwrap() |= (height as i64) << n_bit;
                        n_bit += bits_per_entry;
                        break 'column;
                    }
                }
            }
        }
    }

    result
}

fn encode_paletted<const SIDE: usize, const MIN_BITS: u32>(
    section: &Section<BlockId, SIDE>,
) -> (Primitive<i16>, net_chunk::Paletted) {
    let mut non_zero = 0;
    let mut palette_array = Vec::new();
    // key and value flipped
    let mut palette_lookup = HashMap::new();
    let mut i = 0u32;

    for layer in section {
        for row in layer {
            for &id in row {
                if !palette_lookup.contains_key(&id) {
                    palette_lookup.insert(id, i);
                    palette_array.push(VarInt(id as i32));
                    i += 1;
                }

                if id != 0 {
                    non_zero += 1;
                }
            }
        }
    }

    if i == 1 {
        let value = palette_array.last().unwrap().clone();

        (
            non_zero.into(),
            net_chunk::Paletted {
                bits_per_entry: 0.into(),
                palette: net_chunk::Palette::SingleValue(value),
            },
        )
    } else {
        let mut buffer = bytes::BytesMut::new();

        let bits_per_block = Ord::max(MIN_BITS, u32::ilog2(i) + 1);

        let blocks_per_cell = u64::BITS / bits_per_block;
        let longs = (SIDE * SIDE * SIDE).div_ceil(blocks_per_cell as usize);

        let mut current = 0;
        let mut n_bit = 0;

        for layer in section {
            for row in layer {
                for protocol_id in row {
                    let palette_id = palette_lookup.get(protocol_id).unwrap();

                    if n_bit + bits_per_block > u64::BITS {
                        buffer.put_u64(current);
                        current = 0;
                        n_bit = 0;
                    }

                    if u32::BITS - palette_id.leading_zeros() > bits_per_block {
                        panic!("palette reference too wide {palette_id}");
                    }

                    current |= (*palette_id as u64) << n_bit;
                    n_bit += bits_per_block;
                }
            }
        }

        buffer.put_u64(current);

        (
            non_zero.into(),
            net_chunk::Paletted {
                bits_per_entry: Primitive(bits_per_block as u8),
                palette: net_chunk::Palette::Table {
                    source: palette_array.into(),
                    longs: VarInt(longs as i32),
                    refs: buffer.freeze(),
                },
            },
        )
    }
}

fn encode_light<const SECTIONS: usize>(
    light: &Light<SECTIONS>,
) -> (BitVec<u64>, Vec<net_chunk::SectionLight>) {
    let encode_section = |section: &Section<LightLevel, SECTION_SIDE_BLOCKS>| {
        const LENGTH: usize = SECTION_VOLUME_BLOCKS / 2;
        let mut has_light = false;
        let mut array = [0; LENGTH];
        let mut i = 0;

        for layer in section {
            for row in layer {
                for pair in row.chunks(2) {
                    let (a, b) = (pair[0], pair[1]);

                    if a != 0 || b != 0 {
                        has_light = true;
                    }
                    if a > 15 || b > 15 {
                        panic!("light value overflow");
                    }

                    array[i] = (a << 4) | b;
                    i += 1;
                }
            }
        }

        if has_light {
            Some(net_chunk::SectionLight(VarInt(LENGTH as i32), array))
        } else {
            None
        }
    };

    let mut mask = bitvec![u64, Lsb0; 0; 0];
    let mut arrays = Vec::new();

    if let Some(array) = encode_section(&light.below) {
        mask.push(true);
        arrays.push(array);
    } else {
        mask.push(false);
    }

    for layer in &light.world {
        if let Some(array) = encode_section(layer) {
            mask.push(true);
            arrays.push(array);
        } else {
            mask.push(false);
        }
    }

    if let Some(array) = encode_section(&light.above) {
        mask.push(true);
        arrays.push(array);
    } else {
        mask.push(false);
    }

    (mask, arrays)
}

pub fn to_network<const SECTIONS: usize>(
    column: &super::Column<SECTIONS>,
) -> (
    net_chunk::ColumnHeightmaps,
    net_chunk::ColumnPalettes,
    net_chunk::ColumnLight,
) {
    let (sky_light_mask, sky_light) = encode_light(&column.sky_light);
    let (block_light_mask, block_light) = encode_light(&column.block_light);

    (
        net_chunk::ColumnHeightmaps {
            world_surface: build_heightmap(&column.blockstates, |id| id != 0),
            // todo implement this correctly
            motion_blocking: build_heightmap(&column.blockstates, |id| id != 0),
        },
        net_chunk::ColumnPalettes(
            (0..SECTIONS)
                .map(|n_section| {
                    let (non_air_blocks, blockstates) =
                        encode_paletted::<SECTION_SIDE_BLOCKS, 4>(&column.blockstates[n_section]);
                    let (_, biomes) =
                        encode_paletted::<SECTION_SIDE_BIOMES, 2>(&column.biomes[n_section]);

                    net_chunk::FullPalettes {
                        non_air_blocks,
                        blockstates,
                        biomes,
                    }
                })
                .collect(),
        ),
        net_chunk::ColumnLight {
            sky_light_mask: sky_light_mask.clone(),
            block_light_mask: block_light_mask.clone(),
            no_sky_light_mask: !sky_light_mask,
            no_block_light_mask: !block_light_mask,
            sky_light: sky_light.into(),
            block_light: block_light.into(),
        },
    )
}
