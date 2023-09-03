//! Common decoding/encoding errors

pub use std::convert::Infallible;
use std::fmt;

/// Packet ended before we finished parsing
#[derive(Debug, thiserror::Error, from_never::FromNever)]
#[error("Packet ended before we finished parsing")]
pub struct Eof;

/// An enum-related error (an actual enum, not a sum type)
#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum EnumError<D: crate::ProtocolType>
where
    D: fmt::Debug,
{
    #[error("Failed to read enum discriminator")]
    ReadingDiscriminator(#[source] D::DecodeError),
    #[error("Received invalid enum {0:?}")]
    OutOfRange(D),
}