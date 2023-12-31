use std::str;

use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;

mod name;

pub use name::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkinModel {
    Classic,
    Slim,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkinMetadata {
    pub model: SkinModel,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkinTexture {
    pub url: url::Url,
    pub metadata: Option<SkinMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Texture {
    pub url: url::Url,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Textures {
    pub skin: Option<SkinTexture>,
    pub cape: Option<Texture>,
    pub elytra: Option<Texture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyValueTextures {
    pub textures: Textures,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue<T> {
    pub timestamp: u64,
    pub profile_id: uuid::Uuid,
    pub profile_name: Name,
    #[serde(default)]
    pub signature_required: bool,
    #[serde(flatten)]
    pub value: T,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "name", rename_all = "lowercase")]
pub enum Property {
    Textures {
        #[serde_as(as = "Base64")]
        value: Vec<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<Base64>")]
        signature: Option<Vec<u8>>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    ForcedNameChange,
    UsingBannedSkin,
    #[serde(other)]
    Other,
}
