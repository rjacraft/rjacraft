use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};

/// A multi-purpose identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    snake_case: Ident,
    pascal_case: Ident,
}

impl Name {
    pub fn new(snake_case: impl AsRef<str>, pascal_case: impl AsRef<str>) -> Self {
        Name {
            snake_case: Self::try_parse_ident(snake_case),
            pascal_case: Self::try_parse_ident(pascal_case),
        }
    }

    pub fn from_snake_case(snake_case: impl Into<String>) -> Self {
        let snake_case = snake_case.into();
        let pascal_case = snake_case.as_str().to_case(Case::Pascal);

        Name {
            snake_case: Self::try_parse_ident(snake_case),
            pascal_case: Self::try_parse_ident(pascal_case),
        }
    }

    pub fn snake_case(&self) -> &Ident {
        &self.snake_case
    }

    pub fn pascal_case(&self) -> &Ident {
        &self.pascal_case
    }

    pub fn into_pascal_case(self) -> Ident {
        self.pascal_case
    }

    fn try_parse_ident(src: impl AsRef<str>) -> Ident {
        syn::parse_str(src.as_ref())
            .unwrap_or_else(|_| Ident::new_raw(src.as_ref(), Span::call_site()))
    }
}
