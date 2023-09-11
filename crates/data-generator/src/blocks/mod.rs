use std::io::{Error as IoError, Write as IoWrite};

pub use json::ParseError as JsonParseError;

mod json;
mod model;
mod output;

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("I/O operation failed")]
    Io(#[from] IoError),
    #[error("JSON parsing failed")]
    Parse(#[from] JsonParseError),
}

pub fn gen_blocks_module(json_data: String, sink: &mut impl IoWrite) -> Result<(), GenerateError> {
    let blocks = json::parse_block_registry(json_data)?;
    let stream = output::gen_blocks_mod(blocks);

    let syn_tree = syn::parse2(stream).expect("parse TokenStream into syn::File");
    let pretty_output = prettyplease::unparse(&syn_tree);
    Ok(writeln!(sink, "{}", pretty_output)?)
}
