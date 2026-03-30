use std::sync::Arc;

use scylla::client::session::Session;
use tracing::info;

use crate::error::CassandraError;

/// CQL statements để khởi tạo schema — chạy một lần khi startup.
/// Khớp Java: schema-keyspace.cql, schema-ts.cql, schema-ts-latest.cql
pub async fn init_schema(session: &Arc<Session>, keyspace: &str) -> Result<(), CassandraError> {
    // ── Keyspace ────────────────────────────────────────────────────────────────
    let create_keyspace = format!(
        "CREATE KEYSPACE IF NOT EXISTS {keyspace} \
         WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}} \
         AND durable_writes = true"
    );
    session
        .query_unpaged(create_keyspace, ())
        .await
        .map_err(|e| CassandraError::Schema(format!("CREATE KEYSPACE: {e}")))?;

    info!("Cassandra keyspace '{keyspace}' ready");

    // ── ts_kv_cf — timeseries lịch sử ─────────────────────────────────────────
    // PRIMARY KEY: ((entity_type, entity_id, key, partition), ts)
    // Clustering ORDER BY ts DESC → đọc mới nhất trước
    // TWCS: tối ưu cho append-only time-series (khuyến nghị cho IoT)
    let create_ts = format!(
        "CREATE TABLE IF NOT EXISTS {keyspace}.ts_kv_cf (
            entity_type text,
            entity_id   uuid,
            key         text,
            partition   bigint,
            ts          bigint,
            bool_v      boolean,
            str_v       text,
            long_v      bigint,
            dbl_v       double,
            json_v      text,
            PRIMARY KEY ((entity_type, entity_id, key, partition), ts)
        ) WITH CLUSTERING ORDER BY (ts DESC)
          AND compaction = {{
            'class': 'TimeWindowCompactionStrategy',
            'compaction_window_unit': 'DAYS',
            'compaction_window_size': 1
          }}
          AND gc_grace_seconds = 86400"
    );
    session
        .query_unpaged(create_ts, ())
        .await
        .map_err(|e| CassandraError::Schema(format!("CREATE TABLE ts_kv_cf: {e}")))?;

    // ── ts_kv_partitions_cf — partition tracking ────────────────────────────────
    // Dùng để tối ưu query: biết partition nào tồn tại trước khi scan
    let create_partitions = format!(
        "CREATE TABLE IF NOT EXISTS {keyspace}.ts_kv_partitions_cf (
            entity_type text,
            entity_id   uuid,
            key         text,
            partition   bigint,
            PRIMARY KEY ((entity_type, entity_id, key), partition)
        ) WITH CLUSTERING ORDER BY (partition ASC)
          AND compaction = {{'class': 'LeveledCompactionStrategy'}}"
    );
    session
        .query_unpaged(create_partitions, ())
        .await
        .map_err(|e| CassandraError::Schema(format!("CREATE TABLE ts_kv_partitions_cf: {e}")))?;

    // ── ts_kv_latest_cf — latest values ─────────────────────────────────────────
    // PRIMARY KEY: ((entity_type, entity_id), key)
    // Không có partition column → đọc "state hiện tại" rất nhanh
    let create_latest = format!(
        "CREATE TABLE IF NOT EXISTS {keyspace}.ts_kv_latest_cf (
            entity_type text,
            entity_id   uuid,
            key         text,
            ts          bigint,
            bool_v      boolean,
            str_v       text,
            long_v      bigint,
            dbl_v       double,
            json_v      text,
            PRIMARY KEY ((entity_type, entity_id), key)
        ) WITH compaction = {{'class': 'LeveledCompactionStrategy'}}"
    );
    session
        .query_unpaged(create_latest, ())
        .await
        .map_err(|e| CassandraError::Schema(format!("CREATE TABLE ts_kv_latest_cf: {e}")))?;

    info!("Cassandra schema initialized (ts_kv_cf, ts_kv_partitions_cf, ts_kv_latest_cf)");
    Ok(())
}
