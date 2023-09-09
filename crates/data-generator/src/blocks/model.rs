#[derive(Debug, Clone)]
pub struct Id(pub u32);

#[derive(Debug, Clone)]
pub struct Block {
    pub name_sc: String, // bubble_coral_fan
    pub name_pc: String, // BubbleCoralFan
    pub properties: Vec<BlockProperty>,
    pub states: Vec<(Id, State)>,
    pub default_state: State,
}

#[derive(Debug, Clone)]
pub struct BlockProperty {
    pub block_name: String,                  // BubbleCoralFan
    pub name_sc: String,                     // honey_level
    pub name_pc: String,                     // HoneyLevel
    pub variants: Vec<BlockPropertyVariant>, // [ North, East, South, West ]
}

impl BlockProperty {
    pub fn is_num(&self) -> bool {
        self.variants
            .iter()
            .next()
            .map(|x| matches!(x, BlockPropertyVariant::Numeric(_, _)))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BlockPropertyVariant {
    Regular(String),
    Bool(bool, String),
    Numeric(u8, String),
}

impl BlockPropertyVariant {
    pub fn defused_name<'a>(&'a self) -> &'a String {
        match self {
            &BlockPropertyVariant::Regular(ref s) => s,
            &BlockPropertyVariant::Bool(_, ref s) => s,
            &BlockPropertyVariant::Numeric(_, ref s) => s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub properties: Vec<StateProperty>,
    pub default: bool,
}

#[derive(Debug, Clone)]
pub struct StateProperty {
    pub field: String,     // honey_level
    pub prop_enum: String, // HoneyLevel
    pub variant: String,   // IV
}
