use rjacraft_macro::ProtocolType;
use serde::{Deserialize, Serialize};

use crate::ProtocolType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemStack {
    pub id: String,
    #[serde(rename = "PascalCase")]
    pub count: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<valence_nbt::Compound>,
}

#[derive(Debug, Clone, Default, ProtocolType)]
#[variant(super::Primitive<bool>)]
pub enum ItemStackProto {
    #[variant(true)]
    Some {
        id: super::VarInt<i32>,
        count: super::Primitive<u8>,
        tag: Option<super::Nbt<valence_nbt::Compound>>,
    },
    #[default]
    #[variant(false)]
    None,
}
