//! Minecraft's server status object

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Version {
    pub name: String,
    pub protocol: crate::ProtocolVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SamplePlayer {
    pub name: String,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Players {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<SamplePlayer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "camelCase")]
pub struct StatusObject {
    pub version: Version,
    pub players: Players,
    pub description: super::Chat,
    #[serde(default)]
    pub favicon: Option<String>,
    #[serde(default)]
    pub enforces_secure_chat: bool,
    #[serde(default)]
    pub previews_chat: bool,
}
