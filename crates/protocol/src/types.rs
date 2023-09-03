//! Codec logic for most packet field types

pub mod bool_option;
pub mod json_string;
pub mod length_vec;
pub mod primitive;
pub mod remaining_byte_array;
pub mod string;
pub mod uuid;
pub mod varint;
pub mod version;

pub use bool_option::BoolOption;
pub use json_string::JsonString;
pub use length_vec::LengthVec;
pub use primitive::Primitive;
pub use remaining_byte_array::RemainingByteArray;
pub use varint::VarInt;

// TODO VarLong
