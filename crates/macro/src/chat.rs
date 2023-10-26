use proc_macro2::*;
use quote::*;
use syn::{parse::*, punctuated::*, token::*, *};

enum Flag {
    Bold,
    Color(Bracket, Lit),
}

impl Parse for Flag {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        match name.to_string().as_str() {
            "b" => Ok(Flag::Bold),
            "c" => {
                let content;

                Ok(Flag::Color(bracketed!(content in input), content.parse()?))
            }
            _ => Err(input.error("unrecognized flag")),
        }
    }
}

impl ToTokens for Flag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(match self {
            Flag::Bold => quote!(result.bold = true;),
            Flag::Color(_, name) => quote!(result.color = Some(#name.to_string());),
        });
    }
}

struct Flags(Vec<Flag>);

impl Parse for Flags {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut flags = Vec::new();

        if input.peek(Ident) {
            loop {
                flags.push(input.parse()?);

                if input.peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                } else {
                    break;
                }
            }
        }

        Ok(Flags(flags))
    }
}

impl ToTokens for Flags {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            quote!(let mut result = ::rjacraft_protocol::types::chat::Attrs::default();),
        );

        for flag in &self.0 {
            flag.to_tokens(tokens);
        }

        tokens.append_all(quote!(result));
    }
}

enum Text {
    None,
    Raw(Literal),
    Format(Literal, Token![,], Punctuated<Expr, Token![,]>),
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.peek(Lit) {
            return Ok(Text::None);
        }

        let string = input.parse()?;

        if input.peek(Token![,]) {
            Ok(Text::Format(
                string,
                input.parse()?,
                input.parse_terminated(Expr::parse, Token![,])?,
            ))
        } else {
            Ok(Text::Raw(string))
        }
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Text::None => tokens.append_all(quote!(::std::string::String::new())),
            Text::Raw(x) => tokens.append_all(quote!(#x.to_string())),
            Text::Format(format, _, args) => tokens.append_all(quote!(format!(#format, #args))),
        };
    }
}

struct Extra(Vec<(Paren, ChatNode)>);

impl Parse for Extra {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = Vec::new();

        while input.peek(token::Paren) {
            let content;

            nodes.push((parenthesized!(content in input), content.parse()?));
        }

        Ok(Extra(nodes))
    }
}

impl ToTokens for Extra {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for (_, node) in &self.0 {
            node.to_tokens(tokens);
            tokens.append_all(quote!(,));
        }
    }
}

pub struct ChatNode {
    flags: Flags,
    text: Text,
    extra: Extra,
}

impl Parse for ChatNode {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ChatNode {
            flags: input.parse()?,
            text: input.parse()?,
            extra: input.parse()?,
        })
    }
}

impl ToTokens for ChatNode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ChatNode { flags, text, extra } = self;

        tokens.append_all(quote! {
            ::rjacraft_protocol::types::chat::Chat {
                text: #text,
                attrs: { #flags },
                extra: vec![#extra],
            }
        });
    }
}

pub fn handle(top_node: ChatNode) -> TokenStream {
    top_node.into_token_stream()
}
