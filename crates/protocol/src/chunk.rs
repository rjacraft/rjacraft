//! This is a crucial optimization: There's the friendly chunk format and there's the network
//! format.
//!
//! - Friendly format: The [`Column`] struct. This is how you'll store chunks in memory for fast
//! and ergonomic access.
//! - Network format: [`NetworkColumn`] and [`NetworkLightColumn`]. These contain the serialized
//! forms of your blocks + some extra data that Minecraft really wants. Converting from friendly to
//! network is a process that gets expensive really quickly, so you are expected to cache these
//! structures before sending them out to players.

mod encode;

pub use encode::to_network;

pub const SECTION_SIDE_BLOCKS: usize = 16;
pub const SECTION_SIDE_BIOMES: usize = 4;

pub const SECTION_VOLUME_BLOCKS: usize =
    SECTION_SIDE_BLOCKS * SECTION_SIDE_BLOCKS * SECTION_SIDE_BLOCKS;
pub const SECTION_VOLUME_BIOME: usize =
    SECTION_SIDE_BIOMES * SECTION_SIDE_BIOMES * SECTION_SIDE_BIOMES;

pub type BlockId = u32;
pub type BiomeId = u32;
pub type LightLevel = u8;

pub type Section<T, const SIDE: usize> = [[[T; SIDE]; SIDE]; SIDE];

#[derive(Debug, Clone, Copy)]
pub struct Light<const SECTIONS: usize> {
    /// Indexed `[y][z][x]`.
    pub below: Section<LightLevel, SECTION_SIDE_BLOCKS>,
    /// Indexed `[section][y][z][x]`.
    pub world: [Section<LightLevel, SECTION_SIDE_BLOCKS>; SECTIONS],
    /// Indexed `[y][z][x]`.
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

/// Be aware that this structure can create stack overflows if used too much -- a column of height
/// 384 takes up half a megabyte of space.
#[derive(Debug, Clone, Copy)]
pub struct Column<const SECTIONS: usize> {
    /// Indexed `[section][y][z][x]`.
    pub blockstates: [Section<BlockId, SECTION_SIDE_BLOCKS>; SECTIONS],
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
