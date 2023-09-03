//! Codec logic for most packet field types

pub mod bool_option;
pub mod chat;
pub mod identifier;
pub mod json_string;
pub mod len_string;
pub mod len_vec;
pub mod primitive;
pub mod remaining_byte_array;
pub mod status_object;
pub mod uuid;
pub mod varint;
pub mod version;

pub use bool_option::BoolOption;
pub use chat::Chat;
pub use identifier::Identifier;
pub use json_string::JsonString;
pub use len_string::LenString;
pub use len_vec::LenVec;
pub use primitive::Primitive;
pub use remaining_byte_array::RemainingByteArray;
pub use status_object::StatusObject;
pub use varint::VarInt;

// TODO VarLong
