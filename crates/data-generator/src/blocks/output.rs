pub(crate) use block_convert::*;
pub(crate) use block_default::*;
pub(crate) use block_struct::*;
pub(crate) use prop_struct::*;

mod block_struct {
    use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt};

    use crate::blocks::model::Block;

    pub struct BlockStruct {
        pub name: String, // PascalCase, dirty
        pub properties: Vec<BlockStructField>,
    }

    impl From<&Block> for BlockStruct {
        fn from(block: &Block) -> Self {
            use convert_case::{Case, Casing as _};

            let properties = block
                .properties
                .iter()
                .map(|block_prop| {
                    let prop_name_sc = super::defuse_property_name(&block_prop.prop_name);
                    let prop_name_pc = prop_name_sc.to_case(Case::Pascal);
                    let prop_enum_name = format!("{}{}", &block.name, &prop_name_pc);
                    BlockStructField {
                        prop_name: prop_name_sc,
                        prop_enum_name,
                    }
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
        pub prop_name: String,      // snake_case, defused
        pub prop_enum_name: String, // PascalCase, defused
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

mod block_default {
    use proc_macro2::{Ident, Span, TokenStream};
    use quote::{quote, ToTokens};

    use crate::blocks::model::Block;

    use super::block_convert::BlockStateInst;

    pub struct BlockDefault {
        pub block_name: String, // PascalCase
        pub block_state_default: BlockStateInst,
    }

    impl From<&Block> for BlockDefault {
        fn from(block: &Block) -> Self {
            BlockDefault {
                block_name: block.name.clone(),
                block_state_default: (&block.default_state).into(),
            }
        }
    }

    impl ToTokens for BlockDefault {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = Ident::new(&self.block_name, Span::call_site());
            let block_inst = self.block_state_default.to_token_stream();

            tokens.extend(quote! {
                impl Default for #block_name {
                    fn default() -> Self {
                        #block_inst
                    }
                }
            });
        }
    }
}

mod block_convert {
    use proc_macro2::{Delimiter, Group, Ident, Literal, Span, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt};

    use crate::blocks::model::{Block, BlockState, Id};

    #[derive(Debug)]
    pub struct BlockConvert {
        pub block_name: String, // PascalCase, dirty
        pub id_states: Vec<(Id, BlockStateInst)>,
    }

    impl From<&Block> for BlockConvert {
        fn from(block: &Block) -> Self {
            let id_states = block
                .states
                .iter()
                .map(|(block_id, block_state)| (block_id.clone(), block_state.into()))
                .collect();

            Self {
                block_name: block.name.clone(),
                id_states: id_states,
            }
        }
    }

    impl BlockConvert {
        pub fn into_u32<'a>(&'a self) -> BlockIntoU32<'a> {
            BlockIntoU32(self)
        }

        pub fn from_u32<'a>(&'a self) -> BlockFromU32<'a> {
            BlockFromU32(self)
        }
    }

    #[derive(Debug)]
    pub struct BlockIntoU32<'a>(&'a BlockConvert);

    impl ToTokens for BlockIntoU32<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = Ident::new(&self.0.block_name, Span::call_site());

            let mut match_arms = TokenStream::new();
            for (id, state) in &self.0.id_states {
                // "LargeAmethystBud { Facing: North, Waterlogged: True, }"
                let state_inst = state.to_token_stream();

                // "LavaCauldron => 7402,"
                let stream = quote! { #state_inst => #id, };
                stream.to_tokens(&mut match_arms);
            }

            // It is assumed that the states property covers all possible
            // states for every given block. In other words, the match is
            // supposed to be exhaustive.
            tokens.extend(quote! {
                impl From<&#block_name> for u32 {
                    fn from(state: &#block_name) -> Self {
                        match state {
                            #match_arms
                        }
                    }
                }
            });
        }
    }

    #[derive(Debug)]
    pub struct BlockFromU32<'a>(&'a BlockConvert);

    impl ToTokens for BlockFromU32<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = Ident::new(&self.0.block_name, Span::call_site());

            let mut match_arms = TokenStream::new();
            for (id, state) in &self.0.id_states {
                // "LargeAmethystBud {
                //      facing: LargeAmethystBudFacing::North,
                //      waterlogged: LargeAmethystBudWaterlogged::True,
                //  }"
                let state_inst = state.to_token_stream();

                // "7402 => LavaCauldron,"
                let stream = quote! { #id => #state_inst, };
                stream.to_tokens(&mut match_arms);
            }

            tokens.extend(quote! {
                impl From<u32> for #block_name {
                    fn from(id: u32) -> Self {
                        match id {
                            #match_arms
                            _ => panic!("unknown id"),
                        }
                    }
                }
            });
        }
    }

    #[derive(Debug)]
    pub struct BlockStateInst {
        pub block_name: String, // PascalCase, dirty
        pub properties: Vec<BlockStateProperty>,
    }

    impl From<&BlockState> for BlockStateInst {
        fn from(block_state: &BlockState) -> Self {
            use convert_case::{Case, Casing as _};

            let block_state_props = block_state
                .properties
                .iter()
                .map(|block_prop| {
                    let variant_name = super::defuse_variant_name(&block_prop.variant_name);
                    let prop_name = super::defuse_property_name(&block_prop.name);
                    let prop_enum_name = format!(
                        "{}{}",
                        &block_state.block_name,
                        &prop_name.to_case(Case::Pascal)
                    );

                    BlockStateProperty {
                        variant_name,
                        prop_name,
                        prop_enum_name,
                    }
                })
                .collect();

            BlockStateInst {
                block_name: block_state.block_name.clone(),
                properties: block_state_props,
            }
        }
    }

    impl ToTokens for BlockStateInst {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            // "Ladder"
            tokens.append(Ident::new(&self.block_name, Span::call_site()));

            if !self.properties.is_empty() {
                let mut props = TokenStream::new();
                // "facing: LadderFacing::North, waterlogged: LadderWaterlogged::True,"
                self.properties
                    .iter()
                    .for_each(|prop| prop.to_tokens(&mut props));

                // "Ladder { facing: LadderFacing::North, waterlogged: LadderWaterlogged::True, }"
                tokens.append(Group::new(Delimiter::Brace, props))
            }
        }
    }

    #[derive(Debug)]
    pub struct BlockStateProperty {
        pub prop_name: String,      // snake_case, defused
        pub prop_enum_name: String, // PascalCase, defused
        pub variant_name: String,   // PascalCase, defused
    }

    impl ToTokens for BlockStateProperty {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_name = Ident::new(&self.prop_name, Span::call_site());
            let prop_enum_name = Ident::new(&self.prop_enum_name, Span::call_site());
            let variant_name = Ident::new(&self.variant_name, Span::call_site());

            // "instrument: NoteBlock::IronXylophone,"
            tokens.extend(quote!(#prop_name: #prop_enum_name::#variant_name,))
        }
    }

    impl ToTokens for Id {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.append(Literal::u32_unsuffixed(self.0))
        }
    }
}

mod prop_struct {
    use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt};

    use crate::blocks::model::BlockProperty;

    pub struct PropertyStruct {
        pub block_name: String, // PascalCase, dirty
        pub prop_name: String,  // PascalCase, dirty
        pub prop_variants: Vec<PropertyStructField>,
    }

    impl From<&BlockProperty> for PropertyStruct {
        fn from(block_prop: &BlockProperty) -> Self {
            use convert_case::{Case, Casing as _};

            let block_name_pc = block_prop.block_name.clone();
            let prop_name_sc = super::defuse_property_name(&block_prop.prop_name);
            let prop_name_pc = prop_name_sc.to_case(Case::Pascal);

            let prop_variants = block_prop
                .variants
                .iter()
                .enumerate()
                .map(|(var_ordinal, var_name)| {
                    let var_name_def = super::defuse_variant_name(var_name);
                    let var_ordinal = if var_name == &var_name_def {
                        None
                    } else {
                        Some(var_ordinal)
                    };

                    PropertyStructField {
                        var_name: var_name_def,
                        var_ordinal: var_ordinal,
                    }
                })
                .collect();

            Self {
                block_name: block_name_pc,
                prop_name: prop_name_pc,
                prop_variants,
            }
        }
    }

    impl ToTokens for PropertyStruct {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_name = Ident::new(
                &format!("{}{}", &self.block_name, &self.prop_name),
                Span::call_site(),
            );
            // "pub enum BubbleCoralFan"
            tokens.extend(quote! { pub enum #prop_name });

            // It is assumed that there can never be less than two variants.
            let mut stream = TokenStream::new();
            self.prop_variants
                .iter()
                .for_each(|var| var.to_tokens(&mut stream));

            // "{ North, East, South, West, }"
            tokens.append(Group::new(Delimiter::Brace, stream));
        }
    }

    pub struct PropertyStructField {
        pub var_name: String, // PascalCase, defused
        pub var_ordinal: Option<usize>,
    }

    impl ToTokens for PropertyStructField {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let struct_name = Ident::new(&self.var_name, Span::call_site());
            // "CampfireLit"
            tokens.append(struct_name);

            // "FarmlandMoisture = 5"
            self.var_ordinal.into_iter().for_each(|ord| {
                tokens.append(Punct::new('=', Spacing::Alone));
                tokens.append(Literal::usize_unsuffixed(ord));
            });

            // ","
            tokens.append(Punct::new(',', Spacing::Alone));
        }
    }
}

fn defuse_property_name(property_name: impl AsRef<str>) -> String {
    match property_name.as_ref() {
        "type" => "kind".to_string(),
        _ => property_name.as_ref().to_string(),
    }
}

fn defuse_variant_name(variant_name: impl AsRef<str>) -> String {
    let variant_name = variant_name.as_ref();
    let first_char = variant_name.chars().next().expect("variant name not empty");

    if !first_char.is_numeric() {
        variant_name.into()
    } else {
        if variant_name.chars().all(|c| c.is_numeric()) {
            convert_into_roman(variant_name.parse().unwrap())
        } else {
            format!("_{}", variant_name)
        }
    }
}

fn convert_into_roman(n: u32) -> String {
    use numerals::roman::Roman;
    match n {
        0 => "O".to_string(),
        _ => format!("{:X}", Roman::from(n as i16)),
    }
}
