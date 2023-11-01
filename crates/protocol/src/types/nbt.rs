//! An unnamed NBT compound

use std::{error, io};

use bytes::Buf;

use crate::ProtocolType;

pub trait AsCompound: Sized {
    type DecodeError: error::Error + 'static;
    type EncodeError: error::Error + 'static;

    // fn from_nbt(compound: valence_nbt::Compound) -> Result<Self, Self::DecodeError>;
    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError>;
}

impl<T> AsCompound for T
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type DecodeError = valence_nbt::serde::Error;
    type EncodeError = valence_nbt::serde::Error;

    // fn from_nbt(compound: valence_nbt::Compound) -> Result<Self, Self::DecodeError> {
    //     Self::deserialize(compound)
    // }

    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError> {
        self.serialize(valence_nbt::serde::CompoundSerializer)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    #[error("Error reading/writing the structure")]
    Struct(T),
    #[error("Error reading/writing the compound")]
    Compound(#[from] valence_nbt::binary::Error),
}

#[derive(Debug, Clone)]

pub struct Nbt<T>(pub T);

impl<T: AsCompound> ProtocolType for Nbt<T> {
    type DecodeError = Error<T::DecodeError>;
    type EncodeError = Error<T::EncodeError>;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        // ugly-ass workaround to omit the root compound name (which is the new standard)

        let mut temp = io::Cursor::new(Vec::new());
        valence_nbt::binary::to_binary(&self.0.to_nbt().map_err(Error::Struct)?, &mut temp, "")?;
        temp.set_position(0);
        buffer.put_u8(temp.get_u8());
        temp.advance(2);
        buffer.put(temp);

        Ok(())
    }
}
