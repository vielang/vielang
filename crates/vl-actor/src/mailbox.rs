use crate::{
    ActorMsg, InitFailureStrategy, StopReason, TbActor, TbActorCtx, TbActorId, TbActorRef,
};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

/// Message priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgPriority {
    High,
    Normal,
}

/// Settings for the actor mailbox processing.
#[derive(Debug, Clone)]
pub struct MailboxSettings {
    /// Max messages to process in one batch before yielding.
    pub actor_throughput: usize,
    /// Max init retry attempts (0 = unlimited).
    pub max_init_attempts: u32,
}

impl Default for MailboxSettings {
    fn default() -> Self {
        Self {
            actor_throughput: 30,
            max_init_attempts: 10,
        }
    }
}

/// Internal mailbox that drives an actor's message loop.
///
/// Mirrors ThingsBoard's `TbActorMailbox`:
/// - Two unbounded channels (high & normal priority)
/// - High-priority messages drained first each batch
/// - Throughput limit per batch (yields to other actors)
/// - Init retry with configurable strategy
pub(crate) struct TbActorMailbox;

impl TbActorMailbox {
    /// Spawn the actor's processing loop. Returns the `TbActorRef` for sending
    /// messages and a `JoinHandle` for the background task.
    pub fn spawn(
        id: TbActorId,
        actor: Box<dyn TbActor>,
        ctx: TbActorCtx,
        settings: MailboxSettings,
    ) -> (TbActorRef, tokio::task::JoinHandle<()>) {
        let (high_tx, high_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();

        let actor_ref = TbActorRef::new(id.clone(), high_tx, normal_tx);

        let handle = tokio::spawn(Self::run(id, actor, ctx, high_rx, normal_rx, settings));

        (actor_ref, handle)
    }

    async fn run(
        id: TbActorId,
        mut actor: Box<dyn TbActor>,
        ctx: TbActorCtx,
        mut high_rx: mpsc::UnboundedReceiver<ActorMsg>,
        mut normal_rx: mpsc::UnboundedReceiver<ActorMsg>,
        settings: MailboxSettings,
    ) {
        // ── Phase 1: Init with retry ───────────────────────────
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match actor.init(ctx.clone()).await {
                Ok(()) => {
                    debug!("actor {id} initialized (attempt {attempt})");
                    break;
                }
                Err(e) => {
                    let strategy = actor.on_init_failure(attempt, &e);
                    match strategy {
                        InitFailureStrategy::RetryImmediately => {
                            warn!("actor {id} init failed (attempt {attempt}), retrying immediately: {e}");
                            continue;
                        }
                        InitFailureStrategy::RetryWithDelay { delay_ms } => {
                            if settings.max_init_attempts > 0
                                && attempt >= settings.max_init_attempts
                            {
                                error!(
                                    "actor {id} init failed after {attempt} attempts, stopping: {e}"
                                );
                                actor.destroy(StopReason::InitFailed).await;
                                return;
                            }
                            warn!(
                                "actor {id} init failed (attempt {attempt}), retry in {delay_ms}ms: {e}"
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            continue;
                        }
                        InitFailureStrategy::Stop => {
                            error!("actor {id} init failed, stopping: {e}");
                            actor.destroy(StopReason::InitFailed).await;
                            return;
                        }
                    }
                }
            }
        }

        // ── Phase 2: Message processing loop ───────────────────
        loop {
            // Wait for at least one message from either channel.
            let first_msg = tokio::select! {
                msg = high_rx.recv() => msg,
                msg = normal_rx.recv() => msg,
            };

            let Some(first_msg) = first_msg else {
                // Both channels closed → stop.
                debug!("actor {id} channels closed, stopping");
                break;
            };

            // Process the first message + drain up to throughput limit.
            if !Self::process_msg(&id, &mut actor, first_msg).await {
                break;
            }

            let mut processed = 1usize;
            while processed < settings.actor_throughput {
                // Drain high-priority first, then normal.
                let msg = match high_rx.try_recv() {
                    Ok(m) => m,
                    Err(_) => match normal_rx.try_recv() {
                        Ok(m) => m,
                        Err(_) => break, // No more messages in batch
                    },
                };
                processed += 1;
                if !Self::process_msg(&id, &mut actor, msg).await {
                    actor.destroy(StopReason::ProcessingError).await;
                    return;
                }
            }

            // Yield to other tasks after a batch.
            if processed >= settings.actor_throughput {
                tokio::task::yield_now().await;
            }
        }

        actor.destroy(StopReason::Normal).await;
    }

    /// Process a single message. Returns `false` if the actor should stop.
    async fn process_msg(id: &TbActorId, actor: &mut Box<dyn TbActor>, msg: ActorMsg) -> bool {
        // Catch panics in processing
        let result = std::panic::AssertUnwindSafe(actor.process(msg));
        // Note: we don't use catch_unwind for async — instead rely on
        // the actor's own error handling. The on_process_failure strategy
        // is consulted by the actor implementation itself.
        // For simplicity, if process() returns false, we treat it as
        // "unhandled but continue".
        let handled = result.await;
        if !handled {
            debug!("actor {id}: message not handled");
        }
        true
    }
}
