use async_trait::async_trait;
use uuid::Uuid;

use vl_core::entities::{TsKvEntry, TsRecord};

use crate::{timeseries::{AggType, TimeseriesDao}, DaoError};

use super::kv::KvDao;

/// PostgreSQL implementation của TimeseriesDao.
/// Wraps KvDao và dịch String key ↔ i32 key_id thông qua key_dictionary.
/// entity_type không được dùng ở phía PostgreSQL (UUID đủ unique).
pub struct PostgresTsDao {
    inner: KvDao,
}

impl PostgresTsDao {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { inner: KvDao::new(pool) }
    }

    async fn to_key_id(&self, key: &str) -> Result<i32, DaoError> {
        self.inner.get_or_create_key(key).await
    }

    async fn to_key_ids(&self, keys: &[&str]) -> Result<Vec<i32>, DaoError> {
        let mut ids = Vec::with_capacity(keys.len());
        for key in keys {
            ids.push(self.inner.get_or_create_key(key).await?);
        }
        Ok(ids)
    }

    async fn resolve_key(&self, key_id: i32) -> Result<String, DaoError> {
        Ok(self.inner
            .get_key_name(key_id)
            .await?
            .unwrap_or_else(|| format!("key_{key_id}")))
    }
}

fn record_to_entry(record: &TsRecord, key_id: i32) -> TsKvEntry {
    TsKvEntry {
        entity_id: record.entity_id,
        key: key_id,
        ts: record.ts,
        bool_v: record.bool_v,
        str_v: record.str_v.clone(),
        long_v: record.long_v,
        dbl_v: record.dbl_v,
        json_v: record.json_v.clone(),
        version: 0,
    }
}

fn entry_to_record(entry: TsKvEntry, key_name: String) -> TsRecord {
    TsRecord {
        entity_id: entry.entity_id,
        key: key_name,
        ts: entry.ts,
        bool_v: entry.bool_v,
        str_v: entry.str_v,
        long_v: entry.long_v,
        dbl_v: entry.dbl_v,
        json_v: entry.json_v,
    }
}

#[async_trait]
impl TimeseriesDao for PostgresTsDao {
    async fn save(&self, _entity_type: &str, record: &TsRecord) -> Result<(), DaoError> {
        let key_id = self.to_key_id(&record.key).await?;
        let entry = record_to_entry(record, key_id);
        self.inner.save_ts(&entry).await
    }

    async fn save_latest(&self, _entity_type: &str, record: &TsRecord) -> Result<(), DaoError> {
        let key_id = self.to_key_id(&record.key).await?;
        let entry = record_to_entry(record, key_id);
        self.inner.save_latest(&entry).await
    }

    /// Bulk insert: resolve all unique keys at once, then 1 unnest INSERT.
    async fn save_batch(&self, _entity_type: &str, records: &[TsRecord]) -> Result<(), DaoError> {
        if records.is_empty() {
            return Ok(());
        }
        let unique_keys: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            records.iter().map(|r| r.key.as_str()).filter(|k| seen.insert(*k)).collect()
        };
        let key_map = self.inner.get_or_create_keys(&unique_keys).await?;
        let entries: Vec<vl_core::entities::TsKvEntry> = records.iter()
            .filter_map(|r| key_map.get(&r.key).map(|&kid| record_to_entry(r, kid)))
            .collect();
        self.inner.save_ts_batch(&entries).await
    }

    /// Bulk upsert latest: 1 unnest INSERT ON CONFLICT per call.
    async fn save_latest_batch(&self, _entity_type: &str, records: &[TsRecord]) -> Result<(), DaoError> {
        if records.is_empty() {
            return Ok(());
        }
        // Keep only the most recent record per (entity_id, key)
        let mut latest: std::collections::HashMap<(uuid::Uuid, &str), &TsRecord> = std::collections::HashMap::new();
        for r in records {
            let e = latest.entry((r.entity_id, r.key.as_str())).or_insert(r);
            if r.ts > e.ts { *e = r; }
        }
        let unique_keys: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            latest.values().map(|r| r.key.as_str()).filter(|k| seen.insert(*k)).collect()
        };
        let key_map = self.inner.get_or_create_keys(&unique_keys).await?;
        let entries: Vec<vl_core::entities::TsKvEntry> = latest.values()
            .filter_map(|r| key_map.get(&r.key).map(|&kid| record_to_entry(r, kid)))
            .collect();
        self.inner.save_latest_batch(&entries).await
    }

    async fn find_latest(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
        keys: Option<&[&str]>,
    ) -> Result<Vec<TsRecord>, DaoError> {
        let key_ids = if let Some(ks) = keys {
            self.to_key_ids(ks).await?
        } else {
            let all_keys = self.inner.get_ts_keys(entity_id).await?;
            let mut ids = Vec::with_capacity(all_keys.len());
            for k in &all_keys {
                if let Ok(id) = self.inner.get_or_create_key(k).await {
                    ids.push(id);
                }
            }
            ids
        };

        if key_ids.is_empty() {
            return Ok(Vec::new());
        }

        let entries = self.inner.find_latest(entity_id, &key_ids).await?;
        let mut records = Vec::with_capacity(entries.len());
        for entry in entries {
            let key_name = self.resolve_key(entry.key).await?;
            records.push(entry_to_record(entry, key_name));
        }
        Ok(records)
    }

    async fn get_ts_keys(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
    ) -> Result<Vec<String>, DaoError> {
        self.inner.get_ts_keys(entity_id).await
    }

    async fn find_range(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<TsRecord>, DaoError> {
        let key_id = self.to_key_id(key).await?;
        let entries = self.inner
            .find_ts_range(entity_id, key_id, start_ts, end_ts, limit)
            .await?;
        Ok(entries.into_iter()
            .map(|e| entry_to_record(e, key.to_string()))
            .collect())
    }

    async fn find_range_agg(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        interval_ms: i64,
        agg: AggType,
        limit: i64,
    ) -> Result<Vec<TsRecord>, DaoError> {
        let key_id = self.to_key_id(key).await?;
        let rows = self.inner
            .find_ts_agg(entity_id, key_id, start_ts, end_ts, interval_ms, agg.as_sql(), limit)
            .await?;
        Ok(rows.into_iter().filter_map(|(ts, val)| {
            val.map(|v| TsRecord {
                entity_id,
                key: key.to_string(),
                ts,
                dbl_v: Some(v),
                bool_v: None,
                long_v: None,
                str_v: None,
                json_v: None,
            })
        }).collect())
    }

    async fn delete_ts(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
        keys: &[&str],
        start_ts: i64,
        end_ts: i64,
    ) -> Result<(), DaoError> {
        let key_ids = self.to_key_ids(keys).await?;
        self.inner.delete_ts(entity_id, &key_ids, start_ts, end_ts).await
    }

    async fn delete_latest(
        &self,
        entity_id: Uuid,
        _entity_type: &str,
        keys: &[&str],
    ) -> Result<(), DaoError> {
        let key_ids = self.to_key_ids(keys).await?;
        self.inner.delete_ts_latest(entity_id, &key_ids).await
    }
}
