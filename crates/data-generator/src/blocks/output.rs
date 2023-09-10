use proc_macro2::TokenStream;
use quote::ToTokens;

pub(super) fn gen_blocks_mod(blocks: Vec<super::model::Block>) -> TokenStream {
    blocks_mod::BlocksMod::from(blocks).to_token_stream()
}

mod blocks_mod {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use crate::blocks::model::Block;

    pub struct BlocksMod {
        pub blocks: Vec<Block>,
    }

    impl From<Vec<Block>> for BlocksMod {
        fn from(blocks: Vec<Block>) -> Self {
            Self { blocks }
        }
    }

    impl ToTokens for BlocksMod {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            use super::block_mod::BlockMod;
            let block_mods = self.blocks.iter().map(BlockMod);

            tokens.extend(quote! {
                pub mod blocks {
                    #[derive(Debug)]
                    pub struct UnknownId(u32);
                    #[derive(Debug)]
                    pub struct UnknownVar(u8);

                    #(#block_mods)*
                }
            });
        }
    }
}

mod block_mod {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use super::StringExt as _;
    use crate::blocks::model::Block;

    pub struct BlockMod<'a>(pub &'a Block);

    impl ToTokens for BlockMod<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_mod_name = self.0.name_sc.to_ident();
            let block_code = gen_block_code(self.0);
            let props_code = gen_props_code(self.0);

            tokens.extend(quote! {
                pub mod #block_mod_name {
                    #block_code
                    #props_code
                }
            });
        }
    }

    fn gen_block_code(block: &Block) -> TokenStream {
        let mut tokens = TokenStream::new();
        super::block_struct::BlockStruct::from(block).to_tokens(&mut tokens);
        super::block_default::BlockDefault::from(block).to_tokens(&mut tokens);
        super::block_convert::BlockConvert::from(block).to_tokens(&mut tokens);
        tokens
    }

    fn gen_props_code(block: &Block) -> TokenStream {
        let mut tokens = TokenStream::new();
        for prop in block.properties.iter() {
            let prop = prop.clone();
            super::prop_enum::PropertyEnum::from(&prop).to_tokens(&mut tokens);
            super::prop_convert::PropertyConvert::from(prop).to_tokens(&mut tokens);
        }
        tokens
    }
}

mod block_struct {
    use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt as _};

    use super::StringExt as _;
    use crate::blocks::model::Block;

    pub struct BlockStruct {
        pub name: String, // RedTerracotta
        pub properties: Vec<BlockStructField>,
    }

    impl From<&Block> for BlockStruct {
        fn from(block: &Block) -> Self {
            let properties = block
                .properties
                .iter()
                .map(|block_prop| BlockStructField {
                    prop_name: block_prop.name_sc.clone(),
                    prop_enum_name: block_prop.name_pc.clone(),
                })
                .collect();

            let name = block.name_pc.clone();
            Self { name, properties }
        }
    }

    impl ToTokens for BlockStruct {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = self.name.to_ident();

            tokens.extend(quote! {
                #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
                pub struct #block_name
            });

            if self.properties.is_empty() {
                // "pub struct LavaCauldron;"
                tokens.append(Punct::new(';', Spacing::Alone));
            } else {
                let mut stream = TokenStream::new();
                self.properties
                    .iter()
                    .for_each(|prop| prop.to_tokens(&mut stream));

                // "pub struct Lectern { pub has_book: HasBook, }"
                tokens.append(Group::new(Delimiter::Brace, stream));
            }
        }
    }

    pub struct BlockStructField {
        pub prop_name: String,      // honey_level
        pub prop_enum_name: String, // HoneyLevel
    }

    impl ToTokens for BlockStructField {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_name = self.prop_name.to_ident();
            let prop_enum_name = self.prop_enum_name.to_ident();

            // "pub open: Open,"
            tokens.extend(quote!(pub #prop_name: #prop_enum_name,))
        }
    }
}

mod block_default {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use super::{block_convert::BlockStateInst, StringExt as _};
    use crate::blocks::model::Block;

    #[derive(Debug)]
    pub struct BlockDefault {
        pub block_name: String, // WetSponge
        pub block_state_default: BlockStateInst,
    }

    impl From<&Block> for BlockDefault {
        fn from(block: &Block) -> Self {
            BlockDefault {
                block_name: block.name_pc.clone(),
                block_state_default: (&block.default_state).into(),
            }
        }
    }

    impl ToTokens for BlockDefault {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = self.block_name.to_ident();
            let block_inst = &self.block_state_default;

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
    use proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt as _};

    use super::StringExt as _;
    use crate::blocks::model::{Block, Id, State};

    #[derive(Debug)]
    pub struct BlockConvert {
        pub block_name: String, // BeeHive
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
                block_name: block.name_pc.clone(),
                id_states: id_states,
            }
        }
    }

    impl ToTokens for BlockConvert {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            BlockIntoU32(self).to_tokens(tokens);
            BlockFromU32(self).to_tokens(tokens);
        }
    }

    #[derive(Debug)]
    pub struct BlockIntoU32<'a>(&'a BlockConvert);

    impl ToTokens for BlockIntoU32<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_name = self.0.block_name.to_ident();

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
            let block_name = self.0.block_name.to_ident();

            let mut match_arms = TokenStream::new();
            for (id, state_inst) in &self.0.id_states {
                let state_inst = state_inst;

                // "4287 => Farmland,"
                let stream = quote! { #id => #state_inst, };
                stream.to_tokens(&mut match_arms);
            }

            tokens.extend(quote! {
                impl TryFrom<u32> for #block_name {
                    type Error = super::UnknownId;

                    fn try_from(id: u32) -> Result<Self, Self::Error> {
                        Ok(match id {
                            #match_arms
                            _ => Err(super::UnknownId(id))?,
                        })
                    }
                }
            });
        }
    }

    #[derive(Debug)]
    pub struct BlockStateInst {
        pub block_name: String, // BrownBanner
        pub properties: Vec<BlockStateProperty>,
    }

    impl From<&State> for BlockStateInst {
        fn from(block_state: &State) -> Self {
            let block_state_props = block_state
                .properties
                .iter()
                .map(|block_prop| BlockStateProperty {
                    field_name: block_prop.field.to_ident(),
                    enum_name: block_prop.prop_enum.to_ident(),
                    var_name: block_prop.variant.to_ident(),
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
            tokens.append(self.block_name.to_ident());

            if !self.properties.is_empty() {
                let mut props = TokenStream::new();
                // "facing: Facing::North, waterlogged: Waterlogged::True,"
                self.properties
                    .iter()
                    .for_each(|prop| prop.to_tokens(&mut props));

                // "LargeAmethystBud { facing: Facing::North, waterlogged: Waterlogged::True, }"
                tokens.append(Group::new(Delimiter::Brace, props))
            }
        }
    }

    #[derive(Debug)]
    pub struct BlockStateProperty {
        pub field_name: Ident, // instrument
        pub enum_name: Ident,  // NoteBlock
        pub var_name: Ident,   // IronXylophone
    }

    impl ToTokens for BlockStateProperty {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let field_name = &self.field_name;
            let enum_name = &self.enum_name;
            let var_name = &self.var_name;

            // "instrument: NoteBlock::IronXylophone,"
            tokens.extend(quote!(#field_name: #enum_name::#var_name,))
        }
    }

    impl ToTokens for Id {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.append(Literal::u32_unsuffixed(self.0))
        }
    }
}

mod prop_enum {
    use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt as _};

    use super::StringExt as _;
    use crate::blocks::model::BlockProperty;

    pub struct PropertyEnum {
        pub prop_name: String, // Powered
        pub prop_vars: Vec<PropertyEnumVariant>,
        pub ordered: bool,
    }

    impl From<&BlockProperty> for PropertyEnum {
        fn from(block_prop: &BlockProperty) -> Self {
            let prop_name_pc = block_prop.name_pc.clone();
            let ordered = block_prop.is_num();

            let prop_variants: Vec<PropertyEnumVariant> = block_prop
                .variants
                .iter()
                .enumerate()
                .map(|(var_ord, var_name)| PropertyEnumVariant {
                    var_name: var_name.defused_name().to_string(),
                    var_ordinal: Some(block_prop)
                        .filter(|&prop| prop.is_num())
                        .map(|_| var_ord),
                })
                .collect();

            Self {
                prop_name: prop_name_pc,
                prop_vars: prop_variants,
                ordered,
            }
        }
    }

    impl ToTokens for PropertyEnum {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let derive = if self.ordered {
                quote!(Ord, PartialOrd,)
            } else {
                TokenStream::new()
            };

            let prop_name = self.prop_name.to_ident();

            // "pub enum BubbleCoralFan"
            tokens.extend(quote! {
                #[derive(Debug, #derive Eq, PartialEq, Copy, Clone, Hash)]
                pub enum #prop_name
            });

            let mut stream = TokenStream::new();
            self.prop_vars
                .iter()
                .for_each(|var| var.to_tokens(&mut stream));

            // "{ North, East, South, West, }"
            tokens.append(Group::new(Delimiter::Brace, stream));
        }
    }

    pub struct PropertyEnumVariant {
        pub var_name: String,           // XIV
        pub var_ordinal: Option<usize>, // Some(14)
    }

    impl ToTokens for PropertyEnumVariant {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let struct_name = self.var_name.to_ident();
            // "CampfireLit"
            tokens.append(struct_name);

            // "Moisture = 5"
            self.var_ordinal.into_iter().for_each(|ord| {
                tokens.append(Punct::new('=', Spacing::Alone));
                tokens.append(Literal::usize_unsuffixed(ord));
            });

            // ","
            tokens.append(Punct::new(',', Spacing::Alone));
        }
    }
}

mod prop_convert {
    use proc_macro2::{Ident, Literal, TokenStream};
    use quote::{quote, ToTokens};

    use super::StringExt as _;
    use crate::blocks::model::{BlockProperty, BlockPropertyVariant};

    pub struct PropertyConvert {
        pub block_name: String, // BeeHive
        pub prop_name: String,  // HoneyLevel
        pub prop_vars: Vec<BlockPropertyVariant>,
    }

    impl From<BlockProperty> for PropertyConvert {
        fn from(prop: BlockProperty) -> Self {
            Self {
                block_name: prop.block_name,
                prop_name: prop.name_pc,
                prop_vars: prop.variants,
            }
        }
    }

    impl ToTokens for PropertyConvert {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_enum_name = self.prop_name.to_ident();
            match self.prop_vars.iter().next() {
                Some(BlockPropertyVariant::Bool(_, _)) => {
                    PropertyFromBool(&prop_enum_name).to_tokens(tokens);
                    PropertyIntoBool(&prop_enum_name).to_tokens(tokens);
                }
                Some(BlockPropertyVariant::Numeric(_, _)) => {
                    let impl_from_u8 = PropertyFromU8 {
                        enum_name: &prop_enum_name,
                        vars: &self.prop_vars,
                    };
                    impl_from_u8.to_tokens(tokens);
                }
                _ => {}
            }
        }
    }

    pub struct PropertyFromU8<'a> {
        pub enum_name: &'a Ident,
        pub vars: &'a Vec<BlockPropertyVariant>,
    }

    impl<'a> ToTokens for PropertyFromU8<'a> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut match_arms = TokenStream::new();
            for prop_var in self.vars.iter() {
                // "4"
                let prop_ord = Literal::u8_unsuffixed(match prop_var {
                    BlockPropertyVariant::Numeric(n, _) => *n,
                    _ => unreachable!(),
                });

                // "AzaleaLeavesDistance::IV"
                let prop_var_path = PropertyVariantPath {
                    enum_name: &self.enum_name,
                    var_name: &prop_var.defused_name().to_ident(),
                };

                // "AzaleaLeavesDistance::IV => 4,"
                let match_arm = quote! { #prop_ord => #prop_var_path, };
                match_arm.to_tokens(&mut match_arms);
            }

            let prop_enum_name = self.enum_name;

            tokens.extend(quote! {
                impl TryFrom<u8> for #prop_enum_name {
                    type Error = super::UnknownVar;
                    fn try_from(n: u8) -> Result<Self, Self::Error> {
                        Ok(match n {
                            #match_arms
                            _ => Err(super::UnknownVar(n))?,
                        })
                    }
                }
            });
        }
    }

    pub struct PropertyFromBool<'a>(&'a Ident);

    impl<'a> ToTokens for PropertyFromBool<'a> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_enum_name = self.0;
            tokens.extend(quote! {
                impl From<bool> for #prop_enum_name {
                    fn from(value: bool) -> Self {
                        if value {
                            Self::True
                        } else {
                            Self::False
                        }
                    }
                }
            });
        }
    }

    pub struct PropertyIntoBool<'a>(&'a Ident);

    impl<'a> ToTokens for PropertyIntoBool<'a> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_enum_name = self.0;
            tokens.extend(quote! {
                impl From<#prop_enum_name> for bool {
                    fn from(value: #prop_enum_name) -> Self {
                        value == #prop_enum_name::True
                    }
                }
            });
        }
    }

    pub struct PropertyVariantPath<'a> {
        enum_name: &'a Ident, // Berries
        var_name: &'a Ident,  // True
    }

    impl ToTokens for PropertyVariantPath<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let enum_name = self.enum_name;
            let var_name = self.var_name;

            // "Wall::East"
            tokens.extend(quote!(#enum_name::#var_name));
        }
    }
}

trait StringExt {
    fn to_ident(&self) -> proc_macro2::Ident;
}

impl<T: AsRef<str>> StringExt for T {
    fn to_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(self.as_ref(), proc_macro2::Span::call_site())
    }
}
