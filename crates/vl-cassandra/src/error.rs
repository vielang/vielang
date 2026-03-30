use thiserror::Error;

use vl_dao::DaoError;

#[derive(Debug, Error)]
pub enum CassandraError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Row deserialization error: {0}")]
    Deserialization(String),

    #[error("Schema init error: {0}")]
    Schema(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<CassandraError> for DaoError {
    fn from(e: CassandraError) -> Self {
        DaoError::Cassandra(e.to_string())
    }
}
