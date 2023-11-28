use proc_macro2::*;
use quote::*;
use syn::{parse::*, punctuated::*, token::*, *};

enum Flag {
    Bold,
    Italic,
    Underlined,
    Color(Bracket, Expr),
    Insertion(Bracket, Expr),
    ClickEvent(Bracket, Expr),
    HoverEvent(Bracket, Expr),
}

impl Parse for Flag {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let expr;

        match name.to_string().as_str() {
            "b" => Ok(Flag::Bold),
            "i" => Ok(Flag::Italic),
            "u" => Ok(Flag::Underlined),
            "c" => Ok(Flag::Color(bracketed!(expr in input), expr.parse()?)),
            "in" => Ok(Flag::Insertion(bracketed!(expr in input), expr.parse()?)),
            "ce" => Ok(Flag::ClickEvent(bracketed!(expr in input), expr.parse()?)),
            "he" => Ok(Flag::HoverEvent(bracketed!(expr in input), expr.parse()?)),
            _ => Err(input.error("unrecognized flag")),
        }
    }
}

impl ToTokens for Flag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(match self {
            Flag::Bold => quote!(result.bold = Some(true);),
            Flag::Italic => quote!(result.italic = Some(true);),
            Flag::Underlined => quote!(result.underlined = Some(true);),
            Flag::Color(_, x) => quote!(result.color = Some(#x.to_string());),
            Flag::Insertion(_, x) => quote!(result.insertion = Some(#x.to_string());),
            Flag::ClickEvent(_, x) => quote!(result.click_event = Some(#x);),
            Flag::HoverEvent(_, x) => quote!(result.hover_event = Some(#x);),
        });
    }
}

struct Style(Vec<Flag>);

impl Parse for Style {
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

        Ok(Style(flags))
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            quote!(let mut result = ::rjacraft_protocol::types::text::Style::default();),
        );

        for flag in &self.0 {
            flag.to_tokens(tokens);
        }

        tokens.append_all(quote!(result));
    }
}

enum Content {
    None,
    Raw(Literal),
    Format(Literal, Token![,], Punctuated<Expr, Token![,]>),
}

impl Parse for Content {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.peek(Lit) {
            return Ok(Content::None);
        }

        let string = input.parse()?;

        if input.peek(Token![,]) {
            Ok(Content::Format(
                string,
                input.parse()?,
                input.parse_terminated(Expr::parse, Token![,])?,
            ))
        } else {
            Ok(Content::Raw(string))
        }
    }
}

impl ToTokens for Content {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Content::None => tokens.append_all(quote! { ::std::string::String::new() }),
            Content::Raw(x) => tokens.append_all(quote! { format!(#x) }),
            Content::Format(format, _, args) => {
                tokens.append_all(quote! { format!(#format, #args) })
            }
        };
    }
}

struct Extra(Vec<(Paren, Node)>);

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

pub struct Node {
    style: Style,
    content: Content,
    extra: Extra,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Node {
            style: input.parse()?,
            content: input.parse()?,
            extra: input.parse()?,
        })
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Node {
            style,
            content,
            extra,
        } = self;

        match (&style.0[..], &extra.0[..], content) {
            ([], [], _) => tokens.append_all(quote! {
                ::rjacraft_protocol::types::text::Text::Literal(#content)
            }),
            ([], _, Content::None) => tokens.append_all(quote! {
                ::rjacraft_protocol::types::text::Text::Array(vec![#extra])
            }),
            _ => tokens.append_all(quote! {
                ::rjacraft_protocol::types::text::Text::Fancy {
                    content: ::rjacraft_protocol::types::text::Content::Literal {
                        text: #content
                    },
                    style: { #style },
                    extra: vec![#extra],
                }
            }),
        }
    }
}

pub fn handle(top_node: Node) -> TokenStream {
    top_node.into_token_stream()
}
