use std::sync::Arc;
use std::time::Duration;

use tracing::{error, warn, debug};
use vl_dao::DlqDao;
use vl_queue::{TbConsumer, QueueError};

/// Consumes messages từ DLQ topic và lưu vào bảng `dlq_messages`
/// để admin có thể inspect và replay về sau.
pub struct DlqHandler {
    consumer: Box<dyn TbConsumer>,
    dlq_dao:  Arc<DlqDao>,
}

impl DlqHandler {
    pub fn new(consumer: Box<dyn TbConsumer>, dlq_dao: Arc<DlqDao>) -> Self {
        Self { consumer, dlq_dao }
    }

    /// Vòng lặp vô hạn — poll DLQ topic và persist mỗi message.
    pub async fn run(mut self) {
        loop {
            let msgs = match self.consumer.poll().await {
                Ok(m) => m,
                Err(QueueError::Closed) => {
                    debug!("DLQ queue closed, stopping handler");
                    break;
                }
                Err(e) => {
                    error!(err = %e, "DLQ poll error, retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            for msg in &msgs {
                let error_msg = msg.headers.get("dlq.error").map(|s| s.as_str());
                match self.dlq_dao.store(&msg.topic, &msg.value, error_msg).await {
                    Ok(id) => {
                        warn!(
                            id    = %id,
                            topic = %msg.topic,
                            "Message stored in DLQ — requires manual inspection or replay"
                        );
                    }
                    Err(e) => {
                        error!(
                            err   = %e,
                            topic = %msg.topic,
                            "Failed to persist DLQ message to DB"
                        );
                    }
                }
            }

            if let Err(e) = self.consumer.commit().await {
                error!(err = %e, "DLQ consumer commit failed");
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
