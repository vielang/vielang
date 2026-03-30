use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RuleEngineError {
    #[error("Script error: {0}")]
    Script(String),

    #[error("DAO error: {0}")]
    Dao(#[from] vl_dao::DaoError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid config: {0}")]
    Config(String),

    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Processing error: {0}")]
    Processing(String),
}
