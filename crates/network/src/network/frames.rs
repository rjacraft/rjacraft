use rjacraft_protocol::frame::*;
use tokio::io;

/// Reading varints isn't cancellable so that's why this needs to be its own task
pub async fn frame_read_loop(
    mut source: impl io::AsyncRead + Unpin + Send,
    frames: flume::Sender<bytes::Bytes>,
) -> Result<(), ReadPacketError> {
    loop {
        let packet = read_frame(&mut source).await?;

        let _ = frames.send(packet);
    }
}
