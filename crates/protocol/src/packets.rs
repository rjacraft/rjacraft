use uuid::Uuid;

use crate::{io::Decoder, LengthInferredVecU8, VarInt, VarIntPrefixedVec};

macro_rules! user_type {
    (VarInt) => {
        i32
    };
    (VarIntPrefixedVec <$inner:ident>) => {
        Vec<$inner>
    };
    (ShortPrefixedVec <$inner:ident>) => {
        Vec<$inner>
    };
    (LengthInferredVecU8) => {
        Vec<u8>
    };
    ($typ:ty) => {
        $typ
    };
}

macro_rules! encoder_type {
    (VarInt, $e:expr) => {
        VarInt($e)
    };
    (VarIntPrefixedVec <$inner:ident>, $e:expr) => {
        VarIntPrefixedVec::from($e.as_slice())
    };
    (ShortPrefixedVec <$inner:ident>, $e:expr) => {
        ShortPrefixedVec::from($e.as_slice())
    };
    (LengthInferredVecU8, $e:expr) => {
        LengthInferredVecU8::from($e.as_slice())
    };
    ($typ:ty, $e:expr) => {
        $e
    };
}

macro_rules! packets {
    (
        $(
            $packet:ident {
                $(
                    $field:ident $typ:ident $(<$generics:ident>)?
                );* $(;)?
            } $(,)?
        )*
    ) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $packet {
                $(
                    pub $field: user_type!($typ $(<$generics>)?),
                )*
            }

            #[allow(clippy::useless_conversion)]
            #[allow(unused_imports)]
            #[allow(unused_variables)]
            impl crate::Decoder for $packet {
                fn decode(buffer: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
                    use anyhow::Context;

                    $(
                        let $field = <$typ $(<$generics>)?>::decode(buffer)
                                .context(concat!("failed to decode field `", stringify!($field), "` of packet `", stringify!($packet), "`"))?
                                .into();
                    )*

                    Ok(Self {
                        $(
                            $field,
                        )*
                    })
                }
            }

            #[allow(clippy::useless_conversion)]
            #[allow(unused_imports)]
            #[allow(unused_variables)]
            impl $crate::Encoder for $packet {
                fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
                    use anyhow::Context;

                    $(
                        encoder_type!{$typ $(<$generics>)?, self.$field}.encode(buffer)
                            .context(concat!("failed to encode field `", stringify!($field), "` of packet `", stringify!($packet), "`"))?;
                    )*

                    Ok(())
                }
            }
        )*
    };
}

macro_rules! enum_packets {
    (
        $ident:ident {
            $($opcode:literal = $packet:ident),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $ident {
            $(
                $packet($packet),
            )*
        }

        impl $crate::Decoder for $ident {
            fn decode(buffer: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
                let opcode = $crate::VarInt::decode(buffer)?.0;
                match opcode {
                    $(
                        $opcode => Ok($ident::$packet($packet::decode(buffer)?)),
                    )*
                    _ => Err(anyhow::anyhow!("invalid packet opcode {}", opcode)),
                }
            }
        }

        impl $crate::Encoder for $ident {
            fn encode(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
                match self {
                    $(
                        $ident::$packet(packet) => {
                            $crate::VarInt($opcode).encode(buffer)?;
                            packet.encode(buffer)?;
                        }
                    )*
                }
                Ok(())
            }
        }
    };
}

pub mod client;
pub mod server;
