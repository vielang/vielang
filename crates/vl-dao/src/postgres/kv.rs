use std::collections::HashMap;
use std::time::Duration;

use moka::sync::Cache;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{TsKvEntry, AttributeKvEntry, AttributeScope};
use crate::DaoError;

pub struct KvDao {
    pool:      PgPool,
    /// In-memory bounded cache: key string → key_id (max 100k entries, 1h TTL).
    /// Cache hit → 0 DB calls. Cache miss → single INSERT...RETURNING, then cache.
    key_cache: Cache<String, i32>,
}

impl KvDao {
    pub fn new(pool: PgPool) -> Self {
        let key_cache = Cache::builder()
            .max_capacity(100_000)
            .time_to_live(Duration::from_secs(3600))
            .build();
        Self { pool, key_cache }
    }

    /// Pre-warm the key cache by loading all existing keys from key_dictionary.
    /// Call once at startup after migrations have run.
    pub async fn warm_up(&self) -> Result<(), DaoError> {
        let rows = sqlx::query!("SELECT key, key_id FROM key_dictionary")
            .fetch_all(&self.pool)
            .await?;
        let count = rows.len();
        for row in rows {
            self.key_cache.insert(row.key, row.key_id);
        }
        tracing::info!("KvDao key cache warmed up with {} keys", count);
        Ok(())
    }

    /// Upsert key vào dictionary, trả về key_id.
    /// Cache hit → 0 DB calls. Cache miss → single INSERT…RETURNING, then cache.
    pub async fn get_or_create_key(&self, key: &str) -> Result<i32, DaoError> {
        if let Some(id) = self.key_cache.get(key) {
            return Ok(id);
        }
        let row = sqlx::query!(
            "INSERT INTO key_dictionary (key) VALUES ($1)
             ON CONFLICT (key) DO UPDATE SET key = EXCLUDED.key
             RETURNING key_id",
            key
        )
        .fetch_one(&self.pool)
        .await?;

        self.key_cache.insert(key.to_string(), row.key_id);
        Ok(row.key_id)
    }

    /// Resolve nhiều keys cùng lúc, cache các key mới.
    /// Uses a single bulk INSERT…RETURNING — no separate SELECT needed.
    pub async fn get_or_create_keys(&self, keys: &[&str]) -> Result<HashMap<String, i32>, DaoError> {
        let mut result = HashMap::with_capacity(keys.len());
        let mut missing: Vec<&str> = Vec::new();

        for &key in keys {
            if let Some(id) = self.key_cache.get(key) {
                result.insert(key.to_string(), id);
            } else {
                missing.push(key);
            }
        }

        if !missing.is_empty() {
            let keys_vec: Vec<String> = missing.iter().map(|k| k.to_string()).collect();
            // Single bulk upsert + returning — eliminates N separate round-trips
            let rows: Vec<(String, i32)> = sqlx::query_as(
                "INSERT INTO key_dictionary (key)
                 SELECT unnest($1::text[])
                 ON CONFLICT (key) DO UPDATE SET key = EXCLUDED.key
                 RETURNING key, key_id",
            )
            .bind(&keys_vec)
            .fetch_all(&self.pool)
            .await
            .map_err(DaoError::from_sqlx)?;

            for (k, id) in rows {
                self.key_cache.insert(k.clone(), id);
                result.insert(k, id);
            }
        }
        Ok(result)
    }

    /// Bulk INSERT vào ts_kv dùng unnest() — 1 DB call thay vì N calls.
    pub async fn save_ts_batch(&self, entries: &[TsKvEntry]) -> Result<(), DaoError> {
        if entries.is_empty() {
            return Ok(());
        }
        let entity_ids: Vec<Uuid>         = entries.iter().map(|e| e.entity_id).collect();
        let keys:       Vec<i32>          = entries.iter().map(|e| e.key).collect();
        let tss:        Vec<i64>          = entries.iter().map(|e| e.ts).collect();
        let bool_vs:    Vec<Option<bool>> = entries.iter().map(|e| e.bool_v).collect();
        let str_vs:     Vec<Option<String>> = entries.iter().map(|e| e.str_v.clone()).collect();
        let long_vs:    Vec<Option<i64>>  = entries.iter().map(|e| e.long_v).collect();
        let dbl_vs:     Vec<Option<f64>>  = entries.iter().map(|e| e.dbl_v).collect();
        let json_vs:    Vec<Option<String>> = entries.iter()
            .map(|e| e.json_v.as_ref().map(|v| v.to_string()))
            .collect();

        // Use non-macro query to avoid sqlx compile-time issues with nullable array params
        sqlx::query(
            r#"
            INSERT INTO ts_kv (entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::int4[], $3::int8[],
                $4::bool[], $5::text[], $6::int8[], $7::float8[], $8::json[]
            )
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&entity_ids)
        .bind(&keys)
        .bind(&tss)
        .bind(&bool_vs)
        .bind(&str_vs)
        .bind(&long_vs)
        .bind(&dbl_vs)
        .bind(&json_vs)
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    /// Bulk UPSERT vào ts_kv_latest — chỉ giữ latest ts per (entity, key).
    pub async fn save_latest_batch(&self, entries: &[TsKvEntry]) -> Result<(), DaoError> {
        if entries.is_empty() {
            return Ok(());
        }
        let entity_ids: Vec<Uuid>           = entries.iter().map(|e| e.entity_id).collect();
        let keys:       Vec<i32>            = entries.iter().map(|e| e.key).collect();
        let tss:        Vec<i64>            = entries.iter().map(|e| e.ts).collect();
        let bool_vs:    Vec<Option<bool>>   = entries.iter().map(|e| e.bool_v).collect();
        let str_vs:     Vec<Option<String>> = entries.iter().map(|e| e.str_v.clone()).collect();
        let long_vs:    Vec<Option<i64>>    = entries.iter().map(|e| e.long_v).collect();
        let dbl_vs:     Vec<Option<f64>>    = entries.iter().map(|e| e.dbl_v).collect();
        let json_vs:    Vec<Option<String>> = entries.iter()
            .map(|e| e.json_v.as_ref().map(|v| v.to_string()))
            .collect();

        sqlx::query(
            r#"
            INSERT INTO ts_kv_latest (entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v, version)
            SELECT u.entity_id, u.key, u.ts, u.bool_v, u.str_v, u.long_v, u.dbl_v, u.json_v, 0
            FROM UNNEST(
                $1::uuid[], $2::int4[], $3::int8[],
                $4::bool[], $5::text[], $6::int8[], $7::float8[], $8::json[]
            ) AS u(entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v)
            ON CONFLICT (entity_id, key) DO UPDATE SET
                ts      = EXCLUDED.ts,
                bool_v  = EXCLUDED.bool_v,
                str_v   = EXCLUDED.str_v,
                long_v  = EXCLUDED.long_v,
                dbl_v   = EXCLUDED.dbl_v,
                json_v  = EXCLUDED.json_v,
                version = ts_kv_latest.version + 1
            WHERE ts_kv_latest.ts <= EXCLUDED.ts
            "#,
        )
        .bind(&entity_ids)
        .bind(&keys)
        .bind(&tss)
        .bind(&bool_vs)
        .bind(&str_vs)
        .bind(&long_vs)
        .bind(&dbl_vs)
        .bind(&json_vs)
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    /// Lưu latest telemetry value — khớp Java: TsKvRepository.saveLatest()
    #[instrument(skip(self))]
    pub async fn save_latest(&self, entry: &TsKvEntry) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO ts_kv_latest (entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0)
            ON CONFLICT (entity_id, key) DO UPDATE SET
                ts      = EXCLUDED.ts,
                bool_v  = EXCLUDED.bool_v,
                str_v   = EXCLUDED.str_v,
                long_v  = EXCLUDED.long_v,
                dbl_v   = EXCLUDED.dbl_v,
                json_v  = EXCLUDED.json_v,
                version = ts_kv_latest.version + 1
            "#,
            entry.entity_id,
            entry.key,
            entry.ts,
            entry.bool_v,
            entry.str_v,
            entry.long_v,
            entry.dbl_v,
            entry.json_v,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lưu timeseries (historical)
    #[instrument(skip(self))]
    pub async fn save_ts(&self, entry: &TsKvEntry) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO ts_kv (entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT DO NOTHING
            "#,
            entry.entity_id,
            entry.key,
            entry.ts,
            entry.bool_v,
            entry.str_v,
            entry.long_v,
            entry.dbl_v,
            entry.json_v,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lấy latest values cho một entity
    #[instrument(skip(self))]
    pub async fn find_latest(
        &self,
        entity_id: Uuid,
        keys: &[i32],
    ) -> Result<Vec<TsKvEntry>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v, version
            FROM ts_kv_latest
            WHERE entity_id = $1 AND key = ANY($2)
            "#,
            entity_id,
            keys,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TsKvEntry {
            entity_id: r.entity_id,
            key: r.key,
            ts: r.ts,
            bool_v: r.bool_v,
            str_v: r.str_v,
            long_v: r.long_v,
            dbl_v: r.dbl_v,
            json_v: r.json_v,
            version: r.version,
        }).collect())
    }

    /// Lịch sử telemetry trong khoảng thời gian
    #[instrument(skip(self))]
    pub async fn find_ts_range(
        &self,
        entity_id: Uuid,
        key: i32,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<TsKvEntry>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT entity_id, key, ts, bool_v, str_v, long_v, dbl_v, json_v
            FROM ts_kv
            WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
            ORDER BY ts DESC
            LIMIT $5
            "#,
            entity_id, key, start_ts, end_ts, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TsKvEntry {
            entity_id: r.entity_id,
            key: r.key,
            ts: r.ts,
            bool_v: r.bool_v,
            str_v: r.str_v,
            long_v: r.long_v,
            dbl_v: r.dbl_v,
            json_v: r.json_v,
            version: 0,
        }).collect())
    }

    /// Lưu attribute
    #[instrument(skip(self))]
    pub async fn save_attribute(&self, attr: &AttributeKvEntry) -> Result<(), DaoError> {
        let attr_type = attr.attribute_type as i32;
        sqlx::query!(
            r#"
            INSERT INTO attribute_kv (
                entity_id, attribute_type, attribute_key,
                bool_v, str_v, long_v, dbl_v, json_v,
                last_update_ts, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0)
            ON CONFLICT (entity_id, attribute_type, attribute_key) DO UPDATE SET
                bool_v         = EXCLUDED.bool_v,
                str_v          = EXCLUDED.str_v,
                long_v         = EXCLUDED.long_v,
                dbl_v          = EXCLUDED.dbl_v,
                json_v         = EXCLUDED.json_v,
                last_update_ts = EXCLUDED.last_update_ts,
                version        = attribute_kv.version + 1
            "#,
            attr.entity_id,
            attr_type,
            attr.attribute_key,
            attr.bool_v,
            attr.str_v,
            attr.long_v,
            attr.dbl_v,
            attr.json_v,
            attr.last_update_ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lấy attributes theo scope
    #[instrument(skip(self))]
    pub async fn find_attributes(
        &self,
        entity_id: Uuid,
        scope: AttributeScope,
        keys: Option<&[i32]>,
    ) -> Result<Vec<AttributeKvEntry>, DaoError> {
        let attr_type = scope as i32;
        let rows = sqlx::query!(
            r#"
            SELECT entity_id, attribute_type, attribute_key,
                   bool_v, str_v, long_v, dbl_v, json_v, last_update_ts, version
            FROM attribute_kv
            WHERE entity_id = $1
            AND attribute_type = $2
            AND ($3::int[] IS NULL OR attribute_key = ANY($3))
            "#,
            entity_id,
            attr_type,
            keys,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| AttributeKvEntry {
            entity_id: r.entity_id,
            attribute_type: AttributeScope::try_from(r.attribute_type).unwrap_or(scope),
            attribute_key: r.attribute_key,
            bool_v: r.bool_v,
            str_v: r.str_v,
            long_v: r.long_v,
            dbl_v: r.dbl_v,
            json_v: r.json_v,
            last_update_ts: r.last_update_ts,
            version: r.version,
        }).collect())
    }

    /// Batch lookup: key name → key_id (SELECT only, never inserts)
    #[instrument(skip(self))]
    pub async fn lookup_key_ids(&self, keys: &[String]) -> Result<HashMap<String, i32>, DaoError> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = sqlx::query!(
            "SELECT key, key_id FROM key_dictionary WHERE key = ANY($1::text[])",
            keys,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.key, r.key_id)).collect())
    }

    /// Reverse lookup: key_id → key string — dùng để resolve response
    #[instrument(skip(self))]
    pub async fn get_key_name(&self, key_id: i32) -> Result<Option<String>, DaoError> {
        let row = sqlx::query!(
            "SELECT key FROM key_dictionary WHERE key_id = $1",
            key_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.key))
    }

    /// Lấy tất cả key names của một entity (từ ts_kv_latest)
    #[instrument(skip(self))]
    pub async fn get_ts_keys(&self, entity_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT kd.key FROM ts_kv_latest tkl
            JOIN key_dictionary kd ON kd.key_id = tkl.key
            WHERE tkl.entity_id = $1
            "#,
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.key).collect())
    }

    /// GET /plugins/telemetry/{type}/{id}/keys/attributes — attribute key names for entity
    #[instrument(skip(self))]
    pub async fn get_attr_keys(&self, entity_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT kd.key FROM attribute_kv akv
            JOIN key_dictionary kd ON kd.key_id = akv.attribute_key
            WHERE akv.entity_id = $1
            "#,
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.key).collect())
    }

    /// Get attribute key names filtered by scope.
    #[instrument(skip(self))]
    pub async fn get_attr_keys_by_scope(
        &self,
        entity_id: Uuid,
        scope: AttributeScope,
    ) -> Result<Vec<String>, DaoError> {
        let attr_type = scope as i32;
        let rows = sqlx::query_as::<_, (String,)>(
            r#"SELECT DISTINCT kd.key FROM attribute_kv akv
               JOIN key_dictionary kd ON kd.key_id = akv.attribute_key
               WHERE akv.entity_id = $1 AND akv.attribute_type = $2"#,
        )
        .bind(entity_id)
        .bind(attr_type)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(k,)| k).collect())
    }

    /// Find attributes across ALL scopes for an entity.
    #[instrument(skip(self))]
    pub async fn find_all_attributes(
        &self,
        entity_id: Uuid,
        keys: Option<&[i32]>,
    ) -> Result<Vec<AttributeKvEntry>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT entity_id, attribute_type, attribute_key,
                   bool_v, str_v, long_v, dbl_v, json_v, last_update_ts, version
            FROM attribute_kv
            WHERE entity_id = $1
            AND ($2::int[] IS NULL OR attribute_key = ANY($2))
            "#,
            entity_id,
            keys,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| AttributeKvEntry {
            entity_id: r.entity_id,
            attribute_type: AttributeScope::try_from(r.attribute_type).unwrap_or(AttributeScope::ClientScope),
            attribute_key: r.attribute_key,
            bool_v: r.bool_v,
            str_v: r.str_v,
            long_v: r.long_v,
            dbl_v: r.dbl_v,
            json_v: r.json_v,
            last_update_ts: r.last_update_ts,
            version: r.version,
        }).collect())
    }

    /// Xóa timeseries trong khoảng thời gian cho một số keys
    #[instrument(skip(self))]
    pub async fn delete_ts(
        &self,
        entity_id: Uuid,
        key_ids: &[i32],
        start_ts: i64,
        end_ts: i64,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            DELETE FROM ts_kv
            WHERE entity_id = $1 AND key = ANY($2) AND ts >= $3 AND ts <= $4
            "#,
            entity_id, key_ids, start_ts, end_ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Xóa latest value cho các keys (khi deleteLatest = true)
    #[instrument(skip(self))]
    pub async fn delete_ts_latest(&self, entity_id: Uuid, key_ids: &[i32]) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM ts_kv_latest WHERE entity_id = $1 AND key = ANY($2)",
            entity_id, key_ids,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Xóa attributes theo scope và keys
    #[instrument(skip(self))]
    pub async fn delete_attributes(
        &self,
        entity_id: Uuid,
        scope: AttributeScope,
        key_ids: &[i32],
    ) -> Result<(), DaoError> {
        let attr_type = scope as i32;
        sqlx::query!(
            r#"
            DELETE FROM attribute_kv
            WHERE entity_id = $1 AND attribute_type = $2 AND attribute_key = ANY($3)
            "#,
            entity_id, attr_type, key_ids,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lấy danh sách unique timeseries keys theo tenant, optionally filter by device_profile_id
    #[instrument(skip(self))]
    pub async fn find_timeseries_keys_by_tenant(
        &self,
        tenant_id: Uuid,
        profile_id: Option<Uuid>,
    ) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT kd.key
            FROM ts_kv_latest t
            JOIN device d ON d.id = t.entity_id
            JOIN key_dictionary kd ON kd.key_id = t.key
            WHERE d.tenant_id = $1
              AND ($2::uuid IS NULL OR d.device_profile_id = $2)
            ORDER BY kd.key
            LIMIT 1000
            "#,
            tenant_id,
            profile_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.key).collect())
    }

    /// Aggregated timeseries — returns (bucket_ts, agg_value) per time bucket.
    /// Uses sqlx::query_as (without macro) because aggregate function is dynamic.
    /// agg_fn must be one of: AVG, MIN, MAX, SUM, COUNT (validated by caller).
    #[instrument(skip(self))]
    pub async fn find_ts_agg(
        &self,
        entity_id: Uuid,
        key: i32,
        start_ts: i64,
        end_ts: i64,
        interval_ms: i64,
        agg_fn: &str,
        limit: i64,
    ) -> Result<Vec<(i64, Option<f64>)>, DaoError> {
        let interval_ms = interval_ms.max(1);
        // Dynamic aggregate function — cannot use sqlx::query!() macro.
        // agg_fn is validated against a whitelist before this call.
        let sql: &str = match agg_fn {
            "AVG" => r#"
                SELECT (ts / $5) * $5 AS bucket,
                       AVG(COALESCE(dbl_v, long_v::double precision)) AS val
                FROM ts_kv
                WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
                GROUP BY bucket ORDER BY bucket ASC LIMIT $6"#,
            "MIN" => r#"
                SELECT (ts / $5) * $5 AS bucket,
                       MIN(COALESCE(dbl_v, long_v::double precision)) AS val
                FROM ts_kv
                WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
                GROUP BY bucket ORDER BY bucket ASC LIMIT $6"#,
            "MAX" => r#"
                SELECT (ts / $5) * $5 AS bucket,
                       MAX(COALESCE(dbl_v, long_v::double precision)) AS val
                FROM ts_kv
                WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
                GROUP BY bucket ORDER BY bucket ASC LIMIT $6"#,
            "SUM" => r#"
                SELECT (ts / $5) * $5 AS bucket,
                       SUM(COALESCE(dbl_v, long_v::double precision)) AS val
                FROM ts_kv
                WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
                GROUP BY bucket ORDER BY bucket ASC LIMIT $6"#,
            "COUNT" => r#"
                SELECT (ts / $5) * $5 AS bucket,
                       COUNT(*)::double precision AS val
                FROM ts_kv
                WHERE entity_id = $1 AND key = $2 AND ts >= $3 AND ts <= $4
                GROUP BY bucket ORDER BY bucket ASC LIMIT $6"#,
            _ => return Err(DaoError::Constraint(format!("Invalid agg: {}", agg_fn))),
        };

        let rows: Vec<(i64, Option<f64>)> = sqlx::query_as(sql)
            .bind(entity_id)
            .bind(key)
            .bind(start_ts)
            .bind(end_ts)
            .bind(interval_ms)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    /// Lấy danh sách unique attribute keys theo tenant, optionally filter by device_profile_id
    #[instrument(skip(self))]
    pub async fn find_attribute_keys_by_tenant(
        &self,
        tenant_id: Uuid,
        profile_id: Option<Uuid>,
    ) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT kd.key
            FROM attribute_kv a
            JOIN device d ON d.id = a.entity_id
            JOIN key_dictionary kd ON kd.key_id = a.attribute_key
            WHERE d.tenant_id = $1
              AND ($2::uuid IS NULL OR d.device_profile_id = $2)
            ORDER BY kd.key
            LIMIT 1000
            "#,
            tenant_id,
            profile_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.key).collect())
    }

    /// Look up a device's entity_id by matching a server-scope attribute key/value pair.
    /// Used by SNMP transport to resolve trap source IP → device via "snmpHost" attribute.
    pub async fn find_device_by_server_attr(
        &self,
        attr_key:   &str,
        attr_value: &str,
    ) -> Result<Option<Uuid>, DaoError> {
        // Resolve key → key_id first (read-only lookup, no insert)
        let key_ids = self.lookup_key_ids(&[attr_key.to_string()]).await?;
        let key_id = match key_ids.get(attr_key) {
            Some(id) => *id,
            None     => return Ok(None), // key not registered yet
        };

        // SERVER_SCOPE = 2 per AttributeScope enum
        let server_scope: i32 = 2;
        let row: Option<_> = sqlx::query!(
            r#"
            SELECT entity_id
            FROM attribute_kv
            WHERE attribute_key = $1
              AND str_v = $2
              AND attribute_type = $3
            LIMIT 1
            "#,
            key_id,
            attr_value,
            server_scope,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.entity_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "../../migrations")]
    async fn lookup_key_ids_empty_input(pool: PgPool) {
        let dao = KvDao::new(pool);
        let result = dao.lookup_key_ids(&[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn lookup_key_ids_nonexistent_keys(pool: PgPool) {
        let dao = KvDao::new(pool);
        let result = dao
            .lookup_key_ids(&["ghost_key".to_string(), "phantom".to_string()])
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn lookup_key_ids_existing_keys(pool: PgPool) {
        let dao = KvDao::new(pool);

        let id_temp = dao.get_or_create_key("temperature").await.unwrap();
        let id_hum  = dao.get_or_create_key("humidity").await.unwrap();

        let result = dao
            .lookup_key_ids(&["temperature".to_string(), "humidity".to_string()])
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result["temperature"], id_temp);
        assert_eq!(result["humidity"], id_hum);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn lookup_key_ids_partial_match(pool: PgPool) {
        let dao = KvDao::new(pool);

        let id_temp = dao.get_or_create_key("temperature").await.unwrap();

        let result = dao
            .lookup_key_ids(&["temperature".to_string(), "humidity".to_string()])
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result["temperature"], id_temp);
        assert!(!result.contains_key("humidity"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn lookup_key_ids_does_not_insert(pool: PgPool) {
        let dao = KvDao::new(pool);

        // lookup should not insert into key_dictionary
        dao.lookup_key_ids(&["brand_new_key".to_string()]).await.unwrap();

        let check = dao.lookup_key_ids(&["brand_new_key".to_string()]).await.unwrap();
        assert!(check.is_empty(), "lookup_key_ids must not insert keys");
    }
}
