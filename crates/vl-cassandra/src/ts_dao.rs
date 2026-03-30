use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use scylla::client::session::Session;
use scylla::statement::prepared::PreparedStatement;
use tracing::{instrument, warn};
use uuid::Uuid;

use vl_core::entities::TsRecord;
use vl_dao::{DaoError, TimeseriesDao};

use crate::error::CassandraError;
use crate::partition::{partitions_in_range, to_partition_ts, PartitionGranularity};

/// Cassandra implementation của TimeseriesDao.
/// Dùng ts_kv_cf (history) và ts_kv_latest_cf (latest) tables.
pub struct CassandraTs {
    session:     Arc<Session>,
    #[allow(dead_code)]
    keyspace:    String,
    granularity: PartitionGranularity,
    ttl_seconds: i64,

    // Prepared statements — cache lại để tránh re-prepare mỗi request
    ps_insert_ts:         Arc<PreparedStatement>,
    ps_insert_ts_ttl:     Arc<PreparedStatement>,
    ps_insert_partition:  Arc<PreparedStatement>,
    ps_insert_latest:     Arc<PreparedStatement>,
    ps_select_latest_all: Arc<PreparedStatement>,
    ps_select_latest_key: Arc<PreparedStatement>,
    ps_select_ts:         Arc<PreparedStatement>,
    ps_select_keys:       Arc<PreparedStatement>,
    ps_delete_latest:     Arc<PreparedStatement>,
    ps_delete_ts:         Arc<PreparedStatement>,

    /// Cache tracking partition rows đã được insert.
    /// Key: (entity_type, entity_id, key, partition_ts)
    /// Tránh ghi duplicate vào ts_kv_partitions_cf
    partition_cache: DashMap<(String, Uuid, String, i64), ()>,
}

impl CassandraTs {
    pub async fn new(
        session: Arc<Session>,
        keyspace: &str,
        granularity: PartitionGranularity,
        ttl_seconds: i64,
        partition_cache_size: usize,
    ) -> Result<Self, CassandraError> {
        let ks = keyspace;

        // Prepare tất cả statements một lần khi startup
        let ps_insert_ts = session
            .prepare(format!(
                "INSERT INTO {ks}.ts_kv_cf \
                 (entity_type, entity_id, key, partition, ts, bool_v, str_v, long_v, dbl_v, json_v) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare insert_ts: {e}")))?;

        let ps_insert_ts_ttl = session
            .prepare(format!(
                "INSERT INTO {ks}.ts_kv_cf \
                 (entity_type, entity_id, key, partition, ts, bool_v, str_v, long_v, dbl_v, json_v) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 USING TTL ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare insert_ts_ttl: {e}")))?;

        let ps_insert_partition = session
            .prepare(format!(
                "INSERT INTO {ks}.ts_kv_partitions_cf \
                 (entity_type, entity_id, key, partition) \
                 VALUES (?, ?, ?, ?)"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare insert_partition: {e}")))?;

        let ps_insert_latest = session
            .prepare(format!(
                "INSERT INTO {ks}.ts_kv_latest_cf \
                 (entity_type, entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare insert_latest: {e}")))?;

        let ps_select_latest_all = session
            .prepare(format!(
                "SELECT key, ts, bool_v, str_v, long_v, dbl_v, json_v \
                 FROM {ks}.ts_kv_latest_cf \
                 WHERE entity_type = ? AND entity_id = ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare select_latest_all: {e}")))?;

        let ps_select_latest_key = session
            .prepare(format!(
                "SELECT key, ts, bool_v, str_v, long_v, dbl_v, json_v \
                 FROM {ks}.ts_kv_latest_cf \
                 WHERE entity_type = ? AND entity_id = ? AND key = ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare select_latest_key: {e}")))?;

        let ps_select_ts = session
            .prepare(format!(
                "SELECT ts, bool_v, str_v, long_v, dbl_v, json_v \
                 FROM {ks}.ts_kv_cf \
                 WHERE entity_type = ? AND entity_id = ? AND key = ? AND partition = ? \
                 AND ts >= ? AND ts <= ? \
                 ORDER BY ts DESC \
                 LIMIT ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare select_ts: {e}")))?;

        let ps_select_keys = session
            .prepare(format!(
                "SELECT key FROM {ks}.ts_kv_latest_cf \
                 WHERE entity_type = ? AND entity_id = ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare select_keys: {e}")))?;

        let ps_delete_latest = session
            .prepare(format!(
                "DELETE FROM {ks}.ts_kv_latest_cf \
                 WHERE entity_type = ? AND entity_id = ? AND key = ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare delete_latest: {e}")))?;

        let ps_delete_ts = session
            .prepare(format!(
                "DELETE FROM {ks}.ts_kv_cf \
                 WHERE entity_type = ? AND entity_id = ? AND key = ? AND partition = ? \
                 AND ts >= ? AND ts <= ?"
            ))
            .await
            .map_err(|e| CassandraError::Query(format!("prepare delete_ts: {e}")))?;

        Ok(Self {
            session,
            keyspace: keyspace.to_string(),
            granularity,
            ttl_seconds,
            ps_insert_ts: Arc::new(ps_insert_ts),
            ps_insert_ts_ttl: Arc::new(ps_insert_ts_ttl),
            ps_insert_partition: Arc::new(ps_insert_partition),
            ps_insert_latest: Arc::new(ps_insert_latest),
            ps_select_latest_all: Arc::new(ps_select_latest_all),
            ps_select_latest_key: Arc::new(ps_select_latest_key),
            ps_select_ts: Arc::new(ps_select_ts),
            ps_select_keys: Arc::new(ps_select_keys),
            ps_delete_latest: Arc::new(ps_delete_latest),
            ps_delete_ts: Arc::new(ps_delete_ts),
            partition_cache: DashMap::with_capacity(partition_cache_size),
        })
    }

    /// Insert vào ts_kv_partitions_cf nếu chưa có trong cache.
    async fn ensure_partition(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        key: &str,
        partition: i64,
    ) {
        let cache_key = (entity_type.to_string(), entity_id, key.to_string(), partition);
        if self.partition_cache.contains_key(&cache_key) {
            return;
        }

        let result = self
            .session
            .execute_unpaged(
                &self.ps_insert_partition,
                (entity_type, entity_id, key, partition),
            )
            .await;

        if let Err(e) = result {
            warn!("Failed to insert partition record: {e}");
        } else {
            self.partition_cache.insert(cache_key, ());
        }
    }

    /// Parse một row từ ts_kv_cf (SELECT ts, bool_v, str_v, long_v, dbl_v, json_v)
    fn parse_ts_row(
        entity_id: Uuid,
        key: &str,
        row: (i64, Option<bool>, Option<String>, Option<i64>, Option<f64>, Option<String>),
    ) -> TsRecord {
        let (ts, bool_v, str_v, long_v, dbl_v, json_v_str) = row;
        TsRecord {
            entity_id,
            key: key.to_string(),
            ts,
            bool_v,
            str_v,
            long_v,
            dbl_v,
            json_v: json_v_str.and_then(|s| serde_json::from_str(&s).ok()),
        }
    }

    /// Parse một row từ ts_kv_latest_cf (SELECT key, ts, bool_v, str_v, long_v, dbl_v, json_v)
    fn parse_latest_row(
        entity_id: Uuid,
        row: (String, i64, Option<bool>, Option<String>, Option<i64>, Option<f64>, Option<String>),
    ) -> TsRecord {
        let (key, ts, bool_v, str_v, long_v, dbl_v, json_v_str) = row;
        TsRecord {
            entity_id,
            key,
            ts,
            bool_v,
            str_v,
            long_v,
            dbl_v,
            json_v: json_v_str.and_then(|s| serde_json::from_str(&s).ok()),
        }
    }
}

#[async_trait]
impl TimeseriesDao for CassandraTs {
    #[instrument(skip(self, record))]
    async fn save(&self, entity_type: &str, record: &TsRecord) -> Result<(), DaoError> {
        let partition = to_partition_ts(record.ts, self.granularity);
        let json_v = record.json_v.as_ref().map(|v| v.to_string());

        if self.ttl_seconds > 0 {
            self.session
                .execute_unpaged(
                    &self.ps_insert_ts_ttl,
                    (
                        entity_type,
                        record.entity_id,
                        record.key.as_str(),
                        partition,
                        record.ts,
                        record.bool_v,
                        record.str_v.as_deref(),
                        record.long_v,
                        record.dbl_v,
                        json_v.as_deref(),
                        self.ttl_seconds as i32,
                    ),
                )
                .await
                .map_err(|e| DaoError::Cassandra(e.to_string()))?;
        } else {
            self.session
                .execute_unpaged(
                    &self.ps_insert_ts,
                    (
                        entity_type,
                        record.entity_id,
                        record.key.as_str(),
                        partition,
                        record.ts,
                        record.bool_v,
                        record.str_v.as_deref(),
                        record.long_v,
                        record.dbl_v,
                        json_v.as_deref(),
                    ),
                )
                .await
                .map_err(|e| DaoError::Cassandra(e.to_string()))?;
        }

        // Track partition (best-effort, không fail nếu lỗi)
        self.ensure_partition(entity_type, record.entity_id, &record.key, partition)
            .await;

        Ok(())
    }

    #[instrument(skip(self, record))]
    async fn save_latest(&self, entity_type: &str, record: &TsRecord) -> Result<(), DaoError> {
        let json_v = record.json_v.as_ref().map(|v| v.to_string());

        self.session
            .execute_unpaged(
                &self.ps_insert_latest,
                (
                    entity_type,
                    record.entity_id,
                    record.key.as_str(),
                    record.ts,
                    record.bool_v,
                    record.str_v.as_deref(),
                    record.long_v,
                    record.dbl_v,
                    json_v.as_deref(),
                ),
            )
            .await
            .map_err(|e| DaoError::Cassandra(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self, keys))]
    async fn find_latest(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: Option<&[&str]>,
    ) -> Result<Vec<TsRecord>, DaoError> {
        type LatestRow = (String, i64, Option<bool>, Option<String>, Option<i64>, Option<f64>, Option<String>);
        let mut records = Vec::new();

        match keys {
            None => {
                // Lấy tất cả keys
                let result = self
                    .session
                    .execute_unpaged(&self.ps_select_latest_all, (entity_type, entity_id))
                    .await
                    .map_err(|e| DaoError::Cassandra(e.to_string()))?;

                for row in result
                    .into_rows_result()
                    .map_err(|e| DaoError::Cassandra(e.to_string()))?
                    .rows::<LatestRow>()
                    .map_err(|e| DaoError::Cassandra(e.to_string()))?
                {
                    let row = row.map_err(|e| DaoError::Cassandra(e.to_string()))?;
                    records.push(Self::parse_latest_row(entity_id, row));
                }
            }
            Some(ks) => {
                // Lấy từng key riêng lẻ (Cassandra không hỗ trợ IN trên clustering key hiệu quả)
                for &key in ks {
                    let result = self
                        .session
                        .execute_unpaged(
                            &self.ps_select_latest_key,
                            (entity_type, entity_id, key),
                        )
                        .await
                        .map_err(|e| DaoError::Cassandra(e.to_string()))?;

                    for row in result
                        .into_rows_result()
                        .map_err(|e| DaoError::Cassandra(e.to_string()))?
                        .rows::<LatestRow>()
                        .map_err(|e| DaoError::Cassandra(e.to_string()))?
                    {
                        let row = row.map_err(|e| DaoError::Cassandra(e.to_string()))?;
                        records.push(Self::parse_latest_row(entity_id, row));
                    }
                }
            }
        }

        Ok(records)
    }

    #[instrument(skip(self))]
    async fn get_ts_keys(
        &self,
        entity_id: Uuid,
        entity_type: &str,
    ) -> Result<Vec<String>, DaoError> {
        let result = self
            .session
            .execute_unpaged(&self.ps_select_keys, (entity_type, entity_id))
            .await
            .map_err(|e| DaoError::Cassandra(e.to_string()))?;

        let mut keys = Vec::new();
        for row in result
            .into_rows_result()
            .map_err(|e| DaoError::Cassandra(e.to_string()))?
            .rows::<(String,)>()
            .map_err(|e| DaoError::Cassandra(e.to_string()))?
        {
            let (key,) = row.map_err(|e| DaoError::Cassandra(e.to_string()))?;
            keys.push(key);
        }
        Ok(keys)
    }

    #[instrument(skip(self))]
    async fn find_range(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<TsRecord>, DaoError> {
        type TsRow = (i64, Option<bool>, Option<String>, Option<i64>, Option<f64>, Option<String>);
        let partitions = partitions_in_range(start_ts, end_ts, self.granularity);
        let mut records = Vec::new();
        let remaining_limit = limit;

        for partition in partitions {
            if records.len() as i64 >= remaining_limit {
                break;
            }

            // CQL LIMIT expects Int (i32), không phải BigInt (i64)
            let batch_limit = (remaining_limit - records.len() as i64).min(10_000) as i32;

            let result = self
                .session
                .execute_unpaged(
                    &self.ps_select_ts,
                    (entity_type, entity_id, key, partition, start_ts, end_ts, batch_limit),
                )
                .await
                .map_err(|e| DaoError::Cassandra(e.to_string()))?;

            for row in result
                .into_rows_result()
                .map_err(|e| DaoError::Cassandra(e.to_string()))?
                .rows::<TsRow>()
                .map_err(|e| DaoError::Cassandra(e.to_string()))?
            {
                let row = row.map_err(|e| DaoError::Cassandra(e.to_string()))?;
                records.push(Self::parse_ts_row(entity_id, key, row));
            }
        }

        // Sort by ts DESC (đã order trong mỗi partition, nhưng cross-partition cần sort lại)
        records.sort_by(|a, b| b.ts.cmp(&a.ts));
        records.truncate(limit as usize);

        Ok(records)
    }

    #[instrument(skip(self))]
    async fn delete_ts(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: &[&str],
        start_ts: i64,
        end_ts: i64,
    ) -> Result<(), DaoError> {
        let partitions = partitions_in_range(start_ts, end_ts, self.granularity);

        for key in keys {
            for &partition in &partitions {
                self.session
                    .execute_unpaged(
                        &self.ps_delete_ts,
                        (entity_type, entity_id, *key, partition, start_ts, end_ts),
                    )
                    .await
                    .map_err(|e| DaoError::Cassandra(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Cassandra không có native time-bucket aggregation — fetch raw data rồi aggregate in-memory.
    async fn find_range_agg(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        interval_ms: i64,
        agg: vl_dao::AggType,
        limit: i64,
    ) -> Result<Vec<vl_core::entities::TsRecord>, DaoError> {
        use std::collections::BTreeMap;

        let interval_ms = interval_ms.max(1);
        let raw = self.find_range(entity_id, entity_type, key, start_ts, end_ts, 50_000).await?;

        // Group by bucket
        let mut buckets: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
        for r in &raw {
            let bucket = (r.ts / interval_ms) * interval_ms;
            let val = r.dbl_v
                .or_else(|| r.long_v.map(|v| v as f64));
            if let Some(v) = val {
                buckets.entry(bucket).or_default().push(v);
            } else if matches!(agg, vl_dao::AggType::Count) {
                buckets.entry(bucket).or_default();
            }
        }

        let result: Vec<_> = buckets.into_iter()
            .take(limit as usize)
            .filter_map(|(ts, vals)| {
                let agg_val = match agg {
                    vl_dao::AggType::Avg | vl_dao::AggType::None => {
                        if vals.is_empty() { return None; }
                        Some(vals.iter().sum::<f64>() / vals.len() as f64)
                    }
                    vl_dao::AggType::Min => vals.iter().cloned().reduce(f64::min),
                    vl_dao::AggType::Max => vals.iter().cloned().reduce(f64::max),
                    vl_dao::AggType::Sum => Some(vals.iter().sum()),
                    vl_dao::AggType::Count => Some(vals.len() as f64),
                };
                agg_val.map(|v| vl_core::entities::TsRecord {
                    entity_id,
                    key: key.to_string(),
                    ts,
                    dbl_v: Some(v),
                    bool_v: None,
                    long_v: None,
                    str_v: None,
                    json_v: None,
                })
            })
            .collect();

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn delete_latest(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: &[&str],
    ) -> Result<(), DaoError> {
        for key in keys {
            self.session
                .execute_unpaged(
                    &self.ps_delete_latest,
                    (entity_type, entity_id, *key),
                )
                .await
                .map_err(|e| DaoError::Cassandra(e.to_string()))?;

            // Xóa khỏi partition cache
            self.partition_cache
                .retain(|(et, eid, k, _), _| et != entity_type || *eid != entity_id || k != key);
        }

        Ok(())
    }
}
