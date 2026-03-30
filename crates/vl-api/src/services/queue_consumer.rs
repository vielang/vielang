use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, warn, debug};
use vl_core::entities::TbMsg;
use vl_queue::{TbConsumer, TbProducer, QueueMsg, QueueError};

/// Consumer loop kết nối Queue → Rule Engine.
///
/// Mỗi message được xử lý với timeout `processing_timeout_secs`.
/// Nếu fail hoặc timeout → message được route sang DLQ thay vì lost.
pub struct RuleEngineQueueConsumer {
    pub consumer:                Box<dyn TbConsumer>,
    pub re_sender:               mpsc::Sender<TbMsg>,
    pub dlq_producer:            Arc<dyn TbProducer>,
    pub dlq_topic:               String,
    pub processing_timeout_secs: u64,
    /// Label cho log (ví dụ: "re-consumer-0")
    pub label:                   String,
}

impl RuleEngineQueueConsumer {
    pub fn new(
        consumer:                Box<dyn TbConsumer>,
        re_sender:               mpsc::Sender<TbMsg>,
        dlq_producer:            Arc<dyn TbProducer>,
        dlq_topic:               String,
        processing_timeout_secs: u64,
        label:                   String,
    ) -> Self {
        Self { consumer, re_sender, dlq_producer, dlq_topic, processing_timeout_secs, label }
    }

    /// Chạy vòng poll/process/commit vô hạn.
    /// Chỉ dừng khi queue bị đóng (QueueError::Closed).
    pub async fn run(mut self) {
        let timeout      = Duration::from_secs(self.processing_timeout_secs);
        let label        = self.label.clone();
        let dlq_topic    = self.dlq_topic.clone();
        let dlq_producer = self.dlq_producer.clone();
        let re_sender    = self.re_sender.clone();

        loop {
            let msgs = match self.consumer.poll().await {
                Ok(m) => m,
                Err(QueueError::Closed) => {
                    debug!(consumer = %label, "Queue closed, stopping consumer");
                    break;
                }
                Err(e) => {
                    error!(consumer = %label, err = %e, "Queue poll error, retrying in 1s");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            for msg in &msgs {
                let result = process_with_timeout(msg, timeout, re_sender.clone()).await;
                if let Err(err_str) = result {
                    send_to_dlq(msg, &err_str, &dlq_topic, dlq_producer.clone(), &label).await;
                }
            }

            if let Err(e) = self.consumer.commit().await {
                error!(consumer = %label, err = %e, "Queue commit failed");
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// Deserialize + forward qua Rule Engine sender với timeout.
async fn process_with_timeout(
    msg:       &QueueMsg,
    timeout:   Duration,
    re_sender: mpsc::Sender<TbMsg>,
) -> Result<(), String> {
    let tb_msg = msg.to_tb_msg().map_err(|e| format!("deserialize error: {e}"))?;
    match tokio::time::timeout(timeout, re_sender.send(tb_msg)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("RE send error: {e}")),
        Err(_)     => Err(format!("RE processing timeout ({}s)", timeout.as_secs())),
    }
}

/// Route message sang DLQ; log warning nếu DLQ send fail.
async fn send_to_dlq(
    msg:         &QueueMsg,
    error:       &str,
    dlq_topic:   &str,
    dlq_producer: Arc<dyn TbProducer>,
    label:       &str,
) {
    warn!(
        consumer = %label,
        topic    = %msg.topic,
        err      = %error,
        "Message failed processing → DLQ"
    );
    let dlq_msg = QueueMsg::raw(dlq_topic.to_string(), msg.key.clone(), msg.value.clone());
    if let Err(e) = dlq_producer.send(&dlq_msg).await {
        error!(consumer = %label, err = %e, "Failed to send message to DLQ — message may be lost");
    }
}
