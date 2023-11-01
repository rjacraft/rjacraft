//! Odd position types

use bitfield_struct::bitfield;
use rjacraft_macro::ProtocolType;

use super::Primitive;
use crate::{error, ProtocolType};

#[derive(ProtocolType)]
#[bitfield(u64)]
pub struct BlockPos {
    #[bits(26)]
    pub x: i32,
    #[bits(26)]
    pub z: i32,
    #[bits(12)]
    pub y: u16,
}
