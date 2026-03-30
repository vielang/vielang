use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::{DaoError, PageData, PageLink};

// ── EntityAlarmRow ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EntityAlarmRow {
    pub tenant_id: Uuid,
    pub entity_type: Option<String>,
    pub entity_id: Uuid,
    pub created_time: i64,
    pub alarm_type: String,
    pub customer_id: Option<Uuid>,
    pub alarm_id: Option<Uuid>,
}

// ── EntityAlarmDao ───────────────────────────────────────────────────────────

pub struct EntityAlarmDao {
    pool: PgPool,
}

impl EntityAlarmDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// INSERT ON CONFLICT DO NOTHING — idempotent link between entity and alarm
    #[instrument(skip(self))]
    pub async fn save(&self, ea: &EntityAlarmRow) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO entity_alarm (
                tenant_id, entity_type, entity_id, created_time,
                alarm_type, customer_id, alarm_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT DO NOTHING
            "#,
            ea.tenant_id,
            ea.entity_type,
            ea.entity_id,
            ea.created_time,
            ea.alarm_type,
            ea.customer_id,
            ea.alarm_id,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn find_by_entity(
        &self,
        entity_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<EntityAlarmRow>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM entity_alarm WHERE entity_id = $1",
            entity_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT tenant_id, entity_type, entity_id, created_time,
                   alarm_type, customer_id, alarm_id
            FROM entity_alarm
            WHERE entity_id = $1
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
            entity_id,
            page_link.page_size,
            page_link.offset()
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| EntityAlarmRow {
            tenant_id:   r.tenant_id,
            entity_type: r.entity_type,
            entity_id:   r.entity_id,
            created_time: r.created_time,
            alarm_type:  r.alarm_type,
            customer_id: r.customer_id,
            alarm_id:    Some(r.alarm_id),
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn delete_by_alarm(&self, alarm_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM entity_alarm WHERE alarm_id = $1",
            alarm_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ── AlarmTypesDao ────────────────────────────────────────────────────────────

pub struct AlarmTypesDao {
    pool: PgPool,
}

impl AlarmTypesDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Register an alarm type for a tenant. INSERT ON CONFLICT DO NOTHING — idempotent.
    #[instrument(skip(self))]
    pub async fn register_type(&self, tenant_id: Uuid, alarm_type: &str) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO alarm_types (tenant_id, type)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            tenant_id,
            alarm_type,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;
        Ok(())
    }

    /// List all registered alarm types for a tenant, ordered alphabetically.
    #[instrument(skip(self))]
    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query_scalar!(
            "SELECT type FROM alarm_types WHERE tenant_id = $1 ORDER BY type",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
