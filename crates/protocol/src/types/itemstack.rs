use rjacraft_macro::ProtocolType;
use serde::{Deserialize, Serialize};

use crate::ProtocolType;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ItemStack<Id> {
    pub id: Id,
    #[serde(rename = "PascalCase")]
    pub count: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<valence_nbt::Compound>,
}

#[derive(Debug, ProtocolType)]
pub struct ItemStackProto {
    pub id: super::VarInt<i32>,
    pub count: super::Primitive<u8>,
    pub tag: Option<super::Nbt<valence_nbt::Compound>>,
}

impl From<super::BoolOption<ItemStackProto>> for Option<ItemStack<i32>> {
    fn from(value: super::BoolOption<ItemStackProto>) -> Self {
        value.0.map(|x| ItemStack {
            id: x.id.into(),
            count: x.count.into(),
            tag: x.tag.map(|x| x.0),
        })
    }
}

impl From<Option<ItemStack<i32>>> for super::BoolOption<ItemStackProto> {
    fn from(value: Option<ItemStack<i32>>) -> Self {
        value
            .map(|x| ItemStackProto {
                id: x.id.into(),
                count: x.count.into(),
                tag: x.tag.map(super::Nbt),
            })
            .into()
    }
}
