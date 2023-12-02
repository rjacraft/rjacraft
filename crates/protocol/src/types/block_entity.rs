use rjacraft_macro::ProtocolType;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::*;
use crate::ProtocolType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dye {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignText {
    pub has_glowing_text: bool,
    pub color: Dye,
    #[serde_as(as = "Vec<serde_with::json::JsonString>")]
    pub messages: Vec<Text>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sign {
    pub is_waxed: bool,
    pub front_text: SignText,
    pub back_text: SignText,
}

/// No fields are used by the client
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chest {}

#[derive(Debug, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum BlockEntity {
    #[variant(7)]
    Sign(Nbt<Sign>),
    #[variant(177)]
    Chest(Nbt<Chest>),
}
