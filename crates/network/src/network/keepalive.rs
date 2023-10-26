use tokio::time;
use tracing::*;

const COOLDOWN: time::Duration = time::Duration::from_secs(1);
const TIMEOUT: time::Duration = time::Duration::from_secs(30);

pub enum Message {
    Packet(i64),
    Mismatch,
    Timeout,
}

#[instrument(skip_all)]
pub async fn keepalive_loop(tx: flume::Sender<Message>, rx: flume::Receiver<i64>) {
    let start_time = time::Instant::now();

    loop {
        let id = start_time.elapsed().as_millis() as i64;

        time::sleep(COOLDOWN).await;
        let _ = tx.send(Message::Packet(id));

        if let Ok(id_in) = time::timeout(TIMEOUT, rx.recv_async())
            .await
            .map(|x| x.unwrap())
        {
            if id_in != id {
                info!("id mismatch. expected {id} got {id_in}");
                let _ = tx.send(Message::Mismatch);
            }
        } else {
            info!("timeout");
            let _ = tx.send(Message::Timeout);
        }
    }
}
