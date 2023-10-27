//! An unnamed NBT compound

use std::io;

use bytes::{Buf, BufMut};

use crate::ProtocolType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error serializing the structure")]
    Serde(#[from] valence_nbt::serde::Error),
    #[error("Error encoding the binary data")]
    Binary(#[from] valence_nbt::binary::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct Nbt<T>(pub T);

impl<T> ProtocolType for Nbt<T>
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type DecodeError = Error;
    type EncodeError = Error;

    fn decode(_buffer: &mut impl Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl BufMut) -> Result<(), Self::EncodeError> {
        let compound: valence_nbt::Compound =
            self.0.serialize(valence_nbt::serde::CompoundSerializer)?;

        // ugly-ass workaround to omit the root compound name (which is the new standard)

        let mut temp = io::Cursor::new(Vec::new());
        valence_nbt::binary::to_binary(&compound, &mut temp, "")?;
        temp.set_position(0);
        buffer.put_u8(temp.get_u8());
        temp.advance(2);
        buffer.put(temp.chunk());

        Ok(())
    }
}

impl<T> From<T> for Nbt<T>
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}
