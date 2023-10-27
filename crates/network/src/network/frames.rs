use rjacraft_protocol::frame::*;
use tokio::io;

/// Reading varints isn't cancellable so that's why this needs to be its own task
pub async fn frame_read_loop(
    mut source: impl io::AsyncRead + Unpin + Send,
    frames: flume::Sender<bytes::Bytes>,
) -> Result<(), ReadFrameError> {
    loop {
        let packet = read_frame(&mut source).await?;

        let _ = frames.send(packet);
    }
}

/// Here I just want to have a neat mspc
pub async fn frame_write_loop(
    mut source: impl io::AsyncWrite + Unpin + Send,
    frames: flume::Receiver<bytes::Bytes>,
) -> Result<(), WriteFrameError> {
    while let Ok(frame) = frames.recv_async().await {
        write_frame(&mut source, &frame).await?;
    }

    Ok(())
}
