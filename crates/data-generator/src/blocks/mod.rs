use std::io::{Error as IoError, Write as IoWrite};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse2;

use self::model::{Block, BlockProperty};
use self::output::{BlockConvert, BlockDefault, BlockStruct, PropertyConvert, PropertyStruct};

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

pub(crate) fn gen_block_structs(
    json_data: String,
    sink: &mut impl IoWrite,
) -> Result<(), GenerateError> {
    for block in json::parse_block_registry(json_data)? {
        let mut stream = TokenStream::new();
        gen_block_code(&block, &mut stream);

        let syn_tree = parse2(stream).expect("parse TokenStream into syn::File");
        let pretty_output = prettyplease::unparse(&syn_tree);
        writeln!(sink, "{}", pretty_output).expect("write to destination file");
    }

    Ok(())
}

fn gen_block_code(block: &Block, stream: &mut TokenStream) {
    let block_struct: BlockStruct = BlockStruct::from(block);
    block_struct.to_tokens(stream);

    let block_default = BlockDefault::from(block);
    block_default.to_tokens(stream);

    let block_conv = BlockConvert::from(block);
    block_conv.from_u32().to_tokens(stream);
    block_conv.into_u32().to_tokens(stream);

    block
        .properties
        .iter()
        .for_each(|prop| gen_prop_code(prop, stream));
}

fn gen_prop_code(prop: &BlockProperty, stream: &mut TokenStream) {
    let prop_struct = PropertyStruct::from(prop);
    prop_struct.to_tokens(stream);

    let prop_conv = PropertyConvert::from(prop);
    prop_conv.from_u8().to_tokens(stream);
}
