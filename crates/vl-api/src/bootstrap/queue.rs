use std::sync::Arc;
use anyhow::Context;
use tracing::info;
use vl_config::VieLangConfig;
use vl_dao::DbPool;
use vl_queue::TbProducer;

use super::tasks::BackgroundTasks;

/// Queue state returned by [`init_queue`].
pub struct QueueState {
    pub queue_producer: Arc<dyn TbProducer>,
    pub queue_message_dao: Arc<vl_dao::QueueMessageDao>,
}

/// Initialize queue producer, consumers, DLQ handler, and cleanup job.
pub async fn init_queue(
    config: &VieLangConfig,
    pool: &DbPool,
    rule_engine_sender: tokio::sync::mpsc::Sender<vl_core::entities::TbMsg>,
    tasks: &mut BackgroundTasks,
) -> anyhow::Result<QueueState> {
    let queue_message_dao = Arc::new(vl_dao::QueueMessageDao::new(pool.clone()));

    let queue_producer: Arc<dyn TbProducer> =
        if config.queue.queue_type == vl_config::QueueType::Persistent {
            vl_queue::create_persistent_producer(queue_message_dao.clone())
        } else {
            vl_queue::create_producer(&config.queue)
                .context("Failed to initialize queue producer")?
        };
    info!("Queue backend: {:?}", config.queue.queue_type);

    // Queue → Rule Engine bridge (P6)
    {
        let persistent_cfg = config.queue.persistent.clone().unwrap_or_default();
        let n_consumers    = config.queue.consumer_threads.max(1);
        let timeout_secs   = config.queue.processing_timeout_secs;
        let dlq_topic      = config.queue.dlq_topic.clone();

        let dlq_producer: Arc<dyn TbProducer> =
            if config.queue.queue_type == vl_config::QueueType::Persistent {
                vl_queue::create_persistent_producer(queue_message_dao.clone())
            } else {
                vl_queue::create_producer(&config.queue)
                    .context("Failed to create DLQ producer")?
            };

        for i in 0..n_consumers {
            let label = format!("re-consumer-{i}");
            let consumer: Box<dyn vl_queue::TbConsumer> =
                if config.queue.queue_type == vl_config::QueueType::Persistent {
                    vl_queue::create_persistent_consumer(
                        queue_message_dao.clone(),
                        &[vl_queue::topics::VL_RULE_ENGINE],
                        &label,
                        persistent_cfg.batch_size,
                    )
                } else {
                    vl_queue::create_consumer(
                        &config.queue,
                        &[vl_queue::topics::VL_RULE_ENGINE],
                        &label,
                    )
                    .context("Failed to create rule engine queue consumer")?
                };

            let re_tx    = rule_engine_sender.clone();
            let dlq_prod = dlq_producer.clone();
            let dlq_t    = dlq_topic.clone();
            tasks.spawn(
                label.clone(),
                crate::services::queue_consumer::RuleEngineQueueConsumer::new(
                    consumer, re_tx, dlq_prod, dlq_t, timeout_secs, label,
                )
                .run(),
            );
        }
        info!("Queue → Rule Engine bridge: {n_consumers} consumer(s) started (timeout={timeout_secs}s)");

        // DLQ handler
        {
            let dlq_consumer: Box<dyn vl_queue::TbConsumer> =
                if config.queue.queue_type == vl_config::QueueType::Persistent {
                    vl_queue::create_persistent_consumer(
                        queue_message_dao.clone(),
                        &[&dlq_topic],
                        "dlq-handler",
                        persistent_cfg.batch_size,
                    )
                } else {
                    vl_queue::create_consumer(&config.queue, &[&dlq_topic], "dlq-handler")
                        .context("Failed to create DLQ consumer")?
                };
            let dlq_dao = Arc::new(vl_dao::DlqDao::new(pool.clone()));
            tasks.spawn(
                "dlq-handler",
                crate::services::dlq_handler::DlqHandler::new(dlq_consumer, dlq_dao).run(),
            );
            info!("DLQ handler started (topic={dlq_topic})");
        }

        // Persistent queue cleanup job
        if config.queue.queue_type == vl_config::QueueType::Persistent {
            let cleanup_dao = queue_message_dao.clone();
            let retention_h = config.queue.message_retention_hours;
            let cleanup_ivl = config.queue.persistent.clone()
                .unwrap_or_default()
                .cleanup_interval_h;
            tasks.spawn("queue-cleanup", async move {
                let mut interval = tokio::time::interval(
                    std::time::Duration::from_secs(cleanup_ivl * 3600),
                );
                interval.tick().await;
                loop {
                    interval.tick().await;
                    match cleanup_dao.cleanup_old_acked(retention_h).await {
                        Ok(n) if n > 0 => tracing::debug!(
                            "Queue cleanup: removed {n} acked messages older than {retention_h}h"
                        ),
                        Err(e) => tracing::warn!("Queue cleanup error: {e}"),
                        _ => {}
                    }
                }
            });
            info!("Persistent queue cleanup job started (retention={retention_h}h, interval={cleanup_ivl}h)");
        }
    }

    Ok(QueueState { queue_producer, queue_message_dao })
}
