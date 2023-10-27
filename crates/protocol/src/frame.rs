//! Functions for asychronously decoding packet frames

use core::num;

use pretty_hex::*;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tracing::*;

use crate::*;

const HEX_CONFIG: HexConfig = HexConfig {
    title: true,
    ascii: true,
    width: 16,
    group: 0,
    chunk: 4,
    max_bytes: usize::MAX,
};

// TODO compress & encrypt

#[derive(Debug, thiserror::Error)]
pub enum ReadFrameError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Failed to decode length")]
    DecodingLength(#[from] types::varint::DecodeError),
}

/// The output future is **not cancellable**
pub async fn read_frame(
    source: &mut (impl io::AsyncRead + Unpin + Send),
) -> Result<bytes::Bytes, ReadFrameError> {
    let types::VarInt(length) = types::VarInt::decode_raw(source).await??;

    let mut buffer = vec![0; length as usize];

    source.read_exact(&mut buffer).await?;

    debug!("READ {:?}", buffer.hex_conf(HEX_CONFIG));

    Ok(buffer.into())
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum WriteFrameError {
    #[error("Packet was too large")]
    TooLarge(#[source] num::TryFromIntError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// The output future is **not cancellable**
pub async fn write_frame(
    dest: &mut (impl io::AsyncWrite + Unpin + Send),
    buffer: &[u8],
) -> Result<(), WriteFrameError> {
    debug!("WRITE {:?}", buffer.hex_conf(HEX_CONFIG));

    let length: i32 = buffer.len().try_into().map_err(WriteFrameError::TooLarge)?;

    types::VarInt(length).encode_raw(dest).await??;
    dest.write_all(&buffer).await?;

    Ok(())
}
