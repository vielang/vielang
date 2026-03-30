/// Errors that can occur in the actor system.
#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("actor not found: {0}")]
    NotFound(String),

    #[error("actor mailbox full")]
    MailboxFull,

    #[error("actor stopped")]
    Stopped,

    #[error("dispatcher not found: {0}")]
    DispatcherNotFound(String),

    #[error("init failed after {attempts} attempts: {cause}")]
    InitFailed { attempts: u32, cause: String },

    #[error("{0}")]
    Internal(String),
}
