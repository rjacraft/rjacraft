//! The crucial parts of Minecraft's protocol.
//!
//! # Credits
//! This crate is a substantial reproduction of [wiki.vg](https://wiki.vg).

use tokio::io;

/// A packet or any part of a packet.
pub trait ProtocolType: Sized {
    type DecodeError: std::error::Error + 'static;
    type EncodeError: std::error::Error + 'static;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError>;
    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError>;

    fn to_encoded(&self) -> Result<types::Encoded<Self>, Self::EncodeError> {
        types::Encoded::new(self)
    }

    fn to_encoded_expect(&self) -> types::Encoded<Self> {
        types::Encoded::new(self).expect("error encoding packet")
    }

    fn to_bytes(&self) -> Result<bytes::Bytes, Self::EncodeError> {
        Ok(self.to_encoded()?.data())
    }

    fn to_bytes_expect(&self) -> bytes::Bytes {
        self.to_encoded_expect().data()
    }
}

/// Currently used by [`crate::frame`] to read packet length prefixes.
#[async_trait::async_trait]
pub trait ProtocolTypeIo: Sized {
    async fn decode_io(read: &mut (impl io::AsyncRead + Unpin + Send)) -> io::Result<Self>;
    async fn encode_io(&self, write: &mut (impl io::AsyncWrite + Unpin + Send)) -> io::Result<()>;
    fn written_size(&self) -> usize;
}

pub mod chunk;
pub mod entity_properties;
pub mod error;
pub mod frame;
pub mod packets;
pub mod types;
mod version;

pub use self::version::ProtocolVersion;

/// The protocol version that this library supports.
pub const SUPPORTED_PROTOCOL: ProtocolVersion = ProtocolVersion::from_snapshot(147);
