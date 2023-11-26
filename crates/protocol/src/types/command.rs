use bitfield_struct::bitfield;
use rjacraft_macro::ProtocolType;

use super::*;
use crate::{error, ProtocolType};

#[derive(ProtocolType)]
#[bitfield(u8)]
struct NodeFlags {
    #[bits(2)]
    pub node_type: u8,
    pub executable: bool,
    pub redirect: bool,
    pub suggestions: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(ProtocolType)]
#[bitfield(u8)]
struct BoundsFields {
    pub min: bool,
    pub max: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(Debug, Clone)]
pub struct Bounds<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T: ProtocolType> ProtocolType for Bounds<T> {
    type DecodeError = error::Eof;
    type EncodeError = T::EncodeError;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        BoundsFields::new()
            .with_min(self.min.is_some())
            .with_max(self.max.is_some())
            .encode(buffer)
            .ok();

        if let Some(min) = &self.min {
            min.encode(buffer)?;
        }

        if let Some(max) = &self.max {
            max.encode(buffer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum StringType {
    #[variant(0)]
    Word,
    #[variant(1)]
    Quoted,
    #[variant(2)]
    Greedy,
}

#[derive(ProtocolType)]
#[bitfield(u8)]
pub struct EntityFlags {
    pub only_specific: bool,
    pub only_players: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
    __: bool,
}

#[derive(Debug, Clone, ProtocolType)]
#[variant(VarInt<i32>)]
pub enum Parser {
    #[variant(0)]
    Bool,
    #[variant(1)]
    F32(Bounds<Primitive<f32>>),
    #[variant(2)]
    F64(Bounds<Primitive<f64>>),
    #[variant(3)]
    I32(Bounds<Primitive<i32>>),
    #[variant(4)]
    I64(Bounds<Primitive<i64>>),
    #[variant(5)]
    String(StringType),
    #[variant(6)]
    Entity(EntityFlags),
    #[variant(7)]
    GameProfile,
    #[variant(8)]
    BlockPos,
    #[variant(9)]
    ColumnPos,
    #[variant(10)]
    Vec3,
    #[variant(11)]
    Vec2,
}

#[derive(Debug, Clone)]
pub enum Node {
    Root {
        children: Vec<u32>,
    },
    Literal {
        executable: bool,
        children: Vec<u32>,
        redirect: Option<u32>,
        name: String,
    },
    Argument {
        executable: bool,
        children: Vec<u32>,
        redirect: Option<u32>,
        name: String,
        parser: Parser,
        suggestions: Option<Identifier>,
    },
}

impl ProtocolType for Node {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        match self {
            Self::Root { children } => {
                NodeFlags::new().with_node_type(0).encode(buffer)?;
                VarInt(children.len() as i32).encode(buffer)?;
                for &id in children {
                    VarInt(id as i32).encode(buffer)?;
                }
            }
            Self::Literal {
                executable,
                children,
                redirect,
                name,
            } => {
                NodeFlags::new()
                    .with_node_type(1)
                    .with_executable(*executable)
                    .with_redirect(redirect.is_some())
                    .encode(buffer)?;

                VarInt(children.len() as i32).encode(buffer)?;
                for &id in children {
                    VarInt(id as i32).encode(buffer)?;
                }

                if let &Some(id) = redirect {
                    VarInt(id as i32).encode(buffer)?;
                }

                LenString::<{ 1 << 15 }>::try_from(name.clone())
                    .expect("command name too long")
                    .encode(buffer)?;
            }
            Self::Argument {
                executable,
                children,
                redirect,
                name,
                parser,
                suggestions,
            } => {
                NodeFlags::new()
                    .with_node_type(2)
                    .with_executable(*executable)
                    .with_redirect(redirect.is_some())
                    .with_suggestions(suggestions.is_some())
                    .encode(buffer)?;

                VarInt(children.len() as i32).encode(buffer)?;
                for &id in children {
                    VarInt(id as i32).encode(buffer)?;
                }

                if let &Some(id) = redirect {
                    VarInt(id as i32).encode(buffer)?;
                }

                LenString::<{ 1 << 15 }>::try_from(name.clone())
                    .expect("command name too long")
                    .encode(buffer)?;

                parser.encode(buffer).unwrap();

                if let Some(id) = suggestions {
                    id.encode(buffer)?;
                }
            }
        }

        Ok(())
    }
}
