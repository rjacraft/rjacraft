use bevy_ecs::prelude::*;

#[derive(Debug, Event)]
pub struct PeerDisconnected {
    /// Might not exist in [`bevy_app::PostUpdate`]
    pub peer: Entity,
}

#[derive(Debug, Event)]
pub struct C2sPacket<T>(pub Entity, pub T);

pub mod packet {
    use rjacraft_protocol::{packets::c2s, types};

    #[derive(Debug, Clone)]
    pub struct ClientBrand {
        pub brand: String,
    }

    #[derive(Debug, Clone)]
    pub struct Chat {
        pub content: String,
    }

    #[derive(Debug, Clone)]
    pub enum Movement {
        Position(f64, f64, f64),
        Rotation(f32, f32),
        OnGround(bool),
    }

    #[derive(Debug, Clone)]
    pub struct ClientInfo {
        pub locale: String,
        pub view_distance: i8,
        pub chat_mode: c2s::ChatMode,
        pub chat_colors: bool,
        pub skin_parts: u8,
        pub main_hand: c2s::HandAbs,
        pub text_filtering: bool,
        pub show_on_listings: bool,
    }

    #[derive(Debug, Clone)]
    pub enum WindowAction {
        Button(u8),
        Click {
            slot: c2s::OptionalSlot,
            button: u8,
            mode: u32,
            new_slots: Vec<(types::Primitive<u16>, types::ItemStackProto)>,
            carried_item: types::ItemStackProto,
        },
        Close,
    }

    #[derive(Debug, Clone)]
    pub struct Window {
        pub sync_id: u8,
        pub action: WindowAction,
    }

    #[derive(Debug, Clone)]
    pub enum Item {
        OnBlock {
            hand: c2s::HandRel,
            block_pos: types::BlockPos,
            block_face: c2s::Face,
            cursor_x: f32,
            cursor_y: f32,
            cursor_z: f32,
            head_buried: bool,
        },
        OnAir(c2s::HandRel),
    }
}
