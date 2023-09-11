use std::io::Write;

mod blocks;
mod name;

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("generating blocks failed")]
    Blocks(#[from] blocks::GenerateError),
}

pub fn gen_structs(json_data: String, sink: &mut impl Write) -> Result<(), GenerateError> {
    Ok(blocks::gen_blocks_module(json_data, sink)?)
}
