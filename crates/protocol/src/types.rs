//! Codec logic for most packet field types

pub mod bool_option;
pub mod chat;
pub mod identifier;
pub mod json_string;
pub mod len_string;
pub mod len_vec;
pub mod primitive;
pub mod remaining_bytes;
pub mod server_status;
pub mod uuid;
pub mod varint;
pub mod version;

pub use self::{
    bool_option::BoolOption,
    chat::{Chat, JsonChat},
    identifier::Identifier,
    json_string::JsonString,
    len_string::LenString,
    len_vec::LenVec,
    primitive::Primitive,
    remaining_bytes::RemainingBytes,
    server_status::ServerStatus,
    varint::VarInt,
};

// TODO VarLong