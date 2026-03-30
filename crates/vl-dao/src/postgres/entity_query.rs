use sqlx::{PgPool, Row};
use uuid::Uuid;
use tracing::instrument;
use std::collections::HashMap;

use vl_core::entities::{
    AlarmCountQuery, AlarmData, AlarmDataQuery, EntityCountQuery, EntityData,
    EntityDataQuery, EntityFilter, EntityKey, EntityKeyType, EntityKeyValueType,
    KeyFilter, KeyFilterPredicate, NumericOperation, BooleanOperation,
    StringOperation, QueryEntityId, RelationFilter, TsValue,
};
use crate::{DaoError, PageData};

pub struct EntityQueryDao {
    pool: PgPool,
}

impl EntityQueryDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Count entities ────────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn count_entities(
        &self,
        tenant_id: Uuid,
        query: &EntityCountQuery,
    ) -> Result<i64, DaoError> {
        // Relation-based filters require graph traversal first
        if let Some(params) = extract_relation_params(&query.entity_filter) {
            let ids = self.resolve_relation_entity_ids(&params).await?;
            if ids.is_empty() {
                return Ok(0);
            }
            let count = if params.table.is_empty() {
                // RelationsQuery across multiple tables
                self.count_across_tables(tenant_id, &ids).await?
            } else {
                let mut sql = format!(
                    "SELECT COUNT(*) FROM {} WHERE tenant_id = $1 AND id = ANY($2)",
                    params.table
                );
                let mut bind_params =
                    vec![SqlParam::Uuid(tenant_id), SqlParam::UuidArray(ids)];
                append_subtype_filter(&mut sql, &params.subtypes);
                let filter_sql = build_key_filter_clauses(
                    &query.key_filters,
                    bind_params.len() + 1,
                    &mut bind_params,
                    params.table,
                );
                sql.push_str(&filter_sql);
                execute_count(&self.pool, &sql, &bind_params).await?
            };
            return Ok(count);
        }

        let (table, entity_type_str, extra_where) =
            filter_to_table_and_where(&query.entity_filter)?;

        let mut sql = format!(
            "SELECT COUNT(*) FROM {table} WHERE tenant_id = $1{extra_where}"
        );
        let mut params: Vec<SqlParam> = vec![SqlParam::Uuid(tenant_id)];
        let idx = params.len() + 1;

        let filter_sql = build_key_filter_clauses(&query.key_filters, idx, &mut params, table);
        sql.push_str(&filter_sql);

        if let Some(et) = entity_type_str {
            let n = params.len() + 1;
            sql.push_str(&format!(" AND type = ${n}"));
            params.push(SqlParam::Text(et.to_string()));
        }

        execute_count(&self.pool, &sql, &params).await
    }

    // ── Find entity data ──────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn find_entity_data(
        &self,
        tenant_id: Uuid,
        query: &EntityDataQuery,
    ) -> Result<PageData<EntityData>, DaoError> {
        // Relation-based filters handled separately
        if let Some(params) = extract_relation_params(&query.entity_filter) {
            return self.find_entity_data_via_relations(tenant_id, query, &params).await;
        }

        let (table, entity_type_str, extra_where) =
            filter_to_table_and_where(&query.entity_filter)?;

        let page_size = query.page_link.page_size.max(1).min(1000);
        let page     = query.page_link.page.max(0);
        let offset   = page * page_size;

        // COUNT
        let mut count_sql = format!(
            "SELECT COUNT(*) FROM {table} WHERE tenant_id = $1{extra_where}"
        );
        let mut count_params: Vec<SqlParam> = vec![SqlParam::Uuid(tenant_id)];
        let idx = count_params.len() + 1;
        let filter_sql = build_key_filter_clauses(&query.key_filters, idx, &mut count_params, table);
        count_sql.push_str(&filter_sql);
        if let Some(et) = entity_type_str {
            let n = count_params.len() + 1;
            count_sql.push_str(&format!(" AND type = ${n}"));
            count_params.push(SqlParam::Text(et.to_string()));
        }
        let total = execute_count(&self.pool, &count_sql, &count_params).await?;
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;

        // SELECT
        let mut data_params: Vec<SqlParam> = vec![SqlParam::Uuid(tenant_id)];
        let mut data_sql = format!(
            "SELECT id, created_time, name, type, label, customer_id FROM {table} WHERE tenant_id = $1{extra_where}"
        );
        let idx = data_params.len() + 1;
        let filter_sql = build_key_filter_clauses(&query.key_filters, idx, &mut data_params, table);
        data_sql.push_str(&filter_sql);
        if let Some(et) = entity_type_str {
            let n = data_params.len() + 1;
            data_sql.push_str(&format!(" AND type = ${n}"));
            data_params.push(SqlParam::Text(et.to_string()));
        }

        let order = sort_clause(&query.page_link.sort_order);
        data_sql.push_str(&order);
        let n = data_params.len() + 1;
        data_sql.push_str(&format!(" LIMIT ${n}"));
        data_params.push(SqlParam::I64(page_size));
        let n = data_params.len() + 1;
        data_sql.push_str(&format!(" OFFSET ${n}"));
        data_params.push(SqlParam::I64(offset));

        let entity_kind = table_to_entity_type(table);
        let rows = execute_entity_rows(&self.pool, &data_sql, &data_params, entity_kind).await?;
        let data = self.entity_rows_to_data(rows, query).await?;

        Ok(PageData {
            data,
            total_pages,
            total_elements: total,
            has_next: (page + 1) < total_pages,
        })
    }

    // ── Find entity data via relation traversal ───────────────────────────────

    async fn find_entity_data_via_relations(
        &self,
        tenant_id: Uuid,
        query: &EntityDataQuery,
        params: &RelationFilterParams,
    ) -> Result<PageData<EntityData>, DaoError> {
        let ids = self.resolve_relation_entity_ids(params).await?;
        if ids.is_empty() {
            return Ok(PageData { data: vec![], total_pages: 0, total_elements: 0, has_next: false });
        }

        let page_size = query.page_link.page_size.max(1).min(1000);
        let page     = query.page_link.page.max(0);
        let offset   = page * page_size;
        let order    = sort_clause(&query.page_link.sort_order);

        let (total, rows) = if params.table.is_empty() {
            // RelationsQuery — query across device + asset + entity_view tables
            let count = self.count_across_tables(tenant_id, &ids).await?;

            let sql = format!(
                "SELECT id, created_time, name, type, label, customer_id, 'DEVICE' AS entity_kind \
                 FROM device WHERE tenant_id = $1 AND id = ANY($2) \
                 UNION ALL \
                 SELECT id, created_time, name, type, label, customer_id, 'ASSET' AS entity_kind \
                 FROM asset WHERE tenant_id = $1 AND id = ANY($2) \
                 UNION ALL \
                 SELECT id, created_time, name, type, label, customer_id, 'ENTITY_VIEW' AS entity_kind \
                 FROM entity_view WHERE tenant_id = $1 AND id = ANY($2){order} LIMIT $3 OFFSET $4"
            );
            let bind_params = vec![
                SqlParam::Uuid(tenant_id),
                SqlParam::UuidArray(ids),
                SqlParam::I64(page_size),
                SqlParam::I64(offset),
            ];
            let rows = execute_entity_rows(&self.pool, &sql, &bind_params, "DEVICE").await?;
            (count, rows)
        } else {
            // Typed search (AssetSearch, DeviceSearch, EntityViewSearch) — single table
            let mut count_sql = format!(
                "SELECT COUNT(*) FROM {} WHERE tenant_id = $1 AND id = ANY($2)",
                params.table
            );
            let mut count_params = vec![SqlParam::Uuid(tenant_id), SqlParam::UuidArray(ids.clone())];
            append_subtype_filter(&mut count_sql, &params.subtypes);
            let count = execute_count(&self.pool, &count_sql, &count_params).await?;

            let mut data_sql = format!(
                "SELECT id, created_time, name, type, label, customer_id \
                 FROM {} WHERE tenant_id = $1 AND id = ANY($2)",
                params.table
            );
            append_subtype_filter(&mut data_sql, &params.subtypes);
            data_sql.push_str(&order);
            let n = count_params.len() + 1;
            data_sql.push_str(&format!(" LIMIT ${n}"));
            count_params.push(SqlParam::I64(page_size));
            let n = count_params.len() + 1;
            data_sql.push_str(&format!(" OFFSET ${n}"));
            count_params.push(SqlParam::I64(offset));

            let rows = execute_entity_rows(&self.pool, &data_sql, &count_params, params.entity_kind).await?;
            (count, rows)
        };

        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
        let data = self.entity_rows_to_data(rows, query).await?;

        Ok(PageData {
            data,
            total_pages,
            total_elements: total,
            has_next: (page + 1) < total_pages,
        })
    }

    // ── Count alarms ──────────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn count_alarms(
        &self,
        tenant_id: Uuid,
        query: &AlarmCountQuery,
    ) -> Result<i64, DaoError> {
        let originator_ids = self.resolve_originator_ids(tenant_id, &query.entity_filter).await?;
        if originator_ids.is_empty() {
            return Ok(0);
        }

        let mut sql = String::from(
            "SELECT COUNT(*) FROM alarm WHERE tenant_id = $1 AND originator_id = ANY($2)"
        );
        let mut params: Vec<SqlParam> = vec![
            SqlParam::Uuid(tenant_id),
            SqlParam::UuidArray(originator_ids),
        ];

        append_alarm_filter_clauses(&mut sql, &mut params,
            &query.alarm_status_list, &query.alarm_severity_list,
            &query.alarm_type_list, query.start_ts, query.end_ts);

        execute_count(&self.pool, &sql, &params).await
    }

    // ── Find alarm data ───────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn find_alarm_data(
        &self,
        tenant_id: Uuid,
        query: &AlarmDataQuery,
    ) -> Result<PageData<AlarmData>, DaoError> {
        let originator_ids = self.resolve_originator_ids(tenant_id, &query.entity_filter).await?;
        if originator_ids.is_empty() {
            return Ok(PageData { data: vec![], total_pages: 0, total_elements: 0, has_next: false });
        }

        let page_size = query.page_link.page_size.max(1).min(1000);
        let page     = query.page_link.page.max(0);
        let offset   = page * page_size;

        let mut count_params: Vec<SqlParam> = vec![
            SqlParam::Uuid(tenant_id),
            SqlParam::UuidArray(originator_ids.clone()),
        ];
        let mut count_sql = String::from(
            "SELECT COUNT(*) FROM alarm WHERE tenant_id = $1 AND originator_id = ANY($2)"
        );
        append_alarm_filter_clauses(&mut count_sql, &mut count_params,
            &query.page_link.alarm_status_list, &query.page_link.alarm_severity_list,
            &query.page_link.alarm_type_list, query.page_link.start_ts, query.page_link.end_ts);
        let total = execute_count(&self.pool, &count_sql, &count_params).await?;
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;

        let mut data_params: Vec<SqlParam> = vec![
            SqlParam::Uuid(tenant_id),
            SqlParam::UuidArray(originator_ids),
        ];
        let mut data_sql = String::from(
            "SELECT id, created_time, originator_id, originator_type, type, severity, \
             acknowledged, cleared, ack_ts, clear_ts \
             FROM alarm WHERE tenant_id = $1 AND originator_id = ANY($2)"
        );
        append_alarm_filter_clauses(&mut data_sql, &mut data_params,
            &query.page_link.alarm_status_list, &query.page_link.alarm_severity_list,
            &query.page_link.alarm_type_list, query.page_link.start_ts, query.page_link.end_ts);

        data_sql.push_str(" ORDER BY created_time DESC");
        let n = data_params.len() + 1;
        data_sql.push_str(&format!(" LIMIT ${n}"));
        data_params.push(SqlParam::I64(page_size));
        let n = data_params.len() + 1;
        data_sql.push_str(&format!(" OFFSET ${n}"));
        data_params.push(SqlParam::I64(offset));

        let rows = execute_alarm_rows(&self.pool, &data_sql, &data_params).await?;

        let data: Vec<AlarmData> = rows.into_iter().map(|r| AlarmData {
            entity_id: QueryEntityId {
                id: r.originator_id,
                entity_type: r.originator_type,
            },
            alarm_id: QueryEntityId {
                id: r.id,
                entity_type: "ALARM".to_string(),
            },
            created_time: r.created_time,
            ack_ts:   r.ack_ts.unwrap_or(0),
            clear_ts: r.clear_ts.unwrap_or(0),
            originator_name:  None,
            originator_label: None,
            severity: r.severity,
            status: alarm_status(r.acknowledged, r.cleared),
            alarm_type: r.alarm_type,
            acknowledged: r.acknowledged,
            cleared: r.cleared,
            latest: HashMap::new(),
        }).collect();

        Ok(PageData {
            data,
            total_pages,
            total_elements: total,
            has_next: (page + 1) < total_pages,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Resolve originator_ids for alarm queries from any entity filter.
    async fn resolve_originator_ids(
        &self,
        tenant_id: Uuid,
        filter: &EntityFilter,
    ) -> Result<Vec<Uuid>, DaoError> {
        // Relation-based filters use graph traversal
        if let Some(params) = extract_relation_params(filter) {
            return self.resolve_relation_entity_ids(&params).await;
        }

        let (table, type_filter, extra_where) = filter_to_table_and_where(filter)?;

        let mut sql = format!(
            "SELECT id FROM {table} WHERE tenant_id = $1{extra_where}"
        );
        let mut params: Vec<SqlParam> = vec![SqlParam::Uuid(tenant_id)];
        if let Some(et) = type_filter {
            let n = params.len() + 1;
            sql.push_str(&format!(" AND type = ${n}"));
            params.push(SqlParam::Text(et.to_string()));
        }

        execute_uuid_list(&self.pool, &sql, &params).await
    }

    /// Traverse the relation graph and return entity IDs matching the filter params.
    ///
    /// Uses a PostgreSQL WITH RECURSIVE CTE for multi-hop traversal.
    async fn resolve_relation_entity_ids(
        &self,
        params: &RelationFilterParams,
    ) -> Result<Vec<Uuid>, DaoError> {
        let max_lvl = params.max_level.max(1).min(50) as i64;
        let rel_filter = build_relation_where_fragment(&params.filters, "r.");

        let last_level_clause = if params.fetch_last_level_only {
            format!(" AND lvl = {max_lvl}")
        } else {
            String::new()
        };

        let type_filter = if !params.target_entity_type.is_empty() {
            format!(" AND entity_type = '{}'", escape_sql_string(params.target_entity_type))
        } else {
            String::new()
        };

        // Build CTE depending on traversal direction
        let sql = if params.direction.eq_ignore_ascii_case("FROM") {
            format!(
                "WITH RECURSIVE rel_tree(entity_id, entity_type, lvl) AS (\
                    SELECT r.to_id, r.to_type, 1 \
                    FROM relation r \
                    WHERE r.from_id = $1 AND r.from_type = $2{rel_filter} \
                    UNION ALL \
                    SELECT r.to_id, r.to_type, rt.lvl + 1 \
                    FROM relation r \
                    JOIN rel_tree rt ON r.from_id = rt.entity_id AND r.from_type = rt.entity_type \
                    WHERE rt.lvl < $3{rel_filter}\
                ) \
                SELECT DISTINCT entity_id AS id FROM rel_tree \
                WHERE TRUE{type_filter}{last_level_clause}"
            )
        } else {
            // Direction: TO — traverse from children back to ancestors
            format!(
                "WITH RECURSIVE rel_tree(entity_id, entity_type, lvl) AS (\
                    SELECT r.from_id, r.from_type, 1 \
                    FROM relation r \
                    WHERE r.to_id = $1 AND r.to_type = $2{rel_filter} \
                    UNION ALL \
                    SELECT r.from_id, r.from_type, rt.lvl + 1 \
                    FROM relation r \
                    JOIN rel_tree rt ON r.to_id = rt.entity_id AND r.to_type = rt.entity_type \
                    WHERE rt.lvl < $3{rel_filter}\
                ) \
                SELECT DISTINCT entity_id AS id FROM rel_tree \
                WHERE TRUE{type_filter}{last_level_clause}"
            )
        };

        execute_uuid_list(
            &self.pool,
            &sql,
            &[
                SqlParam::Uuid(params.root_id),
                SqlParam::Text(params.root_type.clone()),
                SqlParam::I64(max_lvl),
            ],
        )
        .await
    }

    /// Count entities across device, asset, entity_view tables (for RelationsQuery).
    async fn count_across_tables(&self, tenant_id: Uuid, ids: &[Uuid]) -> Result<i64, DaoError> {
        let sql =
            "SELECT COUNT(*) FROM (\
                SELECT id FROM device WHERE tenant_id = $1 AND id = ANY($2) \
                UNION ALL \
                SELECT id FROM asset WHERE tenant_id = $1 AND id = ANY($2) \
                UNION ALL \
                SELECT id FROM entity_view WHERE tenant_id = $1 AND id = ANY($2)\
            ) sub";
        execute_count(
            &self.pool,
            sql,
            &[SqlParam::Uuid(tenant_id), SqlParam::UuidArray(ids.to_vec())],
        )
        .await
    }

    /// Convert EntityRows into EntityData, fetching attribute/TS values as needed.
    async fn entity_rows_to_data(
        &self,
        rows: Vec<EntityRow>,
        query: &EntityDataQuery,
    ) -> Result<Vec<EntityData>, DaoError> {
        let mut data: Vec<EntityData> = Vec::with_capacity(rows.len());
        for row in &rows {
            let entity_id = QueryEntityId {
                id: row.id,
                entity_type: row.entity_kind.clone(),
            };
            let mut latest: HashMap<String, HashMap<String, TsValue>> = HashMap::new();

            if !query.entity_fields.is_empty() {
                let ef_map = build_entity_fields_map(row, &query.entity_fields);
                if !ef_map.is_empty() {
                    latest.insert("ENTITY_FIELD".to_string(), ef_map);
                }
            }

            let attr_keys: Vec<&EntityKey> = query.latest_values.iter()
                .filter(|k| matches!(
                    k.key_type,
                    EntityKeyType::Attribute |
                    EntityKeyType::ServerAttribute |
                    EntityKeyType::SharedAttribute |
                    EntityKeyType::ClientAttribute
                ))
                .collect();
            if !attr_keys.is_empty() {
                let attr_map = self.fetch_latest_attributes(row.id, &attr_keys).await?;
                if !attr_map.is_empty() {
                    latest.insert("ATTRIBUTE".to_string(), attr_map);
                }
            }

            let ts_keys: Vec<&EntityKey> = query.latest_values.iter()
                .filter(|k| k.key_type == EntityKeyType::TimeSeries)
                .collect();
            if !ts_keys.is_empty() {
                let ts_map = self.fetch_latest_timeseries(row.id, &ts_keys).await?;
                if !ts_map.is_empty() {
                    latest.insert("TIME_SERIES".to_string(), ts_map);
                }
            }

            data.push(EntityData {
                entity_id,
                latest,
                timeseries: HashMap::new(),
            });
        }
        Ok(data)
    }

    /// Fetch latest attribute values for one entity.
    async fn fetch_latest_attributes(
        &self,
        entity_id: Uuid,
        keys: &[&EntityKey],
    ) -> Result<HashMap<String, TsValue>, DaoError> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let key_names: Vec<String> = keys.iter().map(|k| k.key.clone()).collect();
        let rows = sqlx::query!(
            r#"
            SELECT k.key, akv.str_v, akv.long_v, akv.dbl_v, akv.bool_v, akv.last_update_ts
            FROM attribute_kv akv
            JOIN key_dictionary k ON k.key_id = akv.attribute_key
            WHERE akv.entity_id = $1
              AND k.key = ANY($2::text[])
            "#,
            entity_id,
            &key_names as &[String],
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map = HashMap::new();
        for r in rows {
            let value = r.str_v
                .or_else(|| r.long_v.map(|v: i64| v.to_string()))
                .or_else(|| r.dbl_v.map(|v: f64| v.to_string()))
                .or_else(|| r.bool_v.map(|v: bool| v.to_string()))
                .unwrap_or_default();
            map.insert(r.key, TsValue {
                ts: r.last_update_ts,
                value,
            });
        }
        Ok(map)
    }

    /// Fetch latest timeseries values for one entity.
    async fn fetch_latest_timeseries(
        &self,
        entity_id: Uuid,
        keys: &[&EntityKey],
    ) -> Result<HashMap<String, TsValue>, DaoError> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let key_names: Vec<String> = keys.iter().map(|k| k.key.clone()).collect();
        let rows = sqlx::query!(
            r#"
            SELECT k.key, lkv.str_v, lkv.long_v, lkv.dbl_v, lkv.bool_v, lkv.ts
            FROM ts_kv_latest lkv
            JOIN key_dictionary k ON k.key_id = lkv.key
            WHERE lkv.entity_id = $1
              AND k.key = ANY($2::text[])
            "#,
            entity_id,
            &key_names as &[String],
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map = HashMap::new();
        for r in rows {
            let value = r.str_v
                .or_else(|| r.long_v.map(|v: i64| v.to_string()))
                .or_else(|| r.dbl_v.map(|v: f64| v.to_string()))
                .or_else(|| r.bool_v.map(|v: bool| v.to_string()))
                .unwrap_or_default();
            map.insert(r.key, TsValue { ts: r.ts, value });
        }
        Ok(map)
    }
}

// ── SQL parameter enum (dùng cho dynamic queries) ─────────────────────────────

enum SqlParam {
    Uuid(Uuid),
    UuidArray(Vec<Uuid>),
    Text(String),
    I64(i64),
    F64(f64),
    Bool(bool),
}

// ── Relation filter params ─────────────────────────────────────────────────────

/// Extracted parameters from a relation-based EntityFilter variant.
struct RelationFilterParams {
    root_id:              Uuid,
    root_type:            String,
    /// "FROM" or "TO"
    direction:            String,
    filters:              Vec<RelationFilter>,
    max_level:            i32,
    fetch_last_level_only: bool,
    /// e.g., "DEVICE", "ASSET", "" (empty = all types for RelationsQuery)
    target_entity_type:   &'static str,
    /// DB table name (e.g., "device"), empty = multi-table (RelationsQuery)
    table:                &'static str,
    /// Entity kind label for single-table queries
    entity_kind:          &'static str,
    /// Optional subtype filter (e.g., device_types, asset_types)
    subtypes:             Vec<String>,
}

/// Extract relation traversal parameters from an EntityFilter, if applicable.
fn extract_relation_params(filter: &EntityFilter) -> Option<RelationFilterParams> {
    match filter {
        EntityFilter::RelationsQuery {
            root_entity, direction, filters, max_level, fetch_last_level_only,
        } => Some(RelationFilterParams {
            root_id: root_entity.id,
            root_type: root_entity.entity_type.clone(),
            direction: direction.clone(),
            filters: filters.clone(),
            max_level: max_level.unwrap_or(10),
            fetch_last_level_only: *fetch_last_level_only,
            target_entity_type: "",   // all entity types
            table: "",                // multi-table
            entity_kind: "DEVICE",
            subtypes: vec![],
        }),
        EntityFilter::AssetSearchQuery {
            root_entity, direction, filters, max_level, fetch_last_level_only, asset_types,
        } => Some(RelationFilterParams {
            root_id: root_entity.id,
            root_type: root_entity.entity_type.clone(),
            direction: direction.clone(),
            filters: filters.clone(),
            max_level: max_level.unwrap_or(10),
            fetch_last_level_only: *fetch_last_level_only,
            target_entity_type: "ASSET",
            table: "asset",
            entity_kind: "ASSET",
            subtypes: asset_types.clone(),
        }),
        EntityFilter::DeviceSearchQuery {
            root_entity, direction, filters, max_level, fetch_last_level_only, device_types,
        } => Some(RelationFilterParams {
            root_id: root_entity.id,
            root_type: root_entity.entity_type.clone(),
            direction: direction.clone(),
            filters: filters.clone(),
            max_level: max_level.unwrap_or(10),
            fetch_last_level_only: *fetch_last_level_only,
            target_entity_type: "DEVICE",
            table: "device",
            entity_kind: "DEVICE",
            subtypes: device_types.clone(),
        }),
        EntityFilter::EntityViewSearchQuery {
            root_entity, direction, filters, max_level, fetch_last_level_only, entity_view_types,
        } => Some(RelationFilterParams {
            root_id: root_entity.id,
            root_type: root_entity.entity_type.clone(),
            direction: direction.clone(),
            filters: filters.clone(),
            max_level: max_level.unwrap_or(10),
            fetch_last_level_only: *fetch_last_level_only,
            target_entity_type: "ENTITY_VIEW",
            table: "entity_view",
            entity_kind: "ENTITY_VIEW",
            subtypes: entity_view_types.clone(),
        }),
        _ => None,
    }
}

/// Build an inline SQL WHERE fragment from a list of RelationFilters.
///
/// Each filter is OR'd; within a filter, type_group and relation_type are AND'd.
/// `prefix` is typically `"r."` to qualify column names (e.g., inside a JOIN).
fn build_relation_where_fragment(filters: &[RelationFilter], prefix: &str) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let conditions: Vec<String> = filters
        .iter()
        .filter_map(|f| {
            let mut parts: Vec<String> = Vec::new();
            if let Some(rg) = &f.relation_type_group {
                parts.push(format!(
                    "{prefix}relation_type_group = '{}'",
                    escape_sql_string(rg)
                ));
            }
            if let Some(rt) = &f.relation_type {
                parts.push(format!(
                    "{prefix}relation_type = '{}'",
                    escape_sql_string(rt)
                ));
            }
            if parts.is_empty() {
                None
            } else {
                Some(format!("({})", parts.join(" AND ")))
            }
        })
        .collect();

    if conditions.is_empty() {
        String::new()
    } else {
        format!(" AND ({})", conditions.join(" OR "))
    }
}

// ── Filter helpers ────────────────────────────────────────────────────────────

/// Map EntityFilter to (table, type_column_value, extra_where_fragment).
/// Relation-based filters are handled by `extract_relation_params` — callers must
/// check `extract_relation_params` before calling this function.
fn filter_to_table_and_where(
    filter: &EntityFilter,
) -> Result<(&'static str, Option<&str>, String), crate::DaoError> {
    match filter {
        EntityFilter::EntityType { entity_type } => {
            Ok((entity_type_to_table(entity_type)?, None, String::new()))
        }
        EntityFilter::DeviceType { device_type, device_name_filter } => {
            let mut extra = format!(" AND type = '{}'", escape_sql_string(device_type));
            if let Some(nf) = device_name_filter {
                extra.push_str(&format!(
                    " AND LOWER(name) LIKE LOWER('%{}%')",
                    escape_sql_string(nf)
                ));
            }
            Ok(("device", None, extra))
        }
        EntityFilter::AssetType { asset_type, asset_name_filter } => {
            let mut extra = format!(" AND type = '{}'", escape_sql_string(asset_type));
            if let Some(nf) = asset_name_filter {
                extra.push_str(&format!(
                    " AND LOWER(name) LIKE LOWER('%{}%')",
                    escape_sql_string(nf)
                ));
            }
            Ok(("asset", None, extra))
        }
        EntityFilter::EntityName { entity_type, entity_name_filter } => {
            let table = entity_type_to_table(entity_type)?;
            let extra = format!(
                " AND LOWER(name) LIKE LOWER('%{}%')",
                escape_sql_string(entity_name_filter)
            );
            Ok((table, None, extra))
        }
        EntityFilter::EntityList { entity_type, entity_ids } => {
            let table = entity_type_to_table(entity_type)?;
            if entity_ids.is_empty() {
                Ok((table, None, " AND FALSE".to_string()))
            } else {
                let list: Vec<String> = entity_ids.iter()
                    .map(|id| format!("'{id}'"))
                    .collect();
                let extra = format!(" AND id IN ({})", list.join(","));
                Ok((table, None, extra))
            }
        }
        EntityFilter::SingleEntity { single_entity } => {
            let table = entity_type_to_table(&single_entity.entity_type)?;
            let extra = format!(" AND id = '{}'", single_entity.id);
            Ok((table, None, extra))
        }
        EntityFilter::EntityViewType { entity_view_type, entity_view_name_filter } => {
            let mut extra = format!(" AND type = '{}'", escape_sql_string(entity_view_type));
            if let Some(nf) = entity_view_name_filter {
                extra.push_str(&format!(
                    " AND LOWER(name) LIKE LOWER('%{}%')",
                    escape_sql_string(nf)
                ));
            }
            Ok(("entity_view", None, extra))
        }
        EntityFilter::EdgeType { edge_type, edge_name_filter } => {
            let mut extra = format!(" AND type = '{}'", escape_sql_string(edge_type));
            if let Some(nf) = edge_name_filter {
                extra.push_str(&format!(
                    " AND LOWER(name) LIKE LOWER('%{}%')",
                    escape_sql_string(nf)
                ));
            }
            Ok(("edge", None, extra))
        }
        EntityFilter::ApiUsageState { entity_type } => {
            Ok((entity_type_to_table(entity_type)?, None, String::new()))
        }
        EntityFilter::SubCustomers { root_customer_id } => {
            let extra = format!(
                " AND parent_customer_id = '{}'",
                root_customer_id
            );
            Ok(("customer", None, extra))
        }
        // Relation-based filters are handled by resolve_relation_entity_ids
        EntityFilter::RelationsQuery { .. } |
        EntityFilter::AssetSearchQuery { .. } |
        EntityFilter::DeviceSearchQuery { .. } |
        EntityFilter::EntityViewSearchQuery { .. } => {
            // Should not reach here — callers must check extract_relation_params first
            Ok(("device", None, " AND FALSE".to_string()))
        }
    }
}

fn entity_type_to_table(entity_type: &str) -> Result<&'static str, crate::DaoError> {
    match entity_type.to_uppercase().as_str() {
        "DEVICE"      => Ok("device"),
        "ASSET"       => Ok("asset"),
        "CUSTOMER"    => Ok("customer"),
        "TENANT"      => Ok("tenant"),
        "ENTITY_VIEW" => Ok("entity_view"),
        "EDGE"        => Ok("edge"),
        "DASHBOARD"   => Ok("dashboard"),
        "USER"        => Ok("tb_user"),
        other         => Err(crate::DaoError::InvalidInput(format!("Unknown entity type: {other}"))),
    }
}

fn table_to_entity_type(table: &str) -> &'static str {
    match table {
        "device"      => "DEVICE",
        "asset"       => "ASSET",
        "customer"    => "CUSTOMER",
        "tenant"      => "TENANT",
        "entity_view" => "ENTITY_VIEW",
        "edge"        => "EDGE",
        "dashboard"   => "DASHBOARD",
        "tb_user"     => "USER",
        _             => "DEVICE",
    }
}

/// Escape single quotes in strings used inside inline SQL fragments.
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Append subtype (device_types / asset_types / etc.) filter to SQL.
fn append_subtype_filter(sql: &mut String, subtypes: &[String]) {
    if !subtypes.is_empty() {
        let list: Vec<String> = subtypes.iter()
            .map(|s| format!("'{}'", escape_sql_string(s)))
            .collect();
        sql.push_str(&format!(" AND type IN ({})", list.join(",")));
    }
}

/// Build WHERE clauses from key_filters.
///
/// Hỗ trợ:
/// - `EntityField` — so sánh trực tiếp với columns của entity table
/// - `Attribute` / `*Attribute` — EXISTS subquery vào attribute_kv
/// - `TimeSeries` — EXISTS subquery vào ts_kv_latest
fn build_key_filter_clauses(
    filters: &[KeyFilter],
    start_idx: usize,
    params: &mut Vec<SqlParam>,
    _table: &str,
) -> String {
    let mut sql = String::new();
    let mut next_n = start_idx;
    for f in filters {
        let frag = match f.key.key_type {
            EntityKeyType::EntityField => build_entity_field_clause(f),
            EntityKeyType::Attribute
            | EntityKeyType::ServerAttribute
            | EntityKeyType::SharedAttribute
            | EntityKeyType::ClientAttribute => {
                let n = next_n;
                next_n += 1;
                params.push(SqlParam::Text(f.key.key.clone()));
                build_attribute_exists_clause(f, n)
            }
            EntityKeyType::TimeSeries => {
                let n = next_n;
                next_n += 1;
                params.push(SqlParam::Text(f.key.key.clone()));
                build_ts_exists_clause(f, n)
            }
            EntityKeyType::AlarmField => String::new(), // not applicable in entity queries
        };
        sql.push_str(&frag);
    }
    sql
}

/// WHERE clause cho EntityField (direct column comparison).
fn build_entity_field_clause(f: &KeyFilter) -> String {
    let col = match f.key.key.as_str() {
        "name"                         => "name",
        "type"                         => "type",
        "label"                        => "label",
        "createdTime" | "created_time" => "created_time",
        _                              => return String::new(),
    };
    build_predicate_clause(col, &f.predicate)
}

/// EXISTS subquery cho attribute_kv. Key name được bind qua $n để tránh SQL injection.
fn build_attribute_exists_clause(f: &KeyFilter, key_param_n: usize) -> String {
    let value_col = attribute_value_column(&f.value_type);
    let pred = build_value_predicate(value_col, &f.predicate);
    if pred.is_empty() {
        return String::new();
    }
    format!(
        " AND EXISTS (\
            SELECT 1 FROM attribute_kv akv \
            JOIN key_dictionary kd ON kd.key_id = akv.attribute_key \
            WHERE akv.entity_id = id AND kd.key = ${key_param_n}{pred}\
        )"
    )
}

/// EXISTS subquery cho ts_kv_latest. Key name được bind qua $n để tránh SQL injection.
fn build_ts_exists_clause(f: &KeyFilter, key_param_n: usize) -> String {
    let value_col = ts_value_column(&f.value_type);
    let pred = build_value_predicate(value_col, &f.predicate);
    if pred.is_empty() {
        return String::new();
    }
    format!(
        " AND EXISTS (\
            SELECT 1 FROM ts_kv_latest lkv \
            JOIN key_dictionary kd ON kd.key_id = lkv.key \
            WHERE lkv.entity_id = id AND kd.key = ${key_param_n}{pred}\
        )"
    )
}

/// Chọn column của attribute_kv theo value type.
fn attribute_value_column(vt: &EntityKeyValueType) -> &'static str {
    match vt {
        EntityKeyValueType::String   => "str_v",
        EntityKeyValueType::Numeric  => "dbl_v",
        EntityKeyValueType::Boolean  => "bool_v",
        EntityKeyValueType::DateTime => "long_v",
    }
}

/// Chọn column của ts_kv_latest theo value type.
fn ts_value_column(vt: &EntityKeyValueType) -> &'static str {
    match vt {
        EntityKeyValueType::String   => "str_v",
        EntityKeyValueType::Numeric  => "dbl_v",
        EntityKeyValueType::Boolean  => "bool_v",
        EntityKeyValueType::DateTime => "long_v",
    }
}

/// Build predicate fragment cho một column từ KeyFilterPredicate.
/// Trả về `""` nếu không build được (skip).
fn build_predicate_clause(col: &str, predicate: &KeyFilterPredicate) -> String {
    match predicate {
        KeyFilterPredicate::String(p) => build_string_predicate(col, p),
        KeyFilterPredicate::Numeric(p) => {
            let val = p.value.effective();
            let op = match p.operation {
                NumericOperation::Equal          => "=",
                NumericOperation::NotEqual       => "!=",
                NumericOperation::Greater        => ">",
                NumericOperation::Less           => "<",
                NumericOperation::GreaterOrEqual => ">=",
                NumericOperation::LessOrEqual    => "<=",
            };
            format!(" AND {col} {op} {val}")
        }
        KeyFilterPredicate::Boolean(p) => {
            let val = p.value.effective();
            let op = match p.operation {
                BooleanOperation::Equal    => "=",
                BooleanOperation::NotEqual => "!=",
            };
            format!(" AND {col} {op} {val}")
        }
        KeyFilterPredicate::Complex(p) => build_complex_predicate(col, p),
    }
}

/// Build predicate fragment — bản rút gọn không có " AND " prefix, dùng trong EXISTS.
fn build_value_predicate(col: &str, predicate: &KeyFilterPredicate) -> String {
    let full = build_predicate_clause(col, predicate);
    // build_predicate_clause trả về " AND <expr>", cần giữ nguyên để thêm vào subquery
    full
}

fn build_string_predicate(col: &str, p: &vl_core::entities::StringFilterPredicate) -> String {
    use StringOperation::*;
    let val = escape_sql_string(p.value.effective());
    match p.operation {
        Equal    => if p.ignore_case {
            format!(" AND LOWER({col}) = LOWER('{val}')")
        } else {
            format!(" AND {col} = '{val}'")
        },
        NotEqual => if p.ignore_case {
            format!(" AND LOWER({col}) != LOWER('{val}')")
        } else {
            format!(" AND {col} != '{val}'")
        },
        StartsWith  => format!(" AND LOWER({col}) LIKE LOWER('{val}%')"),
        EndsWith    => format!(" AND LOWER({col}) LIKE LOWER('%{val}')"),
        Contains    => format!(" AND LOWER({col}) LIKE LOWER('%{val}%')"),
        NotContains => format!(" AND LOWER({col}) NOT LIKE LOWER('%{val}%')"),
        In => {
            // Giá trị là chuỗi phân cách bởi dấu phẩy: "a,b,c"
            let items: Vec<String> = p.value.effective()
                .split(',')
                .map(|s| format!("'{}'", escape_sql_string(s.trim())))
                .collect();
            if items.is_empty() {
                String::new()
            } else {
                format!(" AND {col} IN ({})", items.join(","))
            }
        }
        NotIn => {
            let items: Vec<String> = p.value.effective()
                .split(',')
                .map(|s| format!("'{}'", escape_sql_string(s.trim())))
                .collect();
            if items.is_empty() {
                String::new()
            } else {
                format!(" AND {col} NOT IN ({})", items.join(","))
            }
        }
    }
}

fn build_complex_predicate(col: &str, p: &vl_core::entities::ComplexFilterPredicate) -> String {
    use vl_core::entities::ComplexOperation;
    let parts: Vec<String> = p.predicates.iter()
        .map(|pred| {
            let frag = build_predicate_clause(col, pred);
            // strip leading " AND " để dùng trong OR/AND join
            frag.trim_start_matches(" AND ").to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return String::new();
    }
    let joined = match p.operation {
        ComplexOperation::And => parts.join(" AND "),
        ComplexOperation::Or  => parts.join(" OR "),
    };
    format!(" AND ({joined})")
}

fn sort_clause(sort_order: &Option<vl_core::entities::EntityDataSortOrder>) -> String {
    match sort_order {
        Some(so) => {
            let col = match so.key.key.as_str() {
                "name"        => "name",
                "type"        => "type",
                "label"       => "label",
                "createdTime" => "created_time",
                _             => "created_time",
            };
            let dir = match so.direction {
                vl_core::entities::SortDirection::Asc  => "ASC",
                vl_core::entities::SortDirection::Desc => "DESC",
            };
            format!(" ORDER BY {col} {dir}")
        }
        None => " ORDER BY created_time DESC".to_string(),
    }
}

fn append_alarm_filter_clauses(
    sql: &mut String,
    params: &mut Vec<SqlParam>,
    status_list: &[String],
    severity_list: &[String],
    type_list: &[String],
    start_ts: Option<i64>,
    end_ts: Option<i64>,
) {
    if !status_list.is_empty() {
        let list: Vec<String> = status_list.iter()
            .map(|s| format!("'{}'", escape_sql_string(s)))
            .collect();
        sql.push_str(&format!(" AND status IN ({})", list.join(",")));
    }
    if !severity_list.is_empty() {
        let list: Vec<String> = severity_list.iter()
            .map(|s| format!("'{}'", escape_sql_string(s)))
            .collect();
        sql.push_str(&format!(" AND severity IN ({})", list.join(",")));
    }
    if !type_list.is_empty() {
        let list: Vec<String> = type_list.iter()
            .map(|s| format!("'{}'", escape_sql_string(s)))
            .collect();
        sql.push_str(&format!(" AND type IN ({})", list.join(",")));
    }
    if let Some(ts) = start_ts {
        let n = params.len() + 1;
        sql.push_str(&format!(" AND created_time >= ${n}"));
        params.push(SqlParam::I64(ts));
    }
    if let Some(ts) = end_ts {
        let n = params.len() + 1;
        sql.push_str(&format!(" AND created_time <= ${n}"));
        params.push(SqlParam::I64(ts));
    }
}

fn alarm_status(acknowledged: bool, cleared: bool) -> String {
    match (acknowledged, cleared) {
        (_, true) => "CLEARED_ACK".to_string(),
        (true, _) => "ACTIVE_ACK".to_string(),
        _         => "ACTIVE_UNACK".to_string(),
    }
}

// ── Dynamic query execution ───────────────────────────────────────────────────

async fn execute_count(
    pool: &PgPool,
    sql: &str,
    params: &[SqlParam],
) -> Result<i64, DaoError> {
    let mut q = sqlx::query_scalar::<_, Option<i64>>(sql);
    for p in params {
        q = bind_param(q, p);
    }
    let count = q.fetch_one(pool).await?.unwrap_or(0);
    Ok(count)
}

struct EntityRow {
    id:          Uuid,
    created_time: i64,
    name:        String,
    entity_type: Option<String>,  // device/asset subtype ("type" column)
    entity_kind: String,          // DEVICE / ASSET / etc.
    label:       Option<String>,
    customer_id: Option<Uuid>,
}

async fn execute_entity_rows(
    pool: &PgPool,
    sql: &str,
    params: &[SqlParam],
    entity_kind_fallback: &str,
) -> Result<Vec<EntityRow>, DaoError> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_query_param(q, p);
    }
    let rows = q.fetch_all(pool).await.map_err(DaoError::Database)?;
    let result = rows.iter().map(|r| EntityRow {
        id:          r.get("id"),
        created_time: r.get("created_time"),
        name:        r.get("name"),
        entity_type: r.try_get("type").ok(),
        entity_kind: r.try_get::<String, _>("entity_kind")
                      .unwrap_or_else(|_| entity_kind_fallback.to_string()),
        label:       r.try_get("label").ok().flatten(),
        customer_id: r.try_get("customer_id").ok().flatten(),
    }).collect();
    Ok(result)
}

struct AlarmRow {
    id:              Uuid,
    created_time:    i64,
    originator_id:   Uuid,
    originator_type: String,
    alarm_type:      String,
    severity:        String,
    acknowledged:    bool,
    cleared:         bool,
    ack_ts:          Option<i64>,
    clear_ts:        Option<i64>,
}

async fn execute_alarm_rows(
    pool: &PgPool,
    sql: &str,
    params: &[SqlParam],
) -> Result<Vec<AlarmRow>, DaoError> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_query_param(q, p);
    }
    let rows = q.fetch_all(pool).await.map_err(DaoError::Database)?;
    let result = rows.iter().map(|r| {
        let originator_type_i: i32 = r.get("originator_type");
        AlarmRow {
            id:              r.get("id"),
            created_time:    r.get("created_time"),
            originator_id:   r.get("originator_id"),
            originator_type: originator_type_to_str(originator_type_i),
            alarm_type:      r.get("type"),
            severity:        r.get("severity"),
            acknowledged:    r.get("acknowledged"),
            cleared:         r.get("cleared"),
            ack_ts:          r.try_get("ack_ts").ok().flatten(),
            clear_ts:        r.try_get("clear_ts").ok().flatten(),
        }
    }).collect();
    Ok(result)
}

async fn execute_uuid_list(
    pool: &PgPool,
    sql: &str,
    params: &[SqlParam],
) -> Result<Vec<Uuid>, DaoError> {
    let mut q = sqlx::query(sql);
    for p in params {
        q = bind_query_param(q, p);
    }
    let rows = q.fetch_all(pool).await.map_err(DaoError::Database)?;
    Ok(rows.iter().map(|r| r.get::<Uuid, _>("id")).collect())
}

fn originator_type_to_str(t: i32) -> String {
    match t {
        0  => "DEVICE".to_string(),
        1  => "ASSET".to_string(),
        2  => "TENANT".to_string(),
        3  => "CUSTOMER".to_string(),
        4  => "USER".to_string(),
        5  => "DASHBOARD".to_string(),
        6  => "RULE_CHAIN".to_string(),
        7  => "RULE_NODE".to_string(),
        8  => "EDGE".to_string(),
        9  => "ENTITY_VIEW".to_string(),
        10 => "WIDGETS_BUNDLE".to_string(),
        11 => "WIDGET_TYPE".to_string(),
        _  => "DEVICE".to_string(),
    }
}

fn build_entity_fields_map(
    row: &EntityRow,
    fields: &[EntityKey],
) -> HashMap<String, TsValue> {
    let mut map = HashMap::new();
    for f in fields {
        if f.key_type != EntityKeyType::EntityField {
            continue;
        }
        let (val, ts) = match f.key.as_str() {
            "name"        => (row.name.clone(), row.created_time),
            "type"        => (row.entity_type.clone().unwrap_or_default(), row.created_time),
            "label"       => (row.label.clone().unwrap_or_default(), row.created_time),
            "createdTime" => (row.created_time.to_string(), row.created_time),
            "customerId"  => (
                row.customer_id.map(|id| id.to_string()).unwrap_or_default(),
                row.created_time,
            ),
            _ => continue,
        };
        map.insert(f.key.clone(), TsValue { ts, value: val });
    }
    map
}

// ── Bind helpers ──────────────────────────────────────────────────────────────

fn bind_param<'q, O>(
    q: sqlx::query::QueryScalar<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
    p: &'q SqlParam,
) -> sqlx::query::QueryScalar<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>
where
    O: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    match p {
        SqlParam::Uuid(v)      => q.bind(*v),
        SqlParam::UuidArray(v) => q.bind(v.clone()),
        SqlParam::Text(v)      => q.bind(v.clone()),
        SqlParam::I64(v)       => q.bind(*v),
        SqlParam::F64(v)       => q.bind(*v),
        SqlParam::Bool(v)      => q.bind(*v),
    }
}

fn bind_query_param<'q>(
    q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    p: &'q SqlParam,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match p {
        SqlParam::Uuid(v)      => q.bind(*v),
        SqlParam::UuidArray(v) => q.bind(v.clone()),
        SqlParam::Text(v)      => q.bind(v.clone()),
        SqlParam::I64(v)       => q.bind(*v),
        SqlParam::F64(v)       => q.bind(*v),
        SqlParam::Bool(v)      => q.bind(*v),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use vl_core::entities::{EntityFilter, QueryEntityId, RelationFilter};
    use uuid::Uuid;

    fn root_entity(entity_type: &str) -> QueryEntityId {
        QueryEntityId { id: Uuid::new_v4(), entity_type: entity_type.to_string() }
    }

    // ── extract_relation_params ───────────────────────────────────────────────

    #[test]
    fn test_extract_device_search_query() {
        let root = root_entity("ASSET");
        let filter = EntityFilter::DeviceSearchQuery {
            root_entity: root.clone(),
            direction: "FROM".to_string(),
            filters: vec![],
            max_level: Some(3),
            fetch_last_level_only: true,
            device_types: vec!["sensor".to_string()],
        };
        let params = extract_relation_params(&filter).unwrap();
        assert_eq!(params.root_id, root.id);
        assert_eq!(params.root_type, "ASSET");
        assert_eq!(params.table, "device");
        assert_eq!(params.target_entity_type, "DEVICE");
        assert_eq!(params.max_level, 3);
        assert!(params.fetch_last_level_only);
        assert_eq!(params.subtypes, vec!["sensor"]);
    }

    #[test]
    fn test_extract_asset_search_query() {
        let filter = EntityFilter::AssetSearchQuery {
            root_entity: root_entity("CUSTOMER"),
            direction: "FROM".to_string(),
            filters: vec![],
            max_level: None,
            fetch_last_level_only: false,
            asset_types: vec!["building".to_string(), "floor".to_string()],
        };
        let params = extract_relation_params(&filter).unwrap();
        assert_eq!(params.table, "asset");
        assert_eq!(params.max_level, 10);  // default
        assert_eq!(params.subtypes.len(), 2);
    }

    #[test]
    fn test_extract_entity_view_search_query() {
        let filter = EntityFilter::EntityViewSearchQuery {
            root_entity: root_entity("DEVICE"),
            direction: "TO".to_string(),
            filters: vec![],
            max_level: Some(5),
            fetch_last_level_only: false,
            entity_view_types: vec![],
        };
        let params = extract_relation_params(&filter).unwrap();
        assert_eq!(params.table, "entity_view");
        assert_eq!(params.direction, "TO");
        assert_eq!(params.target_entity_type, "ENTITY_VIEW");
    }

    #[test]
    fn test_extract_relations_query_all_types() {
        let filter = EntityFilter::RelationsQuery {
            root_entity: root_entity("CUSTOMER"),
            direction: "FROM".to_string(),
            filters: vec![],
            max_level: None,
            fetch_last_level_only: false,
        };
        let params = extract_relation_params(&filter).unwrap();
        assert!(params.target_entity_type.is_empty(), "RelationsQuery should have empty target type");
        assert!(params.table.is_empty(), "RelationsQuery should have empty table");
    }

    #[test]
    fn test_extract_non_relation_filter_returns_none() {
        let filter = EntityFilter::EntityType { entity_type: "DEVICE".to_string() };
        assert!(extract_relation_params(&filter).is_none());

        let filter = EntityFilter::DeviceType {
            device_type: "sensor".to_string(),
            device_name_filter: None,
        };
        assert!(extract_relation_params(&filter).is_none());
    }

    // ── build_relation_where_fragment ────────────────────────────────────────

    #[test]
    fn test_relation_where_empty_filters() {
        let frag = build_relation_where_fragment(&[], "r.");
        assert!(frag.is_empty());
    }

    #[test]
    fn test_relation_where_type_group_only() {
        let filters = vec![RelationFilter {
            relation_type_group: Some("COMMON".to_string()),
            relation_type: None,
            entity_types: vec![],
        }];
        let frag = build_relation_where_fragment(&filters, "r.");
        assert!(frag.contains("relation_type_group"));
        assert!(frag.contains("COMMON"));
        assert!(frag.starts_with(" AND "));
    }

    #[test]
    fn test_relation_where_type_and_group() {
        let filters = vec![RelationFilter {
            relation_type_group: Some("COMMON".to_string()),
            relation_type: Some("Contains".to_string()),
            entity_types: vec![],
        }];
        let frag = build_relation_where_fragment(&filters, "r.");
        assert!(frag.contains("relation_type_group"));
        assert!(frag.contains("relation_type"));
        assert!(frag.contains("Contains"));
    }

    #[test]
    fn test_relation_where_multiple_filters_or() {
        let filters = vec![
            RelationFilter {
                relation_type_group: None,
                relation_type: Some("Contains".to_string()),
                entity_types: vec![],
            },
            RelationFilter {
                relation_type_group: None,
                relation_type: Some("Manages".to_string()),
                entity_types: vec![],
            },
        ];
        let frag = build_relation_where_fragment(&filters, "r.");
        assert!(frag.contains("Contains"));
        assert!(frag.contains("Manages"));
        assert!(frag.contains(" OR "));
    }

    #[test]
    fn test_relation_where_sql_injection_escaped() {
        let malicious = "abc'def";
        let filters = vec![RelationFilter {
            relation_type_group: None,
            relation_type: Some(malicious.to_string()),
            entity_types: vec![],
        }];
        let frag = build_relation_where_fragment(&filters, "r.");
        // The single quote must be doubled so SQL treats the value as a single string literal.
        assert!(frag.contains("''"), "single quote must be doubled by escape_sql_string");
        // The original unescaped quote must NOT appear as a lone single quote adjacent to non-quote
        assert!(!frag.contains("abc'def"), "original unescaped quote must not appear");
        assert!(frag.contains("abc''def"), "escaped value must use doubled quotes");
    }

    // ── filter_to_table_and_where ─────────────────────────────────────────────

    #[test]
    fn test_filter_device_type() {
        let filter = EntityFilter::DeviceType {
            device_type: "sensor".to_string(),
            device_name_filter: Some("temp".to_string()),
        };
        let (table, type_col, extra) = filter_to_table_and_where(&filter).unwrap();
        assert_eq!(table, "device");
        assert!(type_col.is_none());
        assert!(extra.contains("sensor"));
        assert!(extra.contains("temp"));
    }

    #[test]
    fn test_filter_entity_list_empty() {
        let filter = EntityFilter::EntityList {
            entity_type: "DEVICE".to_string(),
            entity_ids: vec![],
        };
        let (_, _, extra) = filter_to_table_and_where(&filter).unwrap();
        assert!(extra.contains("FALSE"));
    }

    #[test]
    fn test_filter_sub_customers() {
        let cid = Uuid::new_v4();
        let filter = EntityFilter::SubCustomers { root_customer_id: cid };
        let (table, _, extra) = filter_to_table_and_where(&filter).unwrap();
        assert_eq!(table, "customer");
        assert!(extra.contains(&cid.to_string()));
    }

    #[test]
    fn test_filter_api_usage_state() {
        let filter = EntityFilter::ApiUsageState { entity_type: "TENANT".to_string() };
        let (table, _, _) = filter_to_table_and_where(&filter).unwrap();
        assert_eq!(table, "tenant");
    }

    // ── entity_type_to_table ──────────────────────────────────────────────────

    #[test]
    fn test_entity_type_to_table_coverage() {
        assert_eq!(entity_type_to_table("DEVICE").unwrap(),      "device");
        assert_eq!(entity_type_to_table("ASSET").unwrap(),       "asset");
        assert_eq!(entity_type_to_table("CUSTOMER").unwrap(),    "customer");
        assert_eq!(entity_type_to_table("TENANT").unwrap(),      "tenant");
        assert_eq!(entity_type_to_table("ENTITY_VIEW").unwrap(), "entity_view");
        assert_eq!(entity_type_to_table("EDGE").unwrap(),        "edge");
        assert_eq!(entity_type_to_table("DASHBOARD").unwrap(),   "dashboard");
        assert_eq!(entity_type_to_table("USER").unwrap(),        "tb_user");
        // Case insensitive
        assert_eq!(entity_type_to_table("device").unwrap(),      "device");
    }

    #[test]
    fn test_entity_type_to_table_rejects_unknown() {
        let result = entity_type_to_table("UNKNOWN_TYPE");
        assert!(result.is_err(), "Unknown entity type should return Err");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("UNKNOWN_TYPE"), "Error should mention the bad type");
    }

    #[test]
    fn test_filter_unknown_entity_type_returns_err() {
        let filter = EntityFilter::EntityType { entity_type: "BOGUS".to_string() };
        assert!(filter_to_table_and_where(&filter).is_err());
    }

    // ── append_subtype_filter ────────────────────────────────────────────────

    #[test]
    fn test_append_subtype_filter_empty() {
        let mut sql = "SELECT * FROM device WHERE tenant_id = $1".to_string();
        append_subtype_filter(&mut sql, &[]);
        assert!(!sql.contains("IN"));
    }

    #[test]
    fn test_append_subtype_filter_values() {
        let mut sql = "SELECT * FROM device WHERE tenant_id = $1".to_string();
        append_subtype_filter(&mut sql, &["sensor".to_string(), "actuator".to_string()]);
        assert!(sql.contains("type IN"));
        assert!(sql.contains("sensor"));
        assert!(sql.contains("actuator"));
    }

    // ── build_key_filter_clauses ──────────────────────────────────────────────

    fn make_string_filter(
        key_type: EntityKeyType,
        key: &str,
        op: StringOperation,
        val: &str,
    ) -> KeyFilter {
        use vl_core::entities::{
            FilterPredicateValue, StringFilterPredicate, EntityKey,
        };
        KeyFilter {
            key: EntityKey { key_type, key: key.to_string() },
            value_type: EntityKeyValueType::String,
            predicate: KeyFilterPredicate::String(StringFilterPredicate {
                operation: op,
                value: FilterPredicateValue { default_value: val.to_string(), user_value: None },
                ignore_case: false,
            }),
        }
    }

    #[test]
    fn test_entity_field_equals_clause() {
        let f = make_string_filter(EntityKeyType::EntityField, "name", StringOperation::Equal, "temp_sensor");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("name = 'temp_sensor'"), "got: {sql}");
    }

    #[test]
    fn test_entity_field_contains_clause() {
        let f = make_string_filter(EntityKeyType::EntityField, "label", StringOperation::Contains, "floor");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("LIKE"), "got: {sql}");
        assert!(sql.contains("floor"), "got: {sql}");
    }

    #[test]
    fn test_entity_field_in_clause() {
        let f = make_string_filter(EntityKeyType::EntityField, "type", StringOperation::In, "sensor,actuator");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("type IN"), "got: {sql}");
        assert!(sql.contains("sensor"), "got: {sql}");
        assert!(sql.contains("actuator"), "got: {sql}");
    }

    #[test]
    fn test_entity_field_not_in_clause() {
        let f = make_string_filter(EntityKeyType::EntityField, "type", StringOperation::NotIn, "archived");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("NOT IN"), "got: {sql}");
    }

    #[test]
    fn test_attribute_key_filter_generates_exists_subquery() {
        let f = make_string_filter(EntityKeyType::Attribute, "firmware_version", StringOperation::Equal, "v2");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("EXISTS"), "expected EXISTS subquery, got: {sql}");
        assert!(sql.contains("attribute_kv"), "expected attribute_kv join, got: {sql}");
        // key name is in params, not inlined in SQL
        assert!(sql.contains("$1"), "expected $1 placeholder, got: {sql}");
        assert!(matches!(&params[0], SqlParam::Text(k) if k == "firmware_version"), "expected key in params");
        assert!(sql.contains("str_v"), "expected str_v column for string type, got: {sql}");
    }

    #[test]
    fn test_ts_key_filter_generates_exists_subquery() {
        let f = make_string_filter(EntityKeyType::TimeSeries, "temperature", StringOperation::Equal, "22.5");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("EXISTS"), "expected EXISTS subquery, got: {sql}");
        assert!(sql.contains("ts_kv_latest"), "expected ts_kv_latest join, got: {sql}");
        // key name is in params, not inlined in SQL
        assert!(sql.contains("$1"), "expected $1 placeholder, got: {sql}");
        assert!(matches!(&params[0], SqlParam::Text(k) if k == "temperature"), "expected key in params");
    }

    #[test]
    fn test_sql_injection_in_attribute_key() {
        // Key name được bind qua $N — injection string không bao giờ xuất hiện trong SQL
        let f = make_string_filter(EntityKeyType::Attribute, "key'; DROP TABLE device;--", StringOperation::Equal, "x");
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(!sql.contains("'; DROP"), "injection must not appear in SQL, got: {sql}");
        // key ends up in params as a plain string — bound safely by sqlx
        assert!(matches!(&params[0], SqlParam::Text(k) if k.contains("DROP")), "key should be in params");
    }

    #[test]
    fn test_complex_filter_and() {
        use vl_core::entities::{
            ComplexFilterPredicate, ComplexOperation, FilterPredicateValue,
            StringFilterPredicate, EntityKey,
        };
        let make_pred = |val: &str| KeyFilterPredicate::String(StringFilterPredicate {
            operation: StringOperation::Equal,
            value: FilterPredicateValue { default_value: val.to_string(), user_value: None },
            ignore_case: false,
        });
        let f = KeyFilter {
            key: EntityKey { key_type: EntityKeyType::EntityField, key: "name".to_string() },
            value_type: EntityKeyValueType::String,
            predicate: KeyFilterPredicate::Complex(ComplexFilterPredicate {
                operation: ComplexOperation::And,
                predicates: vec![make_pred("alpha"), make_pred("beta")],
            }),
        };
        let mut params = vec![];
        let sql = build_key_filter_clauses(&[f], 1, &mut params, "device");
        assert!(sql.contains("AND"), "got: {sql}");
        assert!(sql.contains("alpha"), "got: {sql}");
        assert!(sql.contains("beta"), "got: {sql}");
    }
}
