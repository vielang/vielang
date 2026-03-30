use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache connection error: {0}")]
    Connection(String),

    #[error("Redis error: {0}")]
    Redis(String),
}
