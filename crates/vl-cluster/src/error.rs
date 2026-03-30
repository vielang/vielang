use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("etcd HTTP error: {0}")]
    Etcd(String),

    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
