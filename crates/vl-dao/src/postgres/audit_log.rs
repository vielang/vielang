use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{AuditActionStatus, AuditActionType, AuditLog};

use crate::{DaoError, PageData, PageLink};

pub struct AuditLogDao {
    pool: PgPool,
}

impl AuditLogDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn save(&self, log: &AuditLog) -> Result<(), DaoError> {
        sqlx::query!(
            "INSERT INTO audit_log
                (id, created_time, tenant_id, user_id, user_name, action_type,
                 action_data, action_status, action_failure_details,
                 entity_type, entity_id, entity_name)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            log.id,
            log.created_time,
            log.tenant_id,
            log.user_id,
            log.user_name,
            log.action_type.as_str(),
            log.action_data,
            log.action_status.as_str(),
            log.action_failure_details,
            log.entity_type,
            log.entity_id,
            log.entity_name,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }

    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<AuditLog>, DaoError> {
        let offset = page.offset();
        let limit  = page.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, user_name, action_type,
                    action_data, action_status, action_failure_details,
                    entity_type, entity_id, entity_name
             FROM audit_log
             WHERE tenant_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            tenant_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM audit_log WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .unwrap_or(0);

        let data = rows.into_iter().map(|r| AuditLog {
            id:                     r.id,
            created_time:           r.created_time,
            tenant_id:              r.tenant_id,
            user_id:                r.user_id,
            user_name:              r.user_name,
            action_type:            AuditActionType::from_str(&r.action_type),
            action_data:            r.action_data,
            action_status:          if r.action_status == "FAILURE" {
                                        AuditActionStatus::Failure
                                    } else {
                                        AuditActionStatus::Success
                                    },
            action_failure_details: r.action_failure_details,
            entity_type:            r.entity_type,
            entity_id:              r.entity_id,
            entity_name:            r.entity_name,
        }).collect();

        let page_link = PageLink::new(page.page, page.page_size);
        Ok(PageData::new(data, total, &page_link))
    }

    pub async fn find_by_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<AuditLog>, DaoError> {
        let offset = page.offset();
        let limit  = page.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, user_name, action_type,
                    action_data, action_status, action_failure_details,
                    entity_type, entity_id, entity_name
             FROM audit_log
             WHERE tenant_id = $1 AND user_id = $2
             ORDER BY created_time DESC
             LIMIT $3 OFFSET $4",
            tenant_id, user_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM audit_log WHERE tenant_id = $1 AND user_id = $2",
            tenant_id, user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .unwrap_or(0);

        let data = rows.into_iter().map(|r| AuditLog {
            id:                     r.id,
            created_time:           r.created_time,
            tenant_id:              r.tenant_id,
            user_id:                r.user_id,
            user_name:              r.user_name,
            action_type:            AuditActionType::from_str(&r.action_type),
            action_data:            r.action_data,
            action_status:          if r.action_status == "FAILURE" {
                                        AuditActionStatus::Failure
                                    } else {
                                        AuditActionStatus::Success
                                    },
            action_failure_details: r.action_failure_details,
            entity_type:            r.entity_type,
            entity_id:              r.entity_id,
            entity_name:            r.entity_name,
        }).collect();
        Ok(PageData::new(data, total, page))
    }

    pub async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<AuditLog>, DaoError> {
        let offset = page.offset();
        let limit  = page.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, user_name, action_type,
                    action_data, action_status, action_failure_details,
                    entity_type, entity_id, entity_name
             FROM audit_log
             WHERE entity_type = $1 AND entity_id = $2
             ORDER BY created_time DESC
             LIMIT $3 OFFSET $4",
            entity_type, entity_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = $1 AND entity_id = $2",
            entity_type, entity_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .unwrap_or(0);

        let data = rows.into_iter().map(|r| AuditLog {
            id:                     r.id,
            created_time:           r.created_time,
            tenant_id:              r.tenant_id,
            user_id:                r.user_id,
            user_name:              r.user_name,
            action_type:            AuditActionType::from_str(&r.action_type),
            action_data:            r.action_data,
            action_status:          if r.action_status == "FAILURE" {
                                        AuditActionStatus::Failure
                                    } else {
                                        AuditActionStatus::Success
                                    },
            action_failure_details: r.action_failure_details,
            entity_type:            r.entity_type,
            entity_id:              r.entity_id,
            entity_name:            r.entity_name,
        }).collect();

        let page_link = PageLink::new(page.page, page.page_size);
        Ok(PageData::new(data, total, &page_link))
    }
}
