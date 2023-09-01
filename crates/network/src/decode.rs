use anyhow::bail;
use bytes::{Buf, BytesMut};
use rjacraft_protocol::{var_int::VarIntDecodeError, VarInt};

#[derive(Clone, Debug, Default)]
pub struct PacketDecoder {
    pub buf: BytesMut,
}

impl PacketDecoder {
    pub fn try_next_frame(&mut self) -> anyhow::Result<Option<BytesMut>> {
        let mut r = &self.buf[..];

        let len = match VarInt::decode_partial(&mut r) {
            Ok(len) => len,
            Err(VarIntDecodeError::Incomplete) => return Ok(None),
            Err(VarIntDecodeError::TooLarge) => bail!("malformed packet length VarInt"),
        };

        if r.len() < len as usize {
            return Ok(None);
        }

        let len_size = VarInt(len).written_size();

        self.buf.advance(len_size);
        let data = self.buf.split_to(len as usize);

        Ok(Some(data))
    }

    pub fn queue_bytes(&mut self, bytes: BytesMut) {
        self.buf.unsplit(bytes);
    }
}
