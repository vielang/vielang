/// What to do when actor initialization fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitFailureStrategy {
    /// Retry init immediately.
    RetryImmediately,
    /// Retry init after a delay.
    RetryWithDelay { delay_ms: u64 },
    /// Stop the actor permanently.
    Stop,
}

/// What to do when message processing fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessFailureStrategy {
    /// Resume processing next message.
    Resume,
    /// Stop the actor.
    Stop,
}
