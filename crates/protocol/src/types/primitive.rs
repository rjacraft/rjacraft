//! A wrapper for the most intuitive implementations of the primitive types

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

#[derive(Debug, Clone)]
pub struct Primitive<T>(pub T);

macro_rules! primitive_impl {
    ($($type:ty, $net_size:literal, $read_fn:tt, $write_fn:tt),* $(,)?) => {
        $(
            impl ProtocolType for Primitive<$type> {
                type DecodeError = error::Eof;
                type EncodeError = error::Infallible;

                fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
                    if buffer.remaining() >= $net_size {
                        Ok(Self(buffer.$read_fn()))
                    } else {
                        Err(error::Eof)
                    }
                }

                fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
                    buffer.$write_fn(self.0);
                    Ok(())
                }
            }

            impl From<$type> for Primitive<$type> {
                fn from(value: $type) -> Self {
                    Self(value)
                }
            }

            impl From<Primitive<$type>> for $type {
                fn from(value: Primitive<$type>) -> Self {
                    value.0
                }
            }
        )*
    }
}

primitive_impl! {
    u8, 1, get_u8, put_u8,
    u16, 2, get_u16, put_u16,
    u32, 4, get_u32, put_u32,
    u64, 8, get_u64, put_u64,

    i8, 1, get_i8, put_i8,
    i16, 2, get_i16, put_i16,
    i32, 4, get_i32, put_i32,
    i64, 8, get_i64, put_i64,

    f32, 4,get_f32, put_f32,
    f64, 8, get_f64, put_f64,
}

impl ProtocolType for Primitive<bool> {
    type DecodeError = error::Eof;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() >= 1 {
            Ok(Self(buffer.get_u8() != 0))
        } else {
            Err(error::Eof)
        }
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        buffer.put_u8(if self.0 { 1 } else { 0 });
        Ok(())
    }
}

impl From<bool> for Primitive<bool> {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<Primitive<bool>> for bool {
    fn from(value: Primitive<bool>) -> Self {
        value.0
    }
}
