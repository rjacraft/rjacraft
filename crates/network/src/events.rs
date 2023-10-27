use bevy_ecs::prelude::*;

#[derive(Debug, Event)]
pub struct PeerDisconnected {
    /// Might not exist in [`bevy_app::PostUpdate`]
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct ClientBrand {
    pub from: Entity,
    pub brand: String,
}

#[derive(Debug, Event)]
pub struct ChatMessageSent {
    pub from: Entity,
    pub content: String,
}
