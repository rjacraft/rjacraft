extern crate core;

pub use io::{
    Decoder, Encoder, LengthInferredVecU8, LengthPrefixedVec, ShortPrefixedVec, VarIntPrefixedVec,
};
pub use var_int::VarInt;

pub mod io;
pub mod packets;
pub mod var_int;
