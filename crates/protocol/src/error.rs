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

impl<const MAX_SIZE: usize> Overrun<MAX_SIZE> {
    pub fn as_serde<E: serde::de::Error>(self) -> E {
        E::invalid_length(self.0, &format!("a length less than {MAX_SIZE}").as_str())
    }
}

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
