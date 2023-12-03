use tokio::time;
use tracing::*;

const COOLDOWN: time::Duration = time::Duration::from_secs(1);
const TIMEOUT: time::Duration = time::Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ID mismatch. Expected: {expected}, got: {got}")]
    Mismatch { expected: i64, got: i64 },
    #[error("Time out exceeded")]
    Timeout,
}

#[instrument(skip_all)]
pub async fn keepalive_loop(tx: flume::Sender<i64>, rx: flume::Receiver<i64>) -> Error {
    let start_time = time::Instant::now();

    loop {
        let id = start_time.elapsed().as_millis() as i64;

        time::sleep(COOLDOWN).await;
        tx.send(id).expect("channel should work");

        if let Ok(id_in) = time::timeout(TIMEOUT, rx.recv_async())
            .await
            .map(|x| x.unwrap())
        {
            if id_in != id {
                return Error::Mismatch {
                    expected: id,
                    got: id_in,
                };
            }
        } else {
            return Error::Timeout;
        }
    }
}
