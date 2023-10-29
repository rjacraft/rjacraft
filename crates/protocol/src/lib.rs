//! The crucial parts of Minecraft's protocol.

use std::future::Future;

use tokio::io;

/// A packet or any part of a packet.
pub trait ProtocolType: Sized {
    type DecodeError: std::error::Error + 'static;
    type EncodeError: std::error::Error + 'static;

    fn decode(buffer: &mut impl bytes::Buf) -> Result<Self, Self::DecodeError>;
    fn encode(&self, buffer: &mut impl bytes::BufMut) -> Result<(), Self::EncodeError>;

    fn encode_owned(&self) -> Result<bytes::Bytes, Self::EncodeError> {
        let mut bytes = bytes::BytesMut::new();
        self.encode(&mut bytes)?;
        Ok(bytes.freeze())
    }
}

/// Currently used by the networking code to read packet length prefixes.
pub trait ProtocolTypeRaw: ProtocolType {
    fn decode_raw(
        read: &mut (impl io::AsyncRead + Unpin + Send),
    ) -> impl Future<Output = io::Result<Result<Self, Self::DecodeError>>> + Send;

    fn encode_raw(
        &self,
        write: &mut (impl io::AsyncWrite + Unpin + Send),
    ) -> impl Future<Output = io::Result<Result<(), Self::EncodeError>>> + Send;
}

pub mod error;
pub mod frame;
pub mod packets;
pub mod types;
mod version;

pub use self::version::ProtocolVersion;

/// The protocol version that this library supports.
pub const SUPPORTED_PROTOCOL: ProtocolVersion = ProtocolVersion::from_snapshot(147);
