use proc_macro2::*;
use quote::*;
use syn::LitStr;

// yes this is copy-pasted. i'm not sure how we wound factor this out without causing recursive
// deps

const MAX_SIZE: usize = 1 << 15;
const DEFAULT_NS: &str = "minecraft";

struct Identifier {
    namespace: String,
    location: String,
}

fn parse_identifier(s: &str) -> Result<Identifier, String> {
    if s.len() > MAX_SIZE {
        Err("too long".to_string())?;
    }

    let mut split = s.split(":");

    let left = split.next().unwrap();
    let right = split.next();

    if split.next().is_some() {
        Err("unexpected extra colon".to_string())?;
    }

    if let Some(right) = right {
        if left.is_empty() {
            Err("expected a namespace (remove the colon?)".to_string())?;
        }

        if right.is_empty() {
            Err("expected a location".to_string())?;
        }

        for char in left.chars() {
            if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '-') {
                Err(format!("unexpected character in namespace: {char}"))?;
            }
        }

        for char in right.chars() {
            if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '/' | '.' | '-') {
                Err(format!("unexpected character in location: {char}"))?;
            }
        }

        Ok(Identifier {
            namespace: left.into(),
            location: right.into(),
        })
    } else {
        if left.is_empty() {
            Err("expected a location".to_string())?;
        }

        for char in left.chars() {
            if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '/' | '.' | '-') {
                Err(format!("unexpected character in location: {char}"))?;
            }
        }

        Ok(Identifier {
            namespace: DEFAULT_NS.into(),
            location: left.into(),
        })
    }
}

pub fn handle(literal: LitStr) -> TokenStream {
    match parse_identifier(&literal.value()) {
        Ok(Identifier {
            namespace,
            location,
        }) => {
            quote! { unsafe {
                ::rjacraft_protocol::types::identifier::Identifier::from_parts_unchecked(
                    #namespace.to_string(),
                    #location.to_string(),
                )
            } }
        }
        Err(message) => {
            quote_spanned!(literal.span() => compile_error!(#message))
        }
    }
}
