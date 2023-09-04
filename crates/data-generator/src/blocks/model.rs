#[derive(Debug, Clone)]
pub struct Id(pub u32);

#[derive(Debug)]
pub struct Block {
    pub name: String, // PascalCase, dirty
    pub properties: Vec<BlockProperty>,
    pub states: Vec<(Id, BlockState)>,
    pub default_state: BlockState,
}

#[derive(Debug)]
pub struct BlockProperty {
    pub name: String,          // snake_case, dirty
    pub variants: Vec<String>, // PascalCase, dirty
}

#[derive(Debug, Clone)]
pub struct BlockState {
    pub block_name: String, // PascalCase, dirty
    pub properties: Vec<BlockStateProperty>,
    pub default: bool,
}

#[derive(Debug, Clone)]
pub struct BlockStateProperty {
    pub name: String,         // snake_case, dirty
    pub variant_name: String, // PascalCase, dirty
}
