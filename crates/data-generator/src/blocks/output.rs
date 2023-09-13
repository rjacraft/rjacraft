use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::ToTokens;

use super::model::Block;
use crate::name::Name;

pub fn gen_blocks_mod(blocks: IndexMap<Name, Block>) -> TokenStream {
    blocks_mod::BlocksMod::new(blocks).to_token_stream()
}

mod blocks_mod {
    use indexmap::IndexMap;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use crate::{blocks::model::Block, name::Name};

    pub struct BlocksMod {
        pub blocks: IndexMap<Name, Block>,
    }

    impl BlocksMod {
        pub fn new(blocks: IndexMap<Name, Block>) -> Self {
            Self { blocks }
        }
    }

    impl ToTokens for BlocksMod {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            use super::{block_exports::BlockExports, block_mod::BlockMod};

            let block_mods = self
                .blocks
                .iter()
                .map(|(name, block)| BlockMod { name, block });
            let exports = BlockExports::new(self.blocks.keys());

            tokens.extend(quote! {
                pub mod blocks {
                    #[derive(Debug)]
                    pub struct UnknownId(u32);
                    #[derive(Debug)]
                    pub struct UnknownVar(u8);

                    #exports
                    #(#block_mods)*
                }
            });
        }
    }
}

mod block_exports {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use crate::name::Name;

    pub struct BlockExports<'a> {
        mod_block_names: Vec<&'a Name>,
    }

    impl<'a> BlockExports<'a> {
        pub fn new(blocks: impl IntoIterator<Item = &'a Name>) -> Self {
            Self {
                mod_block_names: blocks.into_iter().collect(),
            }
        }
    }

    impl ToTokens for BlockExports<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut exports = TokenStream::new();
            for name in self.mod_block_names.iter() {
                let block_mod_name = name.snake_case();
                let block_struct_name = name.pascal_case();
                exports.extend(quote!(#block_mod_name::Block as #block_struct_name,))
            }

            // "pub use self::cave_vines::Block as CaveVines;"
            tokens.extend(quote!(pub use self::{#exports};))
        }
    }
}

mod block_mod {
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};

    use crate::{blocks::model::Block, name::Name};

    pub struct BlockMod<'a> {
        pub name: &'a Name,
        pub block: &'a Block,
    }

    impl ToTokens for BlockMod<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let block_mod_name = self.name.snake_case();
            let block_code = self.gen_block_code();
            let props_code = self.gen_props_code();

            tokens.extend(quote! {
                pub mod #block_mod_name {
                    #block_code
                    #props_code
                }
            });
        }
    }

    impl BlockMod<'_> {
        fn gen_block_code(&self) -> TokenStream {
            let mut tokens = TokenStream::new();
            super::block_struct::BlockStruct::new(self.block).to_tokens(&mut tokens);
            super::block_convert::BlockConvert::from(self.block).to_tokens(&mut tokens);
            tokens
        }

        fn gen_props_code(&self) -> TokenStream {
            use super::{prop_convert::*, prop_default::*, prop_enum::*};

            let mut tokens = TokenStream::new();
            for (prop_name, variants) in self.block.properties.iter() {
                let (_, def_state) = self.block.states.default();
                let def_value = def_state
                    .properties
                    .get(prop_name)
                    .expect("property is present");

                PropertyEnum::new(prop_name, variants).to_tokens(&mut tokens);
                PropertyDefault::new(prop_name, def_value).to_tokens(&mut tokens);
                PropertyConvert::new(prop_name, variants).to_tokens(&mut tokens);
            }
            tokens
        }
    }
}

mod block_struct {
    use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt as _};

    use crate::blocks::model::Block;

    pub struct BlockStruct<'a> {
        pub properties: Vec<BlockStructField<'a>>,
    }

    impl<'a> BlockStruct<'a> {
        pub fn new(block: &'a Block) -> Self {
            let properties = block
                .properties
                .iter()
                .map(|(name, _)| BlockStructField {
                    prop_name: name.snake_case(),
                    prop_enum_name: name.pascal_case(),
                })
                .collect();

            Self { properties }
        }
    }

    impl ToTokens for BlockStruct<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote! {
                #[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
                pub struct Block
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

    pub struct BlockStructField<'a> {
        pub prop_name: &'a Ident,      // honey_level
        pub prop_enum_name: &'a Ident, // HoneyLevel
    }

    impl ToTokens for BlockStructField<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_name = self.prop_name;
            let prop_enum_name = self.prop_enum_name;

            // "pub open: Open,"
            tokens.extend(quote!(pub #prop_name: #prop_enum_name,))
        }
    }
}

mod block_convert {
    use proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream};
    use quote::{quote, ToTokens, TokenStreamExt as _};

    use crate::blocks::model::{Block, Id, State};

    #[derive(Debug)]
    pub struct BlockConvert<'a> {
        pub id_states: Vec<(Id, BlockStateInst<'a>)>,
    }

    impl<'a> From<&'a Block> for BlockConvert<'a> {
        fn from(block: &'a Block) -> Self {
            let id_states = block
                .states
                .states
                .iter()
                .map(|(block_id, block_state)| (block_id.clone(), block_state.into()))
                .collect();

            Self { id_states }
        }
    }

    impl ToTokens for BlockConvert<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            BlockIntoU32(self).to_tokens(tokens);
            BlockFromU32(self).to_tokens(tokens);
        }
    }

    #[derive(Debug)]
    pub struct BlockIntoU32<'a>(&'a BlockConvert<'a>);

    impl ToTokens for BlockIntoU32<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut match_arms = TokenStream::new();
            for (id, state_inst) in &self.0.id_states {
                // "RedstoneLamp { Lit: Lit::True, } => 7417,"
                let match_arm = quote! { #state_inst => #id, };
                match_arm.to_tokens(&mut match_arms);
            }

            // It is assumed that the states property covers all possible
            // states for every given block. In other words, the match is
            // supposed to be exhaustive.
            tokens.extend(quote! {
                impl From<Block> for u32 {
                    fn from(state: Block) -> Self {
                        match state {
                            #match_arms
                        }
                    }
                }
            });
        }
    }

    #[derive(Debug)]
    pub struct BlockFromU32<'a>(&'a BlockConvert<'a>);

    impl ToTokens for BlockFromU32<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut match_arms = TokenStream::new();
            for (id, state_inst) in &self.0.id_states {
                // "4287 => Farmland,"
                let match_arm = quote! { #id => #state_inst, };
                match_arm.to_tokens(&mut match_arms);
            }

            tokens.extend(quote! {
                impl TryFrom<u32> for Block {
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
    pub struct BlockStateInst<'a> {
        pub properties: Vec<BlockStateProperty<'a>>,
    }

    impl<'a> From<&'a State> for BlockStateInst<'a> {
        fn from(block_state: &'a State) -> Self {
            let properties = block_state
                .properties
                .iter()
                .map(|block_prop| BlockStateProperty {
                    field_name: block_prop.0.snake_case(),
                    enum_name: block_prop.0.pascal_case(),
                    var_name: block_prop.1.variant().into_pascal_case(),
                })
                .collect();

            BlockStateInst { properties }
        }
    }

    impl ToTokens for BlockStateInst<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            // "Ladder"
            tokens.extend(quote!(Block));

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
    pub struct BlockStateProperty<'a> {
        pub field_name: &'a Ident, // instrument
        pub enum_name: &'a Ident,  // NoteBlock
        pub var_name: Ident,       // IronXylophone
    }

    impl ToTokens for BlockStateProperty<'_> {
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
    use proc_macro2::{Ident, Literal, TokenStream};
    use quote::{quote, ToTokens};

    use crate::{blocks::model::PropertyVariants, name::Name};

    pub struct PropertyEnum<'a> {
        pub prop_name: &'a Ident, // Powered
        pub prop_vars: Vec<(Ident, Option<Literal>)>,
    }

    impl<'a> PropertyEnum<'a> {
        pub fn new(name: &'a Name, variants: &PropertyVariants) -> Self {
            Self {
                prop_name: name.pascal_case(),
                prop_vars: variants
                    .as_enum()
                    .into_iter()
                    .map(|(name, ord)| {
                        (
                            name.into_pascal_case(),
                            ord.map(|x| Literal::u8_unsuffixed(x)),
                        )
                    })
                    .collect(),
            }
        }
    }

    impl ToTokens for PropertyEnum<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            // "Distance"
            let prop_name = self.prop_name;

            // "Ord, PartialOrd,"
            let derive = self
                .prop_vars
                .first()
                .filter(|(_, ord)| ord.is_some())
                .map(|_| quote!(Ord, PartialOrd,));

            // "I = 1, II = 2, III = 3,"
            let vars = self.prop_vars.iter().map(|(ident, ord)| {
                let ord = ord.as_ref().map(|x| quote!(= #x));
                quote!(#ident #ord,)
            });

            tokens.extend(quote! {
                #[derive(Debug, #derive Eq, PartialEq, Copy, Clone, Hash)]
                pub enum #prop_name {
                    #( #vars )*
                }
            });
        }
    }
}

mod prop_default {
    use proc_macro2::{Ident, TokenStream};
    use quote::{quote, ToTokens};

    use crate::{blocks::model::PropertyVariant, name::Name};

    #[derive(Debug)]
    pub struct PropertyDefault<'a> {
        pub enum_name: &'a Ident, // WeepingVines
        pub def_var_name: Ident,
    }

    impl<'a> PropertyDefault<'a> {
        pub fn new(name: &'a Name, value: &PropertyVariant) -> Self {
            PropertyDefault {
                enum_name: name.pascal_case(),
                def_var_name: value.variant().into_pascal_case(),
            }
        }
    }

    impl ToTokens for PropertyDefault<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let enum_name = &self.enum_name;
            let def_var_name = &self.def_var_name;

            tokens.extend(quote! {
                impl Default for #enum_name {
                    fn default() -> Self {
                        Self::#def_var_name
                    }
                }
            });
        }
    }
}

mod prop_convert {
    use proc_macro2::{Ident, Literal, TokenStream};
    use quote::{quote, ToTokens};

    use crate::{
        blocks::model::{PropertyVariant, PropertyVariants},
        name::Name,
    };

    pub struct PropertyConvert<'a> {
        pub prop_name: &'a Ident, // HoneyLevel
        pub prop_vars: &'a PropertyVariants,
    }

    impl<'a> PropertyConvert<'a> {
        pub fn new(name: &'a Name, variants: &'a PropertyVariants) -> Self {
            Self {
                prop_name: name.pascal_case(),
                prop_vars: variants,
            }
        }
    }

    impl ToTokens for PropertyConvert<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match &self.prop_vars {
                PropertyVariants::Bool => {
                    PropertyNotBool(&self.prop_name).to_tokens(tokens);
                    PropertyFromBool(&self.prop_name).to_tokens(tokens);
                    PropertyIntoBool(&self.prop_name).to_tokens(tokens);
                }
                PropertyVariants::Numeric(numbers) => {
                    let impl_from_u8 = PropertyFromU8 {
                        enum_name: &self.prop_name,
                        numbers: &numbers,
                    };
                    impl_from_u8.to_tokens(tokens);
                }
                _ => {}
            }
        }
    }

    pub struct PropertyFromU8<'a> {
        pub enum_name: &'a Ident,
        pub numbers: &'a [u8],
    }

    impl ToTokens for PropertyFromU8<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            // "Distance"
            let enum_name = self.enum_name;

            // ["1 => Self::I", "2 => Self::II", "3 => Self::III"]
            let match_arms: Vec<_> = self
                .numbers
                .iter()
                .copied()
                .map(|num| {
                    let u8_num = Literal::u8_unsuffixed(num); // "4"
                    let roman_num = PropertyVariant::Numeric(num).variant();
                    let roman_num = roman_num.into_pascal_case(); // "IV"
                    quote! { #u8_num => Self::#roman_num }
                })
                .collect();

            tokens.extend(quote! {
                impl TryFrom<u8> for #enum_name {
                    type Error = super::UnknownVar;

                    fn try_from(n: u8) -> Result<Self, Self::Error> {
                        Ok(match n {
                            #( #match_arms, )*
                            _ => Err(super::UnknownVar(n))?,
                        })
                    }
                }
            });
        }
    }

    pub struct PropertyFromBool<'a>(&'a Ident);

    impl ToTokens for PropertyFromBool<'_> {
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

    impl ToTokens for PropertyIntoBool<'_> {
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

    pub struct PropertyNotBool<'a>(&'a Ident);

    impl ToTokens for PropertyNotBool<'_> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let prop_enum_name = self.0;
            tokens.extend(quote! {
                impl std::ops::Not for #prop_enum_name {
                    type Output = Self;

                    fn not(self) -> Self::Output {
                        if self == Self::False {
                            Self::True
                        } else {
                            Self::False
                        }
                    }
                }
            });
        }
    }
}
