//! Common decoding/encoding errors

pub use std::convert::Infallible;
use std::fmt;

/// Packet ended before we finished parsing.
#[derive(Debug, thiserror::Error, from_never::FromNever)]
#[error("Packet ended before we finished parsing")]
pub struct Eof;

/// Something is too long to be encoded or decoded.
#[derive(Debug, thiserror::Error)]
#[error("Type is too long: {0} > {}", MAX_SIZE)]
pub struct Overrun<const MAX_SIZE: usize>(pub usize);

/// An error related to a fieldless enum.
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
