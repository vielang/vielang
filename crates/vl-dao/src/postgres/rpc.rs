use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Rpc, RpcRequest, RpcStatus};
use crate::{DaoError, PageData, PageLink};

pub struct RpcDao {
    pool: PgPool,
}

impl RpcDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Rpc>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_id, request_id,
                   expiration_time, request, response, status, additional_info
            FROM rpc WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Rpc {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            request_id: r.request_id,
            expiration_time: r.expiration_time,
            request: serde_json::from_value(r.request).unwrap_or_else(|_| RpcRequest {
                method: String::new(),
                params: serde_json::Value::Null,
                oneway: false,
                timeout: 10000,
                additional_info: None,
            }),
            response: r.response,
            status: RpcStatus::from_str(&r.status),
            additional_info: r.additional_info,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_device(
        &self,
        tenant_id: Uuid,
        device_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Rpc>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM rpc WHERE tenant_id = $1 AND device_id = $2"#,
            tenant_id,
            device_id,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_id, request_id,
                   expiration_time, request, response, status, additional_info
            FROM rpc
            WHERE tenant_id = $1 AND device_id = $2
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            device_id,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Rpc {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            request_id: r.request_id,
            expiration_time: r.expiration_time,
            request: serde_json::from_value(r.request).unwrap_or_else(|_| RpcRequest {
                method: String::new(),
                params: serde_json::Value::Null,
                oneway: false,
                timeout: 10000,
                additional_info: None,
            }),
            response: r.response,
            status: RpcStatus::from_str(&r.status),
            additional_info: r.additional_info,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Find pending RPC requests for device (not expired, not final status)
    #[instrument(skip(self))]
    pub async fn find_pending_by_device(
        &self,
        device_id: Uuid,
    ) -> Result<Vec<Rpc>, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_id, request_id,
                   expiration_time, request, response, status, additional_info
            FROM rpc
            WHERE device_id = $1
            AND status IN ('QUEUED', 'SENT', 'DELIVERED')
            AND expiration_time > $2
            ORDER BY created_time ASC
            "#,
            device_id,
            now,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Rpc {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            request_id: r.request_id,
            expiration_time: r.expiration_time,
            request: serde_json::from_value(r.request).unwrap_or_else(|_| RpcRequest {
                method: String::new(),
                params: serde_json::Value::Null,
                oneway: false,
                timeout: 10000,
                additional_info: None,
            }),
            response: r.response,
            status: RpcStatus::from_str(&r.status),
            additional_info: r.additional_info,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, rpc: &Rpc) -> Result<Rpc, DaoError> {
        let request_json = serde_json::to_value(&rpc.request)?;

        sqlx::query!(
            r#"
            INSERT INTO rpc (
                id, created_time, tenant_id, device_id, request_id,
                expiration_time, request, response, status, additional_info
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
                response        = EXCLUDED.response,
                status          = EXCLUDED.status,
                additional_info = EXCLUDED.additional_info
            "#,
            rpc.id,
            rpc.created_time,
            rpc.tenant_id,
            rpc.device_id,
            rpc.request_id,
            rpc.expiration_time,
            request_json,
            rpc.response,
            rpc.status.as_str(),
            rpc.additional_info,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(rpc.id).await?.ok_or(DaoError::NotFound)
    }

    /// Update RPC status
    #[instrument(skip(self))]
    pub async fn update_status(
        &self,
        id: Uuid,
        status: RpcStatus,
        response: Option<serde_json::Value>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            UPDATE rpc SET status = $2, response = COALESCE($3, response)
            WHERE id = $1
            "#,
            id,
            status.as_str(),
            response,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM rpc WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Expire old RPC requests
    #[instrument(skip(self))]
    pub async fn expire_old_requests(&self) -> Result<i64, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();

        let result = sqlx::query!(
            r#"
            UPDATE rpc SET status = 'EXPIRED'
            WHERE status IN ('QUEUED', 'SENT', 'DELIVERED')
            AND expiration_time < $1
            "#,
            now,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Get next request_id for device
    #[instrument(skip(self))]
    pub async fn get_next_request_id(&self, device_id: Uuid) -> Result<i32, DaoError> {
        let max_id: Option<i32> = sqlx::query_scalar!(
            "SELECT MAX(request_id) FROM rpc WHERE device_id = $1",
            device_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(max_id.unwrap_or(0) + 1)
    }
}
