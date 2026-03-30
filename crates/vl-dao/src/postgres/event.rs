use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Event, EventType, EventFilter};
use crate::{DaoError, PageData, PageLink};

pub struct EventDao {
    pool: PgPool,
}

impl EventDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, entity_id, entity_type,
                   event_type, event_uid, body
            FROM event WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Event {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            entity_id: r.entity_id,
            entity_type: r.entity_type,
            event_type: EventType::from_str(&r.event_type).unwrap_or(EventType::LcEvent),
            event_uid: r.event_uid,
            body: r.body,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_entity(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        entity_type: &str,
        filter: &EventFilter,
        page_link: &PageLink,
    ) -> Result<PageData<Event>, DaoError> {
        let event_type_str = filter.event_type.map(|t| t.as_str().to_string());
        let start_ts = filter.start_ts.unwrap_or(0);
        let end_ts = filter.end_ts.unwrap_or(i64::MAX);

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM event
               WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
               AND ($4::text IS NULL OR event_type = $4)
               AND created_time >= $5 AND created_time <= $6"#,
            tenant_id,
            entity_id,
            entity_type,
            event_type_str,
            start_ts,
            end_ts,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, entity_id, entity_type,
                   event_type, event_uid, body
            FROM event
            WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
            AND ($4::text IS NULL OR event_type = $4)
            AND created_time >= $5 AND created_time <= $6
            ORDER BY created_time DESC
            LIMIT $7 OFFSET $8
            "#,
            tenant_id,
            entity_id,
            entity_type,
            event_type_str,
            start_ts,
            end_ts,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Event {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            entity_id: r.entity_id,
            entity_type: r.entity_type,
            event_type: EventType::from_str(&r.event_type).unwrap_or(EventType::LcEvent),
            event_uid: r.event_uid,
            body: r.body,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, event: &Event) -> Result<Event, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO event (
                id, created_time, tenant_id, entity_id, entity_type,
                event_type, event_uid, body
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (tenant_id, entity_id, entity_type, event_uid) DO UPDATE SET
                body = EXCLUDED.body
            "#,
            event.id,
            event.created_time,
            event.tenant_id,
            event.entity_id,
            event.entity_type,
            event.event_type.as_str(),
            event.event_uid,
            event.body,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(event.id).await?.ok_or(DaoError::NotFound)
    }

    /// Delete events for entity within time range
    #[instrument(skip(self))]
    pub async fn delete_by_entity(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        entity_type: &str,
        filter: &EventFilter,
    ) -> Result<i64, DaoError> {
        let event_type_str = filter.event_type.map(|t| t.as_str().to_string());
        let start_ts = filter.start_ts.unwrap_or(0);
        let end_ts = filter.end_ts.unwrap_or(i64::MAX);

        let result = sqlx::query!(
            r#"
            DELETE FROM event
            WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
            AND ($4::text IS NULL OR event_type = $4)
            AND created_time >= $5 AND created_time <= $6
            "#,
            tenant_id,
            entity_id,
            entity_type,
            event_type_str,
            start_ts,
            end_ts,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Get event types for entity
    #[instrument(skip(self))]
    pub async fn get_event_types(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        entity_type: &str,
    ) -> Result<Vec<EventType>, DaoError> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT event_type FROM event
            WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
            "#,
            tenant_id,
            entity_id,
            entity_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter()
            .filter_map(|s: String| EventType::from_str(&s))
            .collect())
    }

    /// Cleanup old events (retention policy)
    #[instrument(skip(self))]
    pub async fn cleanup_old_events(&self, tenant_id: Uuid, retention_ts: i64) -> Result<i64, DaoError> {
        let result = sqlx::query!(
            "DELETE FROM event WHERE tenant_id = $1 AND created_time < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Lấy debug events của một rule node — dùng cho rule engine debug tracing UI.
    /// Trả về tối đa `limit` events gần nhất, sắp xếp mới nhất trước.
    #[instrument(skip(self))]
    pub async fn find_debug_events(
        &self,
        rule_node_id: Uuid,
        limit:        i64,
    ) -> Result<Vec<serde_json::Value>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT body FROM event
            WHERE entity_id = $1 AND entity_type = 'RULE_NODE'
              AND event_type = 'DEBUG_RULE_NODE'
            ORDER BY created_time DESC
            LIMIT $2
            "#,
            rule_node_id,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter()
            .map(|r| r.body)
            .collect())
    }

    /// Xóa debug events cũ của một rule node (dọn dẹp sau khi tắt debug)
    #[instrument(skip(self))]
    pub async fn delete_debug_events(&self, rule_node_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM event
             WHERE entity_id = $1 AND entity_type = 'RULE_NODE'
               AND event_type = 'DEBUG_RULE_NODE'",
            rule_node_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Partitioned event table methods ──────────────────────────────────────

    /// Save a rule node debug event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_rule_node_debug_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: Option<&str>,
        e_type: Option<&str>,
        e_entity_id: Option<Uuid>,
        e_entity_type: Option<&str>,
        e_msg_id: Option<Uuid>,
        e_msg_type: Option<&str>,
        e_data_type: Option<&str>,
        e_relation_type: Option<&str>,
        e_data: Option<&str>,
        e_metadata: Option<&str>,
        e_error: Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO rule_node_debug_event (
                tenant_id, ts, entity_id, service_id,
                e_type, e_entity_id, e_entity_type, e_msg_id, e_msg_type,
                e_data_type, e_relation_type, e_data, e_metadata, e_error
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            e_type,
            e_entity_id,
            e_entity_type,
            e_msg_id,
            e_msg_type,
            e_data_type,
            e_relation_type,
            e_data,
            e_metadata,
            e_error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save a rule chain debug event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_rule_chain_debug_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: &str,
        e_message: Option<&str>,
        e_error: Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO rule_chain_debug_event (
                tenant_id, ts, entity_id, service_id, e_message, e_error
            ) VALUES ($1,$2,$3,$4,$5,$6)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            e_message,
            e_error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save a stats event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_stats_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: &str,
        messages_processed: i64,
        errors_occurred: i64,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO stats_event (
                tenant_id, ts, entity_id, service_id,
                e_messages_processed, e_errors_occurred
            ) VALUES ($1,$2,$3,$4,$5,$6)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            messages_processed,
            errors_occurred,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save a lifecycle event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_lc_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: &str,
        e_type: &str,
        e_success: bool,
        e_error: Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO lc_event (
                tenant_id, ts, entity_id, service_id, e_type, e_success, e_error
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            e_type,
            e_success,
            e_error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save an error event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_error_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: &str,
        e_method: &str,
        e_error: Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO error_event (
                tenant_id, ts, entity_id, service_id, e_method, e_error
            ) VALUES ($1,$2,$3,$4,$5,$6)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            e_method,
            e_error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Save a calculated field debug event to the partitioned table
    #[instrument(skip(self))]
    pub async fn save_cf_debug_event(
        &self,
        tenant_id: Uuid,
        ts: i64,
        entity_id: Uuid,
        service_id: Option<&str>,
        cf_id: Uuid,
        e_entity_id: Option<Uuid>,
        e_entity_type: Option<&str>,
        e_msg_id: Option<Uuid>,
        e_msg_type: Option<&str>,
        e_args: Option<&str>,
        e_result: Option<&str>,
        e_error: Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO cf_debug_event (
                tenant_id, ts, entity_id, service_id, cf_id,
                e_entity_id, e_entity_type, e_msg_id, e_msg_type,
                e_args, e_result, e_error
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            "#,
            tenant_id,
            ts,
            entity_id,
            service_id,
            cf_id,
            e_entity_id,
            e_entity_type,
            e_msg_id,
            e_msg_type,
            e_args,
            e_result,
            e_error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Query rule node debug events by entity (with pagination by ts range)
    #[instrument(skip(self))]
    pub async fn find_rule_node_debug_events(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, ts, service_id, e_type, e_entity_id, e_entity_type,
                   e_msg_id, e_msg_type, e_data_type, e_relation_type,
                   e_data, e_metadata, e_error
            FROM rule_node_debug_event
            WHERE tenant_id = $1 AND entity_id = $2
              AND ts >= $3 AND ts < $4
            ORDER BY ts DESC
            LIMIT $5
            "#,
            tenant_id,
            entity_id,
            start_ts,
            end_ts,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "ts": r.ts,
                    "serviceId": r.service_id,
                    "type": r.e_type,
                    "entityId": r.e_entity_id,
                    "entityType": r.e_entity_type,
                    "msgId": r.e_msg_id,
                    "msgType": r.e_msg_type,
                    "dataType": r.e_data_type,
                    "relationType": r.e_relation_type,
                    "data": r.e_data,
                    "metadata": r.e_metadata,
                    "error": r.e_error,
                })
            })
            .collect())
    }

    /// Query stats events by entity (with pagination by ts range)
    #[instrument(skip(self))]
    pub async fn find_stats_events(
        &self,
        tenant_id: Uuid,
        entity_id: Uuid,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, ts, service_id, e_messages_processed, e_errors_occurred
            FROM stats_event
            WHERE tenant_id = $1 AND entity_id = $2
              AND ts >= $3 AND ts < $4
            ORDER BY ts DESC
            LIMIT $5
            "#,
            tenant_id,
            entity_id,
            start_ts,
            end_ts,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "ts": r.ts,
                    "serviceId": r.service_id,
                    "messagesProcessed": r.e_messages_processed,
                    "errorsOccurred": r.e_errors_occurred,
                })
            })
            .collect())
    }

    /// Cleanup old events from all partitioned tables for a tenant.
    /// Deletes rows with ts < retention_ts. Returns total rows deleted.
    #[instrument(skip(self))]
    pub async fn cleanup_partitioned_events(
        &self,
        tenant_id: Uuid,
        retention_ts: i64,
    ) -> Result<i64, DaoError> {
        let mut total: i64 = 0;

        let r1 = sqlx::query!(
            "DELETE FROM rule_node_debug_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r1.rows_affected() as i64;

        let r2 = sqlx::query!(
            "DELETE FROM rule_chain_debug_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r2.rows_affected() as i64;

        let r3 = sqlx::query!(
            "DELETE FROM stats_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r3.rows_affected() as i64;

        let r4 = sqlx::query!(
            "DELETE FROM lc_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r4.rows_affected() as i64;

        let r5 = sqlx::query!(
            "DELETE FROM error_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r5.rows_affected() as i64;

        let r6 = sqlx::query!(
            "DELETE FROM cf_debug_event WHERE tenant_id = $1 AND ts < $2",
            tenant_id,
            retention_ts,
        )
        .execute(&self.pool)
        .await?;
        total += r6.rows_affected() as i64;

        Ok(total)
    }
}
