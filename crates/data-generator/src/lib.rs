use std::io::Write;

mod blocks;

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("error generating blocks: {0}")]
    Blocks(#[from] blocks::GenerateError),
}

pub fn gen_structs(json_data: String, sink: &mut impl Write) -> Result<(), GenerateError> {
    Ok(blocks::gen_block_structs(json_data, sink)?)
}
