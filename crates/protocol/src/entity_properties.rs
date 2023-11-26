pub mod entity {
    pub const FLAGS: u8 = 0;
    pub const AIR_TICKS: u8 = 1;
    pub const CUSTOM_NAME: u8 = 2;
    pub const CUSTOM_NAME_VISIBLE: u8 = 3;
    pub const SILENT: u8 = 4;
    pub const NO_GRAVITY: u8 = 5;
    pub const POSE: u8 = 6;
    pub const FROZEN_TICKS: u8 = 7;
}

pub mod living_entity {
    pub const HAND_STATES: u8 = 8;
    pub const HEALTH: u8 = 9;
    pub const EFFECT_COLOR: u8 = 10;
    pub const EFFECT_AMBIENT: u8 = 11;
    pub const ARROWS: u8 = 12;
    pub const BEE_STINGERS: u8 = 13;
    pub const SLEEPING_IN: u8 = 14;
}

pub mod player {
    pub const EXTRA_HEARTS: u8 = 15;
    pub const SCORE: u8 = 16;
    pub const SKIN_PARTS: u8 = 17;
    pub const MAIN_HAND: u8 = 18;
    pub const LEFT_SHOULDER_ENTITY: u8 = 19;
    pub const RIGHT_SHOULDER_ENTITY: u8 = 20;
}
