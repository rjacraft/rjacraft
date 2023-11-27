//! An array of entity Propertys

use bytes::{Buf, BufMut};
use rjacraft_macro::ProtocolType;

use super::*;
use crate::{error, ProtocolType};

#[derive(Debug, Clone, Copy, PartialEq, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum Pose {
    #[variant(0)]
    Standing,
    #[variant(1)]
    FallFlying,
    #[variant(2)]
    Sleeping,
    #[variant(3)]
    Swimming,
    #[variant(4)]
    SpinAttack,
    #[variant(5)]
    Sneaking,
    #[variant(6)]
    LongJumping,
    #[variant(7)]
    Dying,
    #[variant(8)]
    Croaking,
    #[variant(9)]
    UsingTongue,
    #[variant(10)]
    Sitting,
    #[variant(11)]
    Roaring,
    #[variant(12)]
    Sniffing,
    #[variant(13)]
    Emerging,
    #[variant(14)]
    Digging,
}

macro_rules! property_types {
    { $($id:literal => $type:ty ,)* } => {
        paste::paste! {
            #[derive(Debug, Clone, ProtocolType)]
            #[variant(VarInt<i32>)]
            pub enum Property {
                $( #[variant($id)] [<V $id>]($type), )*
            }

            $(
                impl From<$type> for Property {
                    fn from(value: $type) -> Self {
                        Self::[<V $id>](value)
                    }
                }

                // todo the other way around
            )*
        }
    };
}

property_types! {
    0 => Primitive<u8>,
    1 => VarInt<i32>,
    2 => VarInt<i64>,
    3 => Primitive<f32>,
    4 => LenString<{ 1 << 15 }>,
    5 => JsonText,
    6 => BoolOption<JsonText>,
    7 => ItemStackProto,
    8 => Primitive<bool>,
    9 => (Primitive<f32>, Primitive<f32>, Primitive<f32>),
    10 => BlockPos,
    11 => BoolOption<BlockPos>,
    // 12 face,
    13 => BoolOption<Uuid>,
    // 14 block id
    // 15 block id
    16 => Nbt<valence_nbt::Compound>,
    // 17 particle
    // 18 villager-specific
    19 => BoolOption<VarInt<i32>>,
    20 => Pose,
    // 21 cat variant id
    // 22 frog variant id
    // 23 optional globalpos
    // 24 painting variant id
    // 25 sniffer state
    // 26 vec3f
    // 27 vec4f
}

#[derive(Debug, Clone)]
pub struct EntityDataValues(pub Vec<(u8, Property)>);

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum DecodeError {
    #[error(transparent)]
    Eof(#[from] error::Eof),
    #[error("Failed to decode Property")]
    Element(#[from] PropertyDecodeError),
}

impl ProtocolType for EntityDataValues {
    type DecodeError = DecodeError;
    type EncodeError = PropertyEncodeError;

    fn decode(_buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        for (index, property) in &self.0 {
            buffer.put_u8(*index);
            property.encode(buffer)?;
        }

        buffer.put_u8(0xff);

        Ok(())
    }
}

impl From<Vec<(u8, Property)>> for EntityDataValues {
    fn from(value: Vec<(u8, Property)>) -> Self {
        Self(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<EntityDataValues> for Vec<(u8, Property)> {
    fn from(value: EntityDataValues) -> Self {
        value.0.into_iter().map(|x| x.into()).collect()
    }
}
