use bitfield_struct::bitfield;
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

#[derive(Debug, ProtocolType)]
pub struct ProfileProperty {
    pub name: LenString<{ 1 << 15 }>,
    pub value: LenString<{ 1 << 15 }>,
    pub signature: BoolOption<LenString<{ 1 << 15 }>>,
}

#[derive(Debug, ProtocolType)]
pub struct Profile {
    pub username: LenString<16>,
    /// See [Mojang's API](https://wiki.vg/Mojang_API#UUID_to_Profile_and_Skin.2FCape) for the
    /// meaning of these
    pub properties: LenVec<ProfileProperty>,
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
