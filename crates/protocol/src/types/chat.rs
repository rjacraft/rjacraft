//! Minecraft's formatted chat type

use serde::{Deserialize, Serialize};

fn is_false(x: &bool) -> bool {
    !x
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Attrs {
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub underlined: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub strikethrough: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub obfuscated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<super::Identifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chat {
    pub text: String,
    #[serde(flatten)]
    pub attrs: Attrs,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<Box<Chat>>,
}
