use bytes::{BufMut, BytesMut};
use rjacraft_protocol::{Encoder, VarInt};

#[derive(Clone, Debug, Default)]
pub struct PacketEncoder {
    pub buf: BytesMut,
}

impl PacketEncoder {
    pub fn append_frame(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let data_len = VarInt(bytes.len().try_into().unwrap());
        let start_len = self.buf.len();
        let data_len_size = data_len.written_size();

        self.buf.put_bytes(0, data_len_size);
        data_len.encode_write(&mut self.buf[start_len..])?;
        self.buf.extend_from_slice(bytes);

        Ok(())
    }

    pub fn append_packet<P>(&mut self, packet: P) -> anyhow::Result<()>
    where
        P: Encoder,
    {
        let mut buf = vec![];
        packet.encode(&mut buf)?;
        self.append_frame(&buf)
    }

    pub fn take(&mut self) -> BytesMut {
        self.buf.split()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_packet_encoder() {
        let mut encoder = PacketEncoder::default();
        encoder.append_frame(&b"hello"[..]).unwrap();
        encoder.append_frame(&b"world"[..]).unwrap();
        assert_eq!(encoder.buf, BytesMut::from(&b"\x05hello\x05world"[..]));
    }
}
