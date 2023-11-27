//! An unnamed NBT compound

use std::io;

use bytes::{Buf, BufMut};

use crate::{error, ProtocolType};

pub trait AsCompound: Sized {
    type DecodeError: error::Error + 'static;
    type EncodeError: error::Error + 'static;

    fn from_nbt(compound: valence_nbt::Compound) -> Result<Self, Self::DecodeError>;
    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError>;
}

impl<T> AsCompound for T
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    type DecodeError = valence_nbt::serde::Error;
    type EncodeError = valence_nbt::serde::Error;

    fn from_nbt(compound: valence_nbt::Compound) -> Result<Self, Self::DecodeError> {
        Self::deserialize(compound)
    }

    fn to_nbt(&self) -> Result<valence_nbt::Compound, Self::EncodeError> {
        self.serialize(valence_nbt::serde::CompoundSerializer)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError<E> {
    #[error(transparent)]
    Eof(error::Eof),
    #[error("Error reading the structure")]
    Struct(E),
    #[error("Error reading the compound")]
    Compound(#[from] valence_nbt::binary::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError<E> {
    #[error("Error writing the structure")]
    Struct(E),
    #[error("Error writing the compound")]
    Compound(#[from] valence_nbt::binary::Error),
}

#[derive(Debug, Clone)]

pub struct Nbt<T>(pub T);

impl<T: AsCompound> ProtocolType for Nbt<T> {
    // ugly-ass workaround to omit the root compound name (which is the new standard)

    type DecodeError = DecodeError<T::DecodeError>;
    type EncodeError = EncodeError<T::EncodeError>;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() < 1 {
            Err(DecodeError::Eof(error::Eof))?;
        }

        let mut temp = bytes::BytesMut::new();
        temp.put_u8(buffer.get_u8());
        temp.put_u16(0);
        temp.put(buffer.chunk());

        let (comp, root_name) =
            valence_nbt::binary::from_binary::<String>(&mut temp.freeze().chunk())?;

        buffer.advance(valence_nbt::binary::written_size(&comp, &root_name) - 3);

        Ok(Nbt(T::from_nbt(comp).map_err(DecodeError::Struct)?))
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        let mut temp = io::Cursor::new(Vec::new());
        valence_nbt::binary::to_binary(
            &self.0.to_nbt().map_err(EncodeError::Struct)?,
            &mut temp,
            "",
        )?;
        temp.set_position(0);
        buffer.put_u8(temp.get_u8());
        temp.advance(2);
        buffer.put(temp);

        Ok(())
    }
}

impl<T: AsCompound> ProtocolType for Option<Nbt<T>> {
    // ugly-ass workaround to omit the root compound name (which is the new standard)

    type DecodeError = DecodeError<T::DecodeError>;
    type EncodeError = EncodeError<T::EncodeError>;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        if buffer.remaining() < 1 {
            Err(DecodeError::Eof(error::Eof))?;
        }

        let type_id = buffer.get_u8();

        if type_id == 0 {
            return Ok(None);
        }

        let mut temp = bytes::BytesMut::new();
        temp.put_u8(type_id);
        temp.put_u16(0);
        temp.put(buffer.chunk());

        let (comp, root_name) =
            valence_nbt::binary::from_binary::<String>(&mut temp.freeze().chunk())?;

        buffer.advance(valence_nbt::binary::written_size(&comp, &root_name) - 3);

        Ok(Some(Nbt(T::from_nbt(comp).map_err(DecodeError::Struct)?)))
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        if let Some(x) = self {
            x.encode(buffer)
        } else {
            buffer.put_u8(0);

            Ok(())
        }
    }
}

pub mod snbt {
    use serde::{de, ser, Deserialize, Serialize};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ser::Serialize,
        S: ser::Serializer,
    {
        valence_nbt::snbt::to_snbt_string(&valence_nbt::Value::Compound(
            value
                .serialize(valence_nbt::serde::CompoundSerializer)
                .map_err(ser::Error::custom)?,
        ))
        .serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: de::DeserializeOwned,
        D: de::Deserializer<'de>,
    {
        T::deserialize(
            valence_nbt::snbt::from_snbt_str(<&str>::deserialize(deserializer)?)
                .map_err(de::Error::custom)?,
        )
        .map_err(de::Error::custom)
    }
}
