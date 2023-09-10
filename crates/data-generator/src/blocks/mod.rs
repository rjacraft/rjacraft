use std::io::{Error as IoError, Write as IoWrite};

pub(crate) use json::ParseError as JsonParseError;

mod json;
mod model;
mod output;

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("parsing error: {0}")]
    Parse(#[from] JsonParseError),
    #[error("I/O error writing: {0}")]
    Write(#[from] IoError),
}

pub(crate) fn gen_blocks_module(
    json_data: String,
    sink: &mut impl IoWrite,
) -> Result<(), GenerateError> {
    let blocks = json::parse_block_registry(json_data)?;
    let stream = output::gen_blocks_mod(blocks);

    let syn_tree = syn::parse2(stream).expect("parse TokenStream into syn::File");
    let pretty_output = prettyplease::unparse(&syn_tree);
    Ok(writeln!(sink, "{}", pretty_output)?)
}
