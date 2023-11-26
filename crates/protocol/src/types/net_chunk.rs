//! The network representation of chunk data. For a convenient memory representation, see
//! [`crate::chunk`].

use std::fmt;

use bytes::Buf;
use rjacraft_macro::ProtocolType;
use valence_nbt::*;

use crate::{chunk::SECTION_VOLUME_BLOCKS, error, types::*, ProtocolType};

/// Cheap to clone!
#[derive(Clone)]
pub struct ColumnHeightmaps {
    pub world_surface: Vec<i64>,
    pub motion_blocking: Vec<i64>,
}

impl nbt::AsCompound for ColumnHeightmaps {
    type DecodeError = error::StringError;
    type EncodeError = error::Infallible;

    fn from_nbt(_compound: valence_nbt::Compound) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError> {
        Ok(compound! {
            "WORLD_SURFACE" => Value::LongArray(self.world_surface.clone()),
            "MOTION_BLOCKING" => Value::LongArray(self.world_surface.clone()),
        })
    }
}

impl fmt::Debug for ColumnHeightmaps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColumnHeightmaps")
            .field(
                "world_surface",
                &format_args!("[... {} elements]", self.world_surface.len()),
            )
            .field(
                "motion_blocking",
                &format_args!("[... {} elements]", self.motion_blocking.len()),
            )
            .finish()
    }
}

/// Cheap to clone!
#[derive(Clone)]
pub enum Paletted {
    SingleValue(VarInt<i32>),
    Lut {
        bits_per_value: Primitive<i8>,
        table: LenVec<VarInt<i32>>,
        longs: VarInt<i32>,
        values: bytes::Bytes,
    },
    Array {
        bits_per_value: Primitive<i8>,
        longs: VarInt<i32>,
        values: bytes::Bytes,
    },
}

impl ProtocolType for Paletted {
    type DecodeError = error::Infallible;
    type EncodeError = len_vec::EncodeError<error::Infallible>;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        match &self {
            Paletted::SingleValue(id) => {
                Primitive(0i8).encode(buffer)?;
                id.encode(buffer)?;
                VarInt(0i32).encode(buffer)?;
            }
            Paletted::Lut {
                bits_per_value,
                table,
                longs,
                values,
            } => {
                bits_per_value.encode(buffer)?;
                table.encode(buffer)?;
                longs.encode(buffer)?;
                buffer.put(values.chunk());
            }
            Paletted::Array {
                bits_per_value,
                longs,
                values,
            } => {
                bits_per_value.encode(buffer)?;
                longs.encode(buffer)?;
                buffer.put(values.chunk());
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Paletted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Paletted::SingleValue(v) => write!(f, "SingleValue({})", v.0),
            Paletted::Lut {
                bits_per_value,
                table,
                longs,
                ..
            } => f
                .debug_struct("Lut")
                .field("bits_per_value", bits_per_value)
                .field("table", &table.0)
                .field("longs", &format_args!("[... {} elements]", longs.0))
                .finish(),
            Paletted::Array {
                bits_per_value,
                longs,
                ..
            } => f
                .debug_struct("Array")
                .field("bits_per_value", bits_per_value)
                .field("longs", &format_args!("[... {} elements]", longs.0))
                .finish(),
        }
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
#[derive(Clone, ProtocolType)]
pub struct SectionLight(pub VarInt<i32>, pub [u8; SECTION_VOLUME_BLOCKS / 2]);

impl fmt::Debug for SectionLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} light values", self.0 .0)
    }
}

/// Cheap to clone!
#[derive(Clone, ProtocolType)]
pub struct ColumnLight {
    pub sky_light_mask: BitVec<u64>,
    pub block_light_mask: BitVec<u64>,
    pub no_sky_light_mask: BitVec<u64>,
    pub no_block_light_mask: BitVec<u64>,
    pub sky_light: LenVec<SectionLight>,
    pub block_light: LenVec<SectionLight>,
}

impl fmt::Debug for ColumnLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColumnLight")
            .field("sky_light_mask", &format_args!("{}", self.sky_light_mask))
            .field(
                "block_light_mask",
                &format_args!("{}", self.block_light_mask),
            )
            .field(
                "no_sky_light_mask",
                &format_args!("{}", self.no_sky_light_mask),
            )
            .field(
                "no_block_light_mask",
                &format_args!("{}", self.no_block_light_mask),
            )
            .field("sky_light", &self.sky_light)
            .field("block_light", &self.block_light)
            .finish()
    }
}
