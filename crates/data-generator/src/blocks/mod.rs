use std::io::{Error as IoError, Write};

pub use json::ParseError as JsonParseError;

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
    sink: &mut impl Write,
) -> Result<(), GenerateError> {
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::parse2;

    use self::output::{BlockConvert, BlockStruct};

    for block in json::parse_block_registry(json_data)? {
        let mut stream = TokenStream::new();

        let block_struct: BlockStruct = BlockStruct::from(&block);
        block_struct.to_tokens(&mut stream);

        let block_conv = BlockConvert::from(&block);
        block_conv.from_u32().to_tokens(&mut stream);
        block_conv.into_u32().to_tokens(&mut stream);

        let syn_tree = parse2(stream).expect("parse TokenStream into syn::File");
        let pretty_output = prettyplease::unparse(&syn_tree);
        writeln!(sink, "{}", pretty_output).expect("write to destination file");
    }

    Ok(())
}
