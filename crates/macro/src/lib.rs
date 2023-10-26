use proc_macro::TokenStream;
use syn::parse_macro_input;

mod chat;
mod id;
mod protocol_type;

/// Construct recursive [chat components][chat] with a compact syntax.
///
/// [chat]: rjacraft_protocol::types::chat
#[proc_macro]
pub fn chat(input: TokenStream) -> TokenStream {
    chat::handle(parse_macro_input!(input)).into()
}

/// Construct [identifiers][id] with a compile-time guarantee about their validity.
///
/// [id]: rjacraft_protocol::types::identifier
#[proc_macro]
pub fn id(input: TokenStream) -> TokenStream {
    id::handle(parse_macro_input!(input)).into()
}

#[proc_macro_derive(ProtocolType, attributes(variant))]
pub fn protocol_type(item: TokenStream) -> TokenStream {
    protocol_type::handle(parse_macro_input!(item)).into()
}
