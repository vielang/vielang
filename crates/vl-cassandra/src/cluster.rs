use std::sync::Arc;

use scylla::client::execution_profile::ExecutionProfile;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::statement::Consistency;
use tracing::info;

use vl_config::CassandraConfig;

use crate::error::CassandraError;
use crate::schema::init_schema;

/// Quản lý kết nối Cassandra / ScyllaDB session.
/// Session là thread-safe và được share qua Arc.
pub struct CassandraCluster {
    session: Arc<Session>,
    keyspace: String,
}

impl CassandraCluster {
    /// Kết nối tới Cassandra và khởi tạo schema.
    pub async fn connect(config: &CassandraConfig) -> Result<Self, CassandraError> {
        info!("Connecting to Cassandra: {}", config.url);

        let mut builder = SessionBuilder::new()
            .known_node(&config.url)
            .default_execution_profile_handle(
                ExecutionProfile::builder()
                    .consistency(Consistency::LocalQuorum)
                    .build()
                    .into_handle(),
            );

        // Authentication nếu có
        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            builder = builder.user(user, pass);
        }

        let session = builder
            .build()
            .await
            .map_err(|e| CassandraError::Connection(e.to_string()))?;

        let session = Arc::new(session);

        // Khởi tạo schema
        init_schema(&session, &config.keyspace).await?;

        info!("Cassandra connected to keyspace '{}'", config.keyspace);

        Ok(Self {
            session,
            keyspace: config.keyspace.clone(),
        })
    }

    pub fn session(&self) -> Arc<Session> {
        self.session.clone()
    }

    pub fn keyspace(&self) -> &str {
        &self.keyspace
    }
}
