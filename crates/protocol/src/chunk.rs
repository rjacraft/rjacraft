//! The memory representation of chunk data
//!
//! Encoding these is a two-step process, with room for optimization. There's the friendly chunk
//! format and then there's the network format.
//!
//! - Friendly format: The [`Column`] struct. This is how you'll store chunks in memory for fast
//! and ergonomic access.
//! - Network format: Everything in [`crate::types::net_chunk`]. These contain the serialized
//! forms of your blocks + some extra data that Minecraft really wants. Converting from friendly to
//! network is a process that gets expensive really quickly, so you are expected to cache these
//! structures before sending them out to players.

mod encode;

pub use encode::to_network;

/// Amount of blocks within a section's side.
pub const SECTION_SIDE_BLOCKS: usize = 16;
/// Amount of biomes within a section's side.
pub const SECTION_SIDE_BIOMES: usize = 4;

/// Amount of blocks within a section.
pub const SECTION_VOLUME_BLOCKS: usize =
    SECTION_SIDE_BLOCKS * SECTION_SIDE_BLOCKS * SECTION_SIDE_BLOCKS;
/// Amount of biomes within a section.
pub const SECTION_VOLUME_BIOMES: usize =
    SECTION_SIDE_BIOMES * SECTION_SIDE_BIOMES * SECTION_SIDE_BIOMES;

/// A blockstate ID in the global palette.
pub type BlockstateId = u32;
/// A biome ID in the global palette.
pub type BiomeId = u32;
/// A light level between 0 and 15.
pub type LightLevel = u8;

/// Generic accessible contents of a chunk section.
pub type Section<T, const SIDE: usize> = [[[T; SIDE]; SIDE]; SIDE];

/// Accessible contents of a light column.
#[derive(Debug, Clone, Copy)]
pub struct Light<const SECTIONS: usize> {
    /// The -1 section. Indexed `[y][z][x]`.
    pub below: Section<LightLevel, SECTION_SIDE_BLOCKS>,
    /// Indexed `[section][y][z][x]`.
    pub world: [Section<LightLevel, SECTION_SIDE_BLOCKS>; SECTIONS],
    /// The S + 1 section. Indexed `[y][z][x]`.
    pub above: Section<LightLevel, SECTION_SIDE_BLOCKS>,
}

impl<const SECTIONS: usize> Default for Light<SECTIONS> {
    fn default() -> Self {
        Light {
            below: [[[0; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS],
            world: [[[[0; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS];
                SECTIONS],
            above: [[[0; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS],
        }
    }
}

/// Accessible contents of a chunk column.
///
/// Be aware that this structure can cause stack overflows if used too much --- a column of height
/// 384 takes up half a megabyte of space.
#[derive(Debug, Clone, Copy)]
pub struct Column<const SECTIONS: usize> {
    /// Indexed `[section][y][z][x]`.
    pub blockstates: [Section<BlockstateId, SECTION_SIDE_BLOCKS>; SECTIONS],
    /// Indexed `[section][y][z][x]`.
    pub biomes: [Section<BiomeId, SECTION_SIDE_BIOMES>; SECTIONS],
    pub sky_light: Light<SECTIONS>,
    pub block_light: Light<SECTIONS>,
}

impl<const SECTIONS: usize> Default for Column<SECTIONS> {
    fn default() -> Self {
        Column {
            blockstates: [[[[0; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS]; SECTION_SIDE_BLOCKS];
                SECTIONS],
            biomes: [[[[0; SECTION_SIDE_BIOMES]; SECTION_SIDE_BIOMES]; SECTION_SIDE_BIOMES];
                SECTIONS],
            sky_light: Default::default(),
            block_light: Default::default(),
        }
    }
}
