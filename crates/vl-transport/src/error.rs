use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("MQTT protocol error: {0}")]
    Protocol(String),

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Database error: {0}")]
    Dao(#[from] vl_dao::DaoError),

    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
}
