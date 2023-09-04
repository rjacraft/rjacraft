//! Minecraft's server status object

use serde::{Deserialize, Serialize};

use super::Chat;

fn is_false(x: &bool) -> bool {
    !x
}

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
pub struct ServerStatus {
    pub version: Version,
    pub players: Players,
    pub description: Chat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub enforces_secure_chat: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub previews_chat: bool,
}
