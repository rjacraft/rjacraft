#[derive(Debug, Clone)]
pub struct Id(pub u32);

#[derive(Debug)]
pub struct Block {
    pub name: String, // PascalCase
    pub properties: Vec<BlockProperty>,
    pub states: Vec<(Id, BlockState)>,
    pub default_state: BlockState,
}

#[derive(Debug)]
pub struct BlockProperty {
    pub name: String,          // snake_case
    pub variants: Vec<String>, // PascalCase
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub block_name: String, // PascalCase
    pub properties: Vec<BlockStateProperty>,
    pub default: bool,
}

#[derive(Debug, Clone)]
pub struct BlockStateProperty {
    pub name: String,         // snake_case
    pub variant_name: String, // PascalCase
}
