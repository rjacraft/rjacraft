use convert_case::{Case, Casing};
use proc_macro2::*;
use quote::*;
use syn::*;

fn handle_bitfield(item: ItemStruct) -> TokenStream {
    let struct_name = &item.ident;

    quote! {
        impl ProtocolType for #struct_name {
            type DecodeError = error::Eof;
            type EncodeError = error::Infallible;

            fn decode(buffer: &mut impl ::bytes::Buf) -> Result<Self, Self::DecodeError> {
                Ok(Self(Primitive::decode(buffer)?.0))
            }

            fn encode(&self, buffer: &mut impl ::bytes::BufMut) -> Result<(), Self::EncodeError> {
                Primitive(self.0).encode(buffer)?;

                Ok(())
            }
        }
    }
}

fn handle_struct(item: ItemStruct) -> TokenStream {
    let struct_name = &item.ident;
    let de_error = format_ident!("{struct_name}DecodeError");
    let en_error = format_ident!("{struct_name}EncodeError");

    let mut de_errors = Vec::new();
    let mut en_errors = Vec::new();
    let mut de_fields = Vec::new();
    let mut en_fields = Vec::new();

    for (i, field) in item.fields.iter().enumerate() {
        let ty = &field.ty;
        let display_name = if let Some(x) = &field.ident {
            x.to_string()
        } else {
            i.to_string()
        };
        let error_name = Ident::new(
            &if let Some(x) = &field.ident {
                x.to_string().to_case(Case::Pascal)
            } else {
                format!("F{i}")
            },
            Span::call_site(),
        );
        let access_name = if let Some(x) = &field.ident {
            quote!(#x)
        } else {
            Index::from(i).into_token_stream()
        };

        de_errors.push(quote! {
            #[error("Failed to decode field {}", #display_name)]
            #error_name(#[source] <#ty as ProtocolType>::DecodeError),
        });
        en_errors.push(quote! {
            #[error("Failed to encode field {}", #display_name)]
            #error_name(#[source] <#ty as ProtocolType>::EncodeError),
        });

        if let Some(name) = &field.ident {
            de_fields.push(quote! {
                #name: <#ty as ProtocolType>::decode(buffer)
                    .map_err(#de_error::#error_name)?,
            });
        } else {
            de_fields.push(quote! {
                <#ty as ProtocolType>::decode(buffer)
                    .map_err(#de_error::#error_name)?,
            });
        }

        en_fields.push(quote! {
            ProtocolType::encode(&self.#access_name, buffer)
                .map_err(#en_error::#error_name)?;
        });
    }

    let de_construct = match item.fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(_) => quote! { (#(#de_fields)*) },
        Fields::Named(_) => quote! { { #(#de_fields)* } },
    };

    quote! {
        #[derive(Debug, ::thiserror::Error)]
        pub enum #de_error {
            #(#de_errors)*
        }

        #[derive(Debug, ::thiserror::Error)]
        pub enum #en_error {
            #(#en_errors)*
        }

        impl ProtocolType for #struct_name {
            type DecodeError = #de_error;
            type EncodeError = #en_error;

            fn decode(buffer: &mut impl ::bytes::Buf) -> Result<Self, Self::DecodeError> {
                Ok(Self #de_construct)
            }

            fn encode(&self, buffer: &mut impl ::bytes::BufMut) -> Result<(), Self::EncodeError> {
                #(#en_fields)*

                Ok(())
            }
        }
    }
}

fn handle_enum(item: ItemEnum) -> TokenStream {
    let enum_name = &item.ident;
    let de_error = format_ident!("{enum_name}DecodeError");
    let en_error = format_ident!("{enum_name}EncodeError");

    let Some(disc_attr) = item
        .attrs
        .iter()
        .find(|it| it.meta.path().is_ident("variant"))
    else {
        return quote! { compile_error!("a variant type is required"); };
    };
    let Ok(disc_type) = disc_attr.parse_args::<Type>() else {
        return quote! { compile_error!("failed to parse variant type"); };
    };
    let mut disc_type_turbofish = disc_type.clone();

    if let Type::Path(path) = &mut disc_type_turbofish {
        for segment in &mut path.path.segments {
            if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                args.colon2_token = Some(parse_quote!(::));
            }
        }
    }

    let mut de_errors = Vec::new();
    let mut en_errors = Vec::new();
    let mut de_variants = Vec::new();
    let mut en_variants = Vec::new();

    for variant in &item.variants {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        let missing_disc = format!("missing variant value for `{}`", variant.ident);
        let invalid_disc = format!("failed to parse variant value for `{}`", variant.ident);

        let Some(disc_attr) = variant
            .attrs
            .iter()
            .find(|it| it.meta.path().is_ident("variant"))
        else {
            return quote! { compile_error!(#missing_disc); };
        };
        let Ok(disc_value) = disc_attr.parse_args::<Expr>() else {
            return quote! { compile_error!(#invalid_disc); };
        };

        let mut de_fields = Vec::new();
        let mut en_fields = Vec::new();
        let mut field_names = Vec::new();

        for (i, field) in variant.fields.iter().enumerate() {
            let ty = &field.ty;
            let display_name = if let Some(x) = &field.ident {
                x.to_string()
            } else {
                i.to_string()
            };
            let error_name = Ident::new(
                &if let Some(x) = &field.ident {
                    format!("{variant_name_str}{}", x.to_string().to_case(Case::Pascal))
                } else {
                    format!("{variant_name_str}F{i}")
                },
                Span::call_site(),
            );
            let access_name = if let Some(x) = field.ident.clone() {
                x
            } else {
                format_ident!("f{i}")
            };

            de_errors.push(quote! {
                #[error("Failed to decode field {} in {}", #display_name, #variant_name_str)]
                #error_name(#[source] <#ty as ProtocolType>::DecodeError),
            });
            en_errors.push(quote! {
                #[error("Failed to encode field {} in {}", #display_name, #variant_name_str)]
                #error_name(#[source] <#ty as ProtocolType>::EncodeError),
            });

            if let Some(name) = &field.ident {
                de_fields.push(quote! {
                    #name: <#ty as ProtocolType>::decode(buffer)
                        .map_err(#de_error::#error_name)?,
                });
            } else {
                de_fields.push(quote! {
                    <#ty as ProtocolType>::decode(buffer)
                        .map_err(#de_error::#error_name)?,
                });
            }

            en_fields.push(quote! {
                ProtocolType::encode(#access_name, buffer)
                    .map_err(#en_error::#error_name)?;
            });

            field_names.push(access_name);
        }

        let de_construct = match &variant.fields {
            Fields::Unit => quote!(),
            Fields::Unnamed(_) => quote! { (#(#de_fields)*) },
            Fields::Named(_) => quote! { { #(#de_fields)* } },
        };
        let en_construct = match &variant.fields {
            Fields::Unit => quote!(),
            Fields::Unnamed(_) => quote! { (#(#field_names),*) },
            Fields::Named(_) => quote! { { #(#field_names),* } },
        };

        de_variants.push(quote! {
            #disc_value => Ok(Self::#variant_name #de_construct),
        });
        en_variants.push(quote! {
            Self::#variant_name #en_construct => {
                #disc_type_turbofish::from(#disc_value).encode(buffer).map_err(#en_error::Discriminator)?;

                #(#en_fields)*
            },
        });
    }

    quote! {
        #[derive(Debug, ::thiserror::Error)]
        pub enum #de_error {
            #[error("Failed to parse enum discriminator")]
            Discriminator(<#disc_type as ProtocolType>::DecodeError),
            #(#de_errors)*
            #[error("Enum out of range: {0:?}")]
            OutOfRange(#disc_type),
        }

        #[derive(Debug, ::thiserror::Error)]
        pub enum #en_error {
            #[error("Failed to encode enum discriminator")]
            Discriminator(<#disc_type as ProtocolType>::EncodeError),
            #(#en_errors)*
        }

        impl ProtocolType for #enum_name {
            type DecodeError = #de_error;
            type EncodeError = #en_error;

            fn decode(buffer: &mut impl ::bytes::Buf) -> Result<Self, Self::DecodeError> {
                let disc = #disc_type_turbofish::decode(buffer).map_err(#de_error::Discriminator)?.into();

                match disc {
                    #(#de_variants)*
                    x => Err(#de_error::OutOfRange(x.into())),
                }
            }

            fn encode(&self, buffer: &mut impl ::bytes::BufMut) -> Result<(), Self::EncodeError> {
                match self {
                    #(#en_variants)*
                }

                Ok(())
            }
        }
    }
}

pub fn handle(item: Item) -> TokenStream {
    match item {
        Item::Struct(item) => {
            if item
                .attrs
                .iter()
                .any(|it| it.meta.path().is_ident("bitfield"))
            {
                handle_bitfield(item)
            } else {
                handle_struct(item)
            }
        }
        Item::Enum(item) => handle_enum(item),
        _ => {
            quote! { compile_error!("unsupported item type"); }
        }
    }
}
