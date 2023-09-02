pub(crate) use block_struct::*;

mod block_struct {
    use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt};

    use crate::blocks::model::Block;

    pub struct BlockStruct {
        pub name: String, // PascalCase
        pub properties: Vec<BlockStructField>,
    }

    impl From<&Block> for BlockStruct {
        fn from(block: &Block) -> Self {
            use convert_case::{Case, Casing as _};

            let properties = block
                .properties
                .iter()
                .map(|block_prop| BlockStructField {
                    prop_name: block_prop.name.clone(),
                    prop_enum_name: format!(
                        "{}{}",
                        &block.name,
                        &block_prop.name.to_case(Case::Pascal)
                    ),
                })
                .collect();

            let name = block.name.clone();
            Self { name, properties }
        }
    }

    impl ToTokens for BlockStruct {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = Ident::new(&self.name, Span::call_site());
            // "pub struct RedTerracotta"
            tokens.extend(quote! { pub struct #block_name });

            if self.properties.is_empty() {
                // "pub struct LavaCauldron;"
                tokens.append(Punct::new(';', Spacing::Joint))
            } else {
                let mut stream = TokenStream::new();
                self.properties
                    .iter()
                    .for_each(|prop| prop.to_tokens(&mut stream));

                // "pub struct Lectern { pub has_book: LecternHasBook, }"
                tokens.append(Group::new(Delimiter::Brace, stream))
            }
        }
    }

    pub struct BlockStructField {
        pub prop_name: String,      // snake_case
        pub prop_enum_name: String, // PascalCase
    }

    impl ToTokens for BlockStructField {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_name = Ident::new(&self.prop_name, Span::call_site());
            let prop_enum_name = Ident::new(&self.prop_enum_name, Span::call_site());

            // "pub open: SpruceDoorOpen,"
            tokens.extend(quote!(pub #prop_name: #prop_enum_name,))
        }
    }
}
