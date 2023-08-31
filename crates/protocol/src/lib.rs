extern crate core;

pub use io::LengthInferredVecU8;
pub use io::LengthPrefixedVec;
pub use io::ShortPrefixedVec;
pub use io::VarIntPrefixedVec;
pub use io::{Decoder, Encoder};
pub use var_int::VarInt;

pub mod io;
pub mod packets;
pub mod var_int;
