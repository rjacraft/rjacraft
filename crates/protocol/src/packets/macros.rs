//! These are private so fuck the hygiene

/// Struct-like packets
macro_rules! packets_struct {
    {
        $(
            $ident:ident {
                $(
                    $field:ident: $type:ty;
                )*
            }
        )*
    } => {
        paste::paste! { $(
            #[derive(Debug, thiserror::Error)]
            pub enum [<$ident DecodeError>] {
                $(
                    #[error("Failed to decode field {}", stringify!($field))]
                    [<$field:camel>](
                        #[source] <$type as crate::ProtocolType>::DecodeError
                    ),
                )*
            }

            #[derive(Debug, thiserror::Error)]
            pub enum [<$ident EncodeError>] {
                $(
                    #[error("Failed to encode field {}", stringify!($field))]
                    [<$field:camel>](
                        #[source] <$type as crate::ProtocolType>::EncodeError
                    ),
                )*
            }

            #[derive(Debug, Clone)]
            pub struct $ident {
                $(
                    pub $field: $type,
                )*
            }

            impl crate::ProtocolType for $ident {
                type DecodeError = [<$ident DecodeError>];
                type EncodeError = [<$ident EncodeError>];

                #[allow(unused_variables)]
                fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
                    $(
                        let $field = <$type as crate::ProtocolType>
                            ::decode(buffer)
                            .map_err([<$ident DecodeError>]::[<$field:camel>])?;
                    )*

                    Ok(Self {
                        $(
                            $field,
                        )*
                    })
                }

                #[allow(unused_variables)]
                fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
                    $(
                        crate::ProtocolType
                            ::encode(&self.$field, buffer)
                            .map_err([<$ident EncodeError>]::[<$field:camel>])?;
                    )*

                    Ok(())
                }
            }
        )* }
    };
}

pub(crate) use packets_struct;

/// Sum type/tagged union/Rust enum-like packets
macro_rules! packet_sumtype {
    {
        $(
            $ident:ident {
                $($opcode:literal = $variant:ident,)*
            }
        )*
    } => {
        paste::paste! { $(
            #[derive(Debug, thiserror::Error)]
            pub enum [<$ident DecodeError>] {
                #[error("Failed to parse enum discriminator")]
                Discriminator(crate::types::varint::DecodeError),
                $(
                    #[error("Failed to decode variant {}", stringify!($variant))]
                    $variant(
                        #[source] <$variant as crate::ProtocolType>::DecodeError
                    ),
                )*
                #[error("Enum out of range: {0}")]
                OutOfRange(i32),
            }

            #[derive(Debug, thiserror::Error, nevermore::FromNever)]
            pub enum [<$ident EncodeError>] {
                $(
                    #[error("Failed to encode variant {}", stringify!($variant))]
                    $variant(
                        #[source] <$variant as crate::ProtocolType>::EncodeError
                    ),
                )*
            }

            #[derive(Debug, Clone)]
            pub enum $ident {
                $(
                    $variant($variant),
                )*
            }

            impl crate::ProtocolType for $ident {
                type DecodeError = [<$ident DecodeError>];
                type EncodeError = [<$ident EncodeError>];

                #[allow(unused_variables)]
                fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
                    let crate::types::VarInt(opcode) = crate::types::VarInt
                        ::decode(buffer)
                        .map_err([<$ident DecodeError>]::Discriminator)?;

                    match opcode {
                        $(
                            $opcode => Ok($ident::$variant(
                                <$variant as crate::ProtocolType>
                                    ::decode(buffer)
                                    .map_err([<$ident DecodeError>]::$variant)?
                            )),
                        )*
                        x => Err([<$ident DecodeError>]::OutOfRange(x)),
                    }
                }

                #[allow(unused_variables, unreachable_patterns)]
                fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
                    match self {
                        $(
                            $ident::$variant(value) => {
                                crate::types::VarInt($opcode).encode(buffer)?;

                                crate::ProtocolType
                                    ::encode(value, buffer)
                                    .map_err([<$ident EncodeError>]::$variant)?;
                            }
                        )*
                        _ => {} // needed for empty enums
                    }

                    Ok(())
                }
            }
        )* }
    };
}

pub(crate) use packet_sumtype;
