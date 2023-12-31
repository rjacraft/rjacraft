//! Optionally encrypted asynchronous I/O streams

use std::{mem, pin, task};

use cipher::*;
use tokio::io;

#[pin_project::pin_project]
pub struct DecryptRead<C, S> {
    pub decryptor: Option<C>,
    #[pin]
    pub source: S,
}

impl<C, S> io::AsyncRead for DecryptRead<C, S>
where
    C: BlockDecryptMut + BlockSizeUser<BlockSize = typenum::U1>,
    S: io::AsyncRead,
{
    fn poll_read(
        self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> task::Poll<io::Result<()>> {
        let pinned = self.project();

        match pinned.source.poll_read(cx, buf) {
            task::Poll::Ready(Ok(())) => {
                if let Some(decryptor) = pinned.decryptor {
                    // SAFETY: In this version of `cipher`, the internal representation of blocks --
                    // `[GenericArray<u8, U1>]` -- is equivalent to that of `[u8]`. I haven't found a
                    // better way to do this transformation.
                    let blocks = unsafe { mem::transmute(buf.filled_mut()) };
                    decryptor.decrypt_blocks_mut(blocks);
                }

                task::Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

#[pin_project::pin_project]
pub struct EncryptWrite<C, W> {
    pub encryptor: Option<C>,
    #[pin]
    pub sink: W,
}

impl<C, S> io::AsyncWrite for EncryptWrite<C, S>
where
    C: BlockEncryptMut + BlockSizeUser<BlockSize = typenum::U1>,
    S: io::AsyncWrite,
{
    fn poll_write(
        self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> task::Poll<Result<usize, io::Error>> {
        let pinned = self.project();

        if let Some(encryptor) = pinned.encryptor {
            // I'm hoping that these don't get polled more than once
            let mut buf_enc = Vec::from(buf);
            // SAFETY: In this version of `cipher`, the internal representation of blocks --
            // `[GenericArray<u8, U1>]` -- is equivalent to that of `[u8]`. I haven't found a
            // better way to do this transformation.
            let blocks = unsafe { mem::transmute(&mut buf_enc[..]) };
            encryptor.encrypt_blocks_mut(blocks);

            pinned.sink.poll_write(cx, &buf_enc)
        } else {
            pinned.sink.poll_write(cx, buf)
        }
    }

    fn poll_flush(
        self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), io::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_shutdown(
        self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), io::Error>> {
        self.project().sink.poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> task::Poll<Result<usize, io::Error>> {
        self.project().sink.poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.sink.is_write_vectored()
    }
}
