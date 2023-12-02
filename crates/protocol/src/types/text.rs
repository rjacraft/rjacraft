//! Minecraft's formatted text type

use serde::{Deserialize, Serialize};

use super::Identifier;

fn is_false(&x: &bool) -> bool {
    !x
}

fn one() -> i32 {
    1
}

fn is_one(&x: &i32) -> bool {
    x == 1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Score {
    pub name: String,
    pub objective: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NbtSource {
    Block(String),
    Entity(String),
    Storage(Identifier),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Content {
    Literal {
        text: String,
    },
    Translate {
        translate: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fallback: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        with: Option<Vec<Text>>,
    },
    Score {
        score: Score,
    },
    Keybind {
        keybind: String,
    },
    Selector {
        selector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Box<Text>>,
    },
    Nbt {
        nbt: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Box<Text>>,
        #[serde(default, skip_serializing_if = "is_false")]
        interpret: bool,
        #[serde(flatten)]
        source: NbtSource,
    },
}

/// All of these properties are inheritable. `None` means inherit and `Some` means override.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Identifier>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "value")]
pub enum ClickEvent {
    OpenUrl(url::Url),
    RunCommand(String),
    SuggestCommand(String),
    ChangePage(u32),
    CopyToClipboard(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "action", content = "contents")]
pub enum HoverEvent {
    ShowText(Box<Text>),
    ShowItem {
        id: Identifier,
        #[serde(default = "one", skip_serializing_if = "is_one")]
        count: i32,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "super::nbt::snbt"
        )]
        tag: Option<valence_nbt::Compound>,
    },
    ShowEntity {
        #[serde(rename = "type")]
        kind: Identifier,
        id: uuid::Uuid,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<Box<Text>>,
    },
}

/// We recommend using [`rjacraft_macro::text!`] to construct these.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Text {
    Literal(String),
    Array(Vec<Text>),
    Fancy {
        #[serde(flatten)]
        content: Content,
        #[serde(flatten)]
        style: Style,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        extra: Vec<Text>,
    },
}

pub const JSON_TEXT_LEN: usize = 1 << 18;
pub type JsonText = super::JsonString<JSON_TEXT_LEN, Text>;
