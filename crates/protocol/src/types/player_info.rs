use std::string;

use base64::Engine;
use bitfield_struct::bitfield;
use rjacraft_authlib::profile;
use rjacraft_macro::ProtocolType;

use super::*;
use crate::{error, ProtocolType};

#[derive(Debug, Clone, Copy, ProtocolType)]
#[variant(Primitive<u8>)]
pub enum GameMode {
    #[variant(0)]
    Survival,
    #[variant(1)]
    Creative,
    #[variant(2)]
    Adventure,
    #[variant(3)]
    Spectator,
}

#[derive(Debug, thiserror::Error)]
pub enum NameDecodeError {
    #[error("Failed to read string length")]
    Length(#[from] varint::I32DecodeError),
    #[error(transparent)]
    TooLong(#[from] profile::NameTooLong),
    #[error("UTF-8 error")]
    Utf8(#[from] string::FromUtf8Error),
}

impl ProtocolType for profile::Name {
    type DecodeError = NameDecodeError;
    type EncodeError = error::Infallible;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        let super::VarInt::<i32>(len) = super::VarInt::decode(buffer)?;

        Ok(Self::try_from(String::from_utf8(
            buffer.copy_to_bytes(len as usize).to_vec(),
        )?)?)
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        let bytes = self.as_ref().as_bytes();

        super::VarInt(bytes.len() as i32).encode(buffer)?;
        buffer.put(bytes);

        Ok(())
    }
}

pub const PROPERTY_FIELD_SIZE: usize = 1 << 15;

#[derive(Debug, thiserror::Error)]
pub enum PropertyDecodeError {
    #[error("Failed to read name")]
    Name(len_string::DecodeError<{ PROPERTY_FIELD_SIZE }>),
    #[error("Falied to read value")]
    Value(len_string::DecodeError<{ PROPERTY_FIELD_SIZE }>),
    #[error("JSON error")]
    Serde(#[from] serde_json::Error),
    #[error("Falied to read signature")]
    Signature(bool_option::DecodeError<len_string::DecodeError<{ PROPERTY_FIELD_SIZE }>>),
    #[error("Unsupported property name {0}")]
    Unsupported(len_string::LenString<{ PROPERTY_FIELD_SIZE }>),
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum PropertyEncodeError {
    #[error("Failed to write value")]
    Value(error::Overrun<{ PROPERTY_FIELD_SIZE }>),
    #[error("Failed to write signature")]
    Signature(error::Overrun<{ PROPERTY_FIELD_SIZE }>),
}

impl ProtocolType for profile::Property {
    type DecodeError = PropertyDecodeError;
    type EncodeError = PropertyEncodeError;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        match self {
            profile::Property::Textures {
                value, signature, ..
            } => {
                LenString::<{ PROPERTY_FIELD_SIZE }>("textures".into()).encode(buffer)?;
                LenString::<{ PROPERTY_FIELD_SIZE }>::try_from(
                    base64::engine::general_purpose::STANDARD.encode(value),
                )
                .map_err(PropertyEncodeError::Value)?
                .encode(buffer)?;

                if let Some(signature) = signature {
                    buffer.put_u8(1);
                    LenString::<{ PROPERTY_FIELD_SIZE }>::try_from(
                        base64::engine::general_purpose::STANDARD.encode(signature),
                    )
                    .map_err(PropertyEncodeError::Signature)?
                    .encode(buffer)?;
                } else {
                    buffer.put_u8(0);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, ProtocolType)]
pub struct Profile {
    pub username: rjacraft_authlib::profile::Name,
    /// See [Mojang's API](https://wiki.vg/Mojang_API#UUID_to_Profile_and_Skin.2FCape) for the
    /// meaning of these
    pub properties: LenVec<profile::Property>,
}

/// About the fields: There's no way to express this nicely in Rust. The lengths of each present
/// vector have to stay the same for this to be decodable.
#[derive(Debug)]
pub struct Updates {
    pub players: Vec<Uuid>,
    pub profile: Option<Vec<Profile>>,
    // todo signature
    pub gamemode: Option<Vec<GameMode>>,
    pub listed: Option<Vec<Primitive<bool>>>,
    pub ping: Option<Vec<VarInt<i32>>>,
    pub nickname: Option<Vec<JsonText>>,
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum UpdatesEncodeError {
    #[error("Profile")]
    Profile(#[from] ProfileEncodeError),
    #[error("Nickname")]
    Nickname(#[source] json_string::EncodeError<{ text::JSON_TEXT_LEN }>),
}

impl ProtocolType for Updates {
    type DecodeError = error::Eof;
    type EncodeError = UpdatesEncodeError;

    fn decode(_buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError> {
        todo!()
    }

    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError> {
        #[derive(ProtocolType)]
        #[bitfield(u8)]
        struct UsedProperties {
            profile: bool,
            signature: bool,
            gamemode: bool,
            listed: bool,
            ping: bool,
            nickname: bool,
            __: bool,
            __: bool,
        }

        UsedProperties::new()
            .with_profile(self.profile.is_some())
            .with_gamemode(self.gamemode.is_some())
            .with_listed(self.listed.is_some())
            .with_ping(self.ping.is_some())
            .with_nickname(self.nickname.is_some())
            .encode(buffer)?;

        VarInt(self.players.len() as i32).encode(buffer)?;

        for uuid in &self.players {
            uuid.encode(buffer)?;

            for x in self.profile.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.gamemode.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.listed.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.ping.iter().flatten() {
                x.encode(buffer)?;
            }

            for x in self.nickname.iter().flatten() {
                x.encode(buffer).map_err(UpdatesEncodeError::Nickname)?;
            }
        }

        Ok(())
    }
}
