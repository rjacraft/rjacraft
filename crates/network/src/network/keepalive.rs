use std::{future::Future, pin, task};

use tokio::time;

const COOLDOWN: time::Duration = time::Duration::from_secs(1);
const TIMEOUT: time::Duration = time::Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
#[error("No keep alive response received in time")]
pub struct TimedOut;

/// This is more of an async generator rather than a future, but since those haven't come around
/// yet, this is how we'll do things.
///
/// 1. We start with a cooldown
/// 2. Poll: the cooldown is pending -> pending
/// 3. Poll: the cooldown is ready -> ready with the id
/// 4. Poll: the timeout is pending -> back to pending
/// 5. The right ID came in. The future must be dropped and started again. Alternatively, the
/// timeout resolves to ready -> ready with None
#[pin_project::pin_project(project = Projected)]
pub enum KeepAlive {
    Cooldown(time::Instant, #[pin] time::Sleep),
    RequestSent(#[pin] time::Sleep),
    TimedOut,
}

impl KeepAlive {
    pub fn start(start_time: time::Instant) -> Self {
        Self::Cooldown(start_time, time::sleep(COOLDOWN))
    }
}

impl Future for KeepAlive {
    type Output = Option<i64>;

    fn poll(mut self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let (new, result) = match self.as_mut().project() {
            Projected::Cooldown(start, sleep) => {
                if let task::Poll::Ready(_) = sleep.poll(cx) {
                    (
                        Some(KeepAlive::RequestSent(time::sleep(TIMEOUT))),
                        task::Poll::Ready(Some(start.elapsed().as_millis() as i64)),
                    )
                } else {
                    (None, task::Poll::Pending)
                }
            }
            Projected::RequestSent(sleep) => {
                if let task::Poll::Ready(_) = sleep.poll(cx) {
                    (Some(KeepAlive::TimedOut), task::Poll::Ready(None))
                } else {
                    (None, task::Poll::Pending)
                }
            }
            Projected::TimedOut => (None, task::Poll::Ready(None)),
        };

        if let Some(new) = new {
            self.set(new);
        }

        result
    }
}
