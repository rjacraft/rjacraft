use quote::{quote, quote_spanned, ToTokens};
use rjacraft_protocol::types::Identifier;
use syn::parse_macro_input;

mod chat;

/// Construct recursive [chat components][chat] with a compact syntax.
///
/// [chat]: rjacraft_protocol::types::chat
#[proc_macro]
pub fn chat(tokens_in: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let top_node: chat::ChatNode = parse_macro_input!(tokens_in);

    top_node.into_token_stream().into()
}

/// Construct [identifiers][id] with a compile-time guarantee about their validity.
///
/// [id]: rjacraft_protocol::types::identifier
#[proc_macro]
pub fn id(tokens_in: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let literal: syn::LitStr = parse_macro_input!(tokens_in);

    match literal.value().parse::<Identifier>() {
        Ok(parsed) => {
            let (ns, loc) = parsed.parts();

            quote! { unsafe {
                ::rjacraft_protocol::types::identifier::Identifier::from_parts_unchecked(
                    #ns.to_string(),
                    #loc.to_string(),
                )
            } }
            .into()
        }
        Err(e) => {
            let message = e.to_string();

            quote_spanned!(literal.span() => compile_error!(#message)).into()
        }
    }
}
