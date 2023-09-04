use quote::ToTokens;
use syn::*;

#[proc_macro_derive(FromNever)]
pub fn derive_from_never(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let generic_params = &input.generics.params;
    let generic_where = &input.generics.where_clause;
    let generic_names = generic_params.iter().map(|it| match it {
        GenericParam::Const(x) => x.ident.to_token_stream(),
        GenericParam::Lifetime(x) => x.lifetime.to_token_stream(),
        GenericParam::Type(x) => x.ident.to_token_stream(),
    });

    proc_macro::TokenStream::from(quote::quote! {
        impl<#generic_params>
            ::core::convert::From<::core::convert::Infallible> for #ident <#(#generic_names),*>
            #generic_where
        {
            fn from(x: ::core::convert::Infallible) -> Self {
                match x {}
            }
        }
    })
}
