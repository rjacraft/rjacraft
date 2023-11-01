use bytes::Buf;
use rjacraft_macro::ProtocolType;
use valence_nbt::*;

use crate::{chunk::SECTION_VOLUME_BLOCKS, error, types::*, ProtocolType};

/// Cheap to clone!
#[derive(Debug, Clone)]
pub struct ColumnHeightmaps {
    pub world_surface: Vec<i64>,
    pub motion_blocking: Vec<i64>,
}

impl nbt::AsCompound for ColumnHeightmaps {
    type DecodeError = error::StringError;
    type EncodeError = error::Infallible;

    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError> {
        Ok(compound! {
            "WORLD_SURFACE" => Value::LongArray(self.world_surface.clone()),
            "MOTION_BLOCKING" => Value::LongArray(self.world_surface.clone()),
        })
    }
}

/// Cheap to clone!
#[derive(Debug, Clone)]
pub enum Palette {
    SingleValue(VarInt<i32>),
    Table {
        source: LenVec<VarInt<i32>>,
        longs: VarInt<i32>,
        refs: bytes::Bytes,
    },
    None {
        longs: VarInt<i32>,
        ids: bytes::Bytes,
    },
}

/// Cheap to clone!
#[derive(Debug, Clone)]
pub struct Paletted {
    pub bits_per_entry: Primitive<u8>,
    pub palette: Palette,
}

impl ProtocolType for Paletted {
    type DecodeError = error::Infallible;
    type EncodeError = len_vec::EncodeError<error::Infallible>;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        self.bits_per_entry.encode(buffer)?;
        match &self.palette {
            Palette::SingleValue(id) => {
                id.encode(buffer)?;
                VarInt(0i32).encode(buffer)?;
            }
            Palette::Table {
                source,
                longs,
                refs,
            } => {
                source.encode(buffer)?;
                longs.encode(buffer)?;
                buffer.put(refs.chunk());
            }
            Palette::None { longs, ids } => {
                longs.encode(buffer)?;
                buffer.put(ids.chunk());
            }
        }

        Ok(())
    }
}

/// Cheap to clone!
#[derive(Debug, Clone, ProtocolType)]
pub struct FullPalettes {
    pub non_air_blocks: Primitive<i16>,
    pub blockstates: Paletted,
    pub biomes: Paletted,
}

/// Cheap to clone!
#[derive(Debug, Clone)]
pub struct ColumnPalettes<P = FullPalettes>(pub Vec<P>);

impl<P: ProtocolType> ProtocolType for ColumnPalettes<P> {
    type DecodeError = error::Infallible;
    type EncodeError = P::EncodeError;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        let mut buf_ahead = bytes::BytesMut::new();

        for section in &self.0 {
            section.encode(&mut buf_ahead)?;
        }

        VarInt(buf_ahead.len() as i32).encode(buffer).ok();
        buffer.put(buf_ahead);

        Ok(())
    }
}

/// Cheap to clone!
#[derive(Debug, Clone, ProtocolType)]
pub struct SectionLight(pub VarInt<i32>, pub [u8; SECTION_VOLUME_BLOCKS / 2]);

/// Cheap to clone!
#[derive(Debug, Clone, ProtocolType)]
pub struct ColumnLight {
    pub sky_light_mask: BitVec<u64>,
    pub block_light_mask: BitVec<u64>,
    pub no_sky_light_mask: BitVec<u64>,
    pub no_block_light_mask: BitVec<u64>,
    pub sky_light: LenVec<SectionLight>,
    pub block_light: LenVec<SectionLight>,
}
