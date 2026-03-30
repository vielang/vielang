use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Alarm, AlarmComment, AlarmSeverity, EntityType};
use crate::{DaoError, PageData, PageLink};

pub struct AlarmDao {
    pool: PgPool,
}

impl AlarmDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Alarm>, DaoError> {
        // NOT NULL columns: id, created_time, tenant_id, type, originator_id,
        // originator_type, severity, acknowledged, cleared, start_ts, end_ts, assign_ts,
        // propagate, propagate_to_owner, propagate_to_tenant
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, type as alarm_type,
                   originator_id, originator_type, severity,
                   acknowledged, cleared, assignee_id,
                   start_ts, end_ts, ack_ts, clear_ts, assign_ts,
                   propagate, propagate_to_owner, propagate_to_tenant,
                   propagate_relation_types, details
            FROM alarm WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Alarm {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,       // NOT NULL
            customer_id: r.customer_id,   // nullable
            alarm_type: r.alarm_type,     // NOT NULL (aliased from type)
            originator_id: r.originator_id, // NOT NULL
            originator_type: EntityType::try_from(r.originator_type) // NOT NULL i32
                .unwrap_or(EntityType::Device),
            severity: parse_severity(&r.severity), // NOT NULL
            acknowledged: r.acknowledged,           // NOT NULL bool
            cleared: r.cleared,                     // NOT NULL bool
            assignee_id: r.assignee_id,             // nullable
            start_ts: r.start_ts,                   // NOT NULL
            end_ts: r.end_ts,                       // NOT NULL
            ack_ts: r.ack_ts,                       // nullable
            clear_ts: r.clear_ts,                   // nullable
            assign_ts: r.assign_ts,                 // NOT NULL
            propagate: r.propagate,                 // NOT NULL
            propagate_to_owner: r.propagate_to_owner,   // NOT NULL
            propagate_to_tenant: r.propagate_to_tenant, // NOT NULL
            propagate_relation_types: r.propagate_relation_types, // nullable
            details: r.details.and_then(|s| serde_json::from_str(&s).ok()),
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_originator(
        &self,
        tenant_id: Uuid,
        originator_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Alarm>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alarm WHERE tenant_id = $1 AND originator_id = $2",
            tenant_id, originator_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, type as alarm_type,
                   originator_id, originator_type, severity,
                   acknowledged, cleared, assignee_id,
                   start_ts, end_ts, ack_ts, clear_ts, assign_ts,
                   propagate, propagate_to_owner, propagate_to_tenant,
                   propagate_relation_types, details
            FROM alarm
            WHERE tenant_id = $1 AND originator_id = $2
            ORDER BY start_ts DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id, originator_id,
            page_link.page_size, page_link.offset()
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Alarm {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            alarm_type: r.alarm_type,
            originator_id: r.originator_id,
            originator_type: EntityType::try_from(r.originator_type)
                .unwrap_or(EntityType::Device),
            severity: parse_severity(&r.severity),
            acknowledged: r.acknowledged,
            cleared: r.cleared,
            assignee_id: r.assignee_id,
            start_ts: r.start_ts,
            end_ts: r.end_ts,
            ack_ts: r.ack_ts,
            clear_ts: r.clear_ts,
            assign_ts: r.assign_ts,
            propagate: r.propagate,
            propagate_to_owner: r.propagate_to_owner,
            propagate_to_tenant: r.propagate_to_tenant,
            propagate_relation_types: r.propagate_relation_types,
            details: r.details.and_then(|s| serde_json::from_str(&s).ok()),
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, alarm: &Alarm) -> Result<Alarm, DaoError> {
        let severity = format!("{:?}", alarm.severity).to_uppercase();
        let originator_type: i32 = alarm.originator_type.clone().into();
        let details = alarm.details.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO alarm (
                id, created_time, tenant_id, customer_id, type,
                originator_id, originator_type, severity,
                acknowledged, cleared, assignee_id,
                start_ts, end_ts, ack_ts, clear_ts, assign_ts,
                propagate, propagate_to_owner, propagate_to_tenant,
                propagate_relation_types, details
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
            ON CONFLICT (id) DO UPDATE SET
                severity                 = EXCLUDED.severity,
                acknowledged             = EXCLUDED.acknowledged,
                cleared                  = EXCLUDED.cleared,
                assignee_id              = EXCLUDED.assignee_id,
                end_ts                   = EXCLUDED.end_ts,
                ack_ts                   = EXCLUDED.ack_ts,
                clear_ts                 = EXCLUDED.clear_ts,
                assign_ts                = EXCLUDED.assign_ts,
                propagate                = EXCLUDED.propagate,
                propagate_to_owner       = EXCLUDED.propagate_to_owner,
                propagate_to_tenant      = EXCLUDED.propagate_to_tenant,
                propagate_relation_types = EXCLUDED.propagate_relation_types,
                details                  = EXCLUDED.details
            "#,
            alarm.id, alarm.created_time, alarm.tenant_id, alarm.customer_id,
            alarm.alarm_type, alarm.originator_id, originator_type, severity,
            alarm.acknowledged, alarm.cleared, alarm.assignee_id,
            alarm.start_ts, alarm.end_ts, alarm.ack_ts, alarm.clear_ts, alarm.assign_ts,
            alarm.propagate, alarm.propagate_to_owner, alarm.propagate_to_tenant,
            alarm.propagate_relation_types, details,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        let saved = self.find_by_id(alarm.id).await?.ok_or(DaoError::NotFound)?;
        metrics::counter!("vielang_alarms_created_total").increment(1);
        Ok(saved)
    }

    #[instrument(skip(self))]
    pub async fn acknowledge(&self, id: Uuid, ack_ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE alarm SET acknowledged = TRUE, ack_ts = $1 WHERE id = $2",
            ack_ts, id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn clear(&self, id: Uuid, clear_ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE alarm SET cleared = TRUE, clear_ts = $1 WHERE id = $2",
            clear_ts, id
        )
        .execute(&self.pool)
        .await?;
        metrics::counter!("vielang_alarms_cleared_total").increment(1);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM alarm WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// POST /api/alarm/{alarmId}/assign/{userId} — gán alarm cho user
    #[instrument(skip(self))]
    pub async fn assign_to_user(&self, alarm_id: Uuid, user_id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE alarm SET assignee_id = $1, assign_ts = $2 WHERE id = $3",
            user_id, ts, alarm_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// DELETE /api/alarm/{alarmId}/assign — bỏ gán alarm
    #[instrument(skip(self))]
    pub async fn unassign(&self, alarm_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE alarm SET assignee_id = NULL, assign_ts = 0 WHERE id = $1",
            alarm_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// GET /api/alarms — list all alarms for a tenant (Java AlarmController.getAlarms)
    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Alarm>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alarm WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, type as alarm_type,
                   originator_id, originator_type, severity,
                   acknowledged, cleared, assignee_id,
                   start_ts, end_ts, ack_ts, clear_ts, assign_ts,
                   propagate, propagate_to_owner, propagate_to_tenant,
                   propagate_relation_types, details
            FROM alarm
            WHERE tenant_id = $1
            ORDER BY start_ts DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            page_link.page_size,
            page_link.offset()
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Alarm {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            alarm_type: r.alarm_type,
            originator_id: r.originator_id,
            originator_type: EntityType::try_from(r.originator_type)
                .unwrap_or(EntityType::Device),
            severity: parse_severity(&r.severity),
            acknowledged: r.acknowledged,
            cleared: r.cleared,
            assignee_id: r.assignee_id,
            start_ts: r.start_ts,
            end_ts: r.end_ts,
            ack_ts: r.ack_ts,
            clear_ts: r.clear_ts,
            assign_ts: r.assign_ts,
            propagate: r.propagate,
            propagate_to_owner: r.propagate_to_owner,
            propagate_to_tenant: r.propagate_to_tenant,
            propagate_relation_types: r.propagate_relation_types,
            details: r.details.and_then(|s| serde_json::from_str(&s).ok()),
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Resolve originator names for a batch of alarm originators.
    /// Looks up device, asset, and customer tables by ID.
    /// Returns a map of id → name for use in AlarmResponse.originatorName.
    pub async fn resolve_originator_names(
        &self,
        originator_ids: &[Uuid],
    ) -> std::collections::HashMap<Uuid, String> {
        if originator_ids.is_empty() {
            return std::collections::HashMap::new();
        }
        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, name FROM device  WHERE id = ANY($1)
            UNION ALL
            SELECT id, name FROM asset   WHERE id = ANY($1)
            UNION ALL
            SELECT id, title FROM customer WHERE id = ANY($1)
            "#,
        )
        .bind(originator_ids)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();
        rows.into_iter().collect()
    }

    /// GET /api/alarm/types — danh sách alarm types của tenant
    #[instrument(skip(self))]
    pub async fn find_types_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query_scalar!(
            "SELECT DISTINCT type FROM alarm WHERE tenant_id = $1 ORDER BY type",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Find highest alarm severity for an entity (active/uncleared alarms only).
    /// Java: AlarmService.findHighestAlarmSeverity()
    #[instrument(skip(self))]
    pub async fn find_highest_severity(
        &self,
        originator_id: Uuid,
    ) -> Result<Option<String>, DaoError> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT severity FROM alarm
               WHERE originator_id = $1 AND clear_ts IS NULL
               ORDER BY CASE severity
                   WHEN 'CRITICAL' THEN 1
                   WHEN 'MAJOR' THEN 2
                   WHEN 'MINOR' THEN 3
                   WHEN 'WARNING' THEN 4
                   WHEN 'INDETERMINATE' THEN 5
                   ELSE 6 END
               LIMIT 1"#,
        )
        .bind(originator_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(s,)| s))
    }

    /// GET /api/alarms/v2 — advanced filtered alarm query (Flutter PE AlarmQueryV2)
    #[instrument(skip(self))]
    pub async fn find_with_filters(
        &self,
        tenant_id:   Uuid,
        start_time:  Option<i64>,
        end_time:    Option<i64>,
        severity:    Option<&str>,
        status:      Option<&str>,
        assignee_id: Option<Uuid>,
        alarm_type:  Option<&str>,
        text_search: Option<&str>,
        limit:       i64,
        offset:      i64,
    ) -> Result<Vec<Alarm>, DaoError> {
        // Note: alarm table has no 'status' column — status is computed from acknowledged+cleared.
        // The status filter is applied post-query via the AlarmStatus computed property if needed.
        // For now, status parameter is accepted but ignored in SQL (consistent with TB Java v1 query).
        let _ = status;
        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, customer_id, type as alarm_type,
                      originator_id, originator_type, severity,
                      acknowledged, cleared, assignee_id,
                      start_ts, end_ts, ack_ts, clear_ts, assign_ts,
                      propagate, propagate_to_owner, propagate_to_tenant,
                      propagate_relation_types, details
               FROM alarm
               WHERE tenant_id = $1
                 AND ($2::bigint IS NULL OR start_ts >= $2)
                 AND ($3::bigint IS NULL OR start_ts <= $3)
                 AND ($4::text   IS NULL OR severity  = $4)
                 AND ($5::uuid   IS NULL OR assignee_id = $5)
                 AND ($6::text   IS NULL OR type      = $6)
                 AND ($7::text   IS NULL OR type ILIKE '%' || $7 || '%')
               ORDER BY created_time DESC
               LIMIT $8 OFFSET $9"#,
            tenant_id,
            start_time,
            end_time,
            severity,
            assignee_id,
            alarm_type,
            text_search,
            limit,
            offset,
        ).fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(|r| Alarm {
            id:                       r.id,
            created_time:             r.created_time,
            tenant_id:                r.tenant_id,
            customer_id:              r.customer_id,
            alarm_type:               r.alarm_type,
            originator_id:            r.originator_id,
            originator_type:          EntityType::try_from(r.originator_type)
                .unwrap_or(EntityType::Device),
            severity:                 parse_severity(&r.severity),
            acknowledged:             r.acknowledged,
            cleared:                  r.cleared,
            assignee_id:              r.assignee_id,
            start_ts:                 r.start_ts,
            end_ts:                   r.end_ts,
            ack_ts:                   r.ack_ts,
            clear_ts:                 r.clear_ts,
            assign_ts:                r.assign_ts,
            propagate:                r.propagate,
            propagate_to_owner:       r.propagate_to_owner,
            propagate_to_tenant:      r.propagate_to_tenant,
            propagate_relation_types: r.propagate_relation_types,
            details:                  r.details.and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// COUNT for /api/alarms/v2 — advanced filtered alarm count
    #[instrument(skip(self))]
    pub async fn count_with_filters(
        &self,
        tenant_id:   Uuid,
        start_time:  Option<i64>,
        end_time:    Option<i64>,
        severity:    Option<&str>,
        status:      Option<&str>,
        assignee_id: Option<Uuid>,
        alarm_type:  Option<&str>,
        text_search: Option<&str>,
    ) -> Result<i64, DaoError> {
        // Note: alarm table has no 'status' column — status is computed from acknowledged+cleared.
        let _ = status;
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM alarm
               WHERE tenant_id = $1
                 AND ($2::bigint IS NULL OR start_ts >= $2)
                 AND ($3::bigint IS NULL OR start_ts <= $3)
                 AND ($4::text   IS NULL OR severity  = $4)
                 AND ($5::uuid   IS NULL OR assignee_id = $5)
                 AND ($6::text   IS NULL OR type      = $6)
                 AND ($7::text   IS NULL OR type ILIKE '%' || $7 || '%')"#,
            tenant_id, start_time, end_time, severity, assignee_id, alarm_type, text_search,
        ).fetch_one(&self.pool).await?.unwrap_or(0);
        Ok(count)
    }

    /// Find all TENANT_ADMIN user IDs for a given tenant (for notification targeting)
    #[instrument(skip(self))]
    pub async fn find_tenant_admin_ids(&self, tenant_id: Uuid) -> Result<Vec<Uuid>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id FROM tb_user WHERE tenant_id = $1 AND authority = 'TENANT_ADMIN'",
            tenant_id
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}

// ── AlarmCommentDao ───────────────────────────────────────────────────────────

pub struct AlarmCommentDao {
    pool: PgPool,
}

impl AlarmCommentDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// POST /api/alarm/{alarmId}/comment — tạo comment
    #[instrument(skip(self))]
    pub async fn save(&self, comment: &AlarmComment) -> Result<AlarmComment, DaoError> {
        let comment_json = comment.comment.to_string();
        sqlx::query!(
            r#"
            INSERT INTO alarm_comment (id, created_time, alarm_id, user_id, type, comment)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET comment = EXCLUDED.comment
            "#,
            comment.id,
            comment.created_time,
            comment.alarm_id,
            comment.user_id,
            comment.comment_type,
            comment_json,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(comment.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AlarmComment>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, created_time, alarm_id, user_id, type as comment_type, comment FROM alarm_comment WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| AlarmComment {
            id: r.id,
            created_time: r.created_time,
            alarm_id: r.alarm_id,
            user_id: r.user_id,
            comment_type: r.comment_type,
            comment: serde_json::from_str(&r.comment).unwrap_or(serde_json::Value::Null),
        }))
    }

    /// GET /api/alarm/{alarmId}/comments — list comments của alarm
    #[instrument(skip(self))]
    pub async fn find_by_alarm(
        &self,
        alarm_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<AlarmComment>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alarm_comment WHERE alarm_id = $1",
            alarm_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, alarm_id, user_id, type as comment_type, comment
            FROM alarm_comment
            WHERE alarm_id = $1
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            alarm_id,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| AlarmComment {
            id: r.id,
            created_time: r.created_time,
            alarm_id: r.alarm_id,
            user_id: r.user_id,
            comment_type: r.comment_type,
            comment: serde_json::from_str(&r.comment).unwrap_or(serde_json::Value::Null),
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// DELETE /api/alarm/{alarmId}/comment/{commentId}
    #[instrument(skip(self))]
    pub async fn delete(&self, comment_id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM alarm_comment WHERE id = $1", comment_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

fn parse_severity(s: &str) -> AlarmSeverity {
    match s {
        "CRITICAL"      => AlarmSeverity::Critical,
        "MAJOR"         => AlarmSeverity::Major,
        "MINOR"         => AlarmSeverity::Minor,
        "WARNING"       => AlarmSeverity::Warning,
        _               => AlarmSeverity::Indeterminate,
    }
}
