use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaoError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Entity not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    Constraint(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cassandra error: {0}")]
    Cassandra(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl DaoError {
    /// Phân biệt unique constraint violation từ sqlx error
    pub fn from_sqlx(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db_err) = err {
            if db_err.code().as_deref() == Some("23505") {
                return Self::Constraint(db_err.message().to_string());
            }
        }
        Self::Database(err)
    }
}
