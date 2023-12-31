//! Functions for asynchronously decoding and encoding packet frames

use core::num;
use std::io::{Read, Write};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use pretty_hex::*;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tracing::*;
use types::VarInt;

use crate::*;

pub mod encrypted_stream;

const HEX_CONFIG: HexConfig = HexConfig {
    title: false,
    ascii: true,
    width: 16,
    group: 0,
    chunk: 4,
    max_bytes: usize::MAX,
};

#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Failed to decode length")]
    DecodingLength(#[from] types::varint::I32DecodeError),
}

pub struct Reader<C, S> {
    pub source: encrypted_stream::DecryptRead<C, S>,
    pub compress: bool,
}

impl<C, S> Reader<C, S>
where
    encrypted_stream::DecryptRead<C, S>: io::AsyncRead + Unpin + Send,
{
    /// Reads a raw frame from a source. The output future is **not cancellable**.
    pub async fn read_frame(&mut self) -> Result<bytes::Bytes, ReaderError> {
        let VarInt::<i32>(outer_length) = VarInt::decode_io(&mut self.source).await?;
        let mut inner_buffer;
        let compressed;
        let compressed_length;

        if self.compress {
            let VarInt::<i32>(inner_length) = VarInt::decode_io(&mut self.source).await?;
            let next_length = outer_length as usize - VarInt::<i32>(inner_length).written_size();

            if inner_length == 0 {
                compressed = false;
                compressed_length = None;

                inner_buffer = vec![0; next_length];
                self.source.read_exact(&mut inner_buffer).await?;
            } else {
                compressed = true;
                compressed_length = Some(inner_length);

                let mut compressed_buffer = vec![0; next_length];
                self.source.read_exact(&mut compressed_buffer).await?;

                let mut decoder = ZlibDecoder::new(&compressed_buffer[..]);
                inner_buffer = vec![0; inner_length as usize];
                decoder.read_exact(&mut inner_buffer)?;
            }
        } else {
            compressed = false;
            compressed_length = None;

            inner_buffer = vec![0; outer_length as usize];
            self.source.read_exact(&mut inner_buffer).await?;
        }

        trace!(
            length = inner_buffer.len(),
            dir = "recv",
            compress = compressed.then(|| "zlib"),
            compressed_length,
            encrypt = self.source.decryptor.is_some().then(|| "aes/cfb8"),
            "\n{:?}",
            inner_buffer.hex_conf(HEX_CONFIG)
        );

        Ok(inner_buffer.into())
    }
}

#[derive(Debug, thiserror::Error, from_never::FromNever)]
pub enum WriterError {
    #[error("Packet was too large")]
    TooLarge(#[source] num::TryFromIntError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Writer<C, S> {
    pub sink: encrypted_stream::EncryptWrite<C, S>,
    pub compress: Option<u32>,
}

impl<C, S> Writer<C, S>
where
    encrypted_stream::EncryptWrite<C, S>: io::AsyncWrite + Unpin + Send,
{
    /// Writes a raw frame to a writer. The output future is **not cancellable**.
    pub async fn write_frame(&mut self, inner_buffer: &[u8]) -> Result<(), WriterError> {
        let compressed;
        let compressed_length;

        let inner_length: i32 = inner_buffer
            .len()
            .try_into()
            .map_err(WriterError::TooLarge)?;

        if let Some(threshold) = self.compress {
            if inner_buffer.len() < threshold as usize {
                compressed = false;
                compressed_length = None;

                VarInt(inner_length + 1).encode_io(&mut self.sink).await?;
                VarInt(0i32).encode_io(&mut self.sink).await?;
                self.sink.write_all(inner_buffer).await?;
            } else {
                compressed = true;

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(inner_buffer)?;
                let compressed_buffer = encoder.finish()?;

                compressed_length = Some(compressed_buffer.len());

                VarInt(VarInt(inner_length).written_size() as i32 + compressed_buffer.len() as i32)
                    .encode_io(&mut self.sink)
                    .await?;

                VarInt(inner_length).encode_io(&mut self.sink).await?;
                self.sink.write_all(&compressed_buffer).await?;
            }
        } else {
            compressed = false;
            compressed_length = None;

            VarInt(inner_length).encode_io(&mut self.sink).await?;
            self.sink.write_all(inner_buffer).await?;
        }

        trace!(
            length = inner_buffer.len(),
            dir = "send",
            compress = compressed.then(|| "zlib"),
            compressed_length,
            encrypt = self.sink.encryptor.is_some().then(|| "aes/cfb8"),
            "\n{:?}",
            inner_buffer.hex_conf(HEX_CONFIG)
        );

        Ok(())
    }
}
