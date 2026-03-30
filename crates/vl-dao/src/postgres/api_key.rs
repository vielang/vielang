use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::ApiKey;

use crate::{DaoError, PageData, PageLink};

pub struct ApiKeyDao {
    pool: PgPool,
}

impl ApiKeyDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>, DaoError> {
        let r = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, name, key_hash, key_prefix,
                    scopes, expires_at, last_used_at, enabled
             FROM api_key WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(r.map(|r| ApiKey {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            user_id:      r.user_id,
            name:         r.name,
            key_hash:     r.key_hash,
            key_prefix:   r.key_prefix,
            scopes:       serde_json::from_value(r.scopes).unwrap_or_default(),
            expires_at:   r.expires_at,
            last_used_at: r.last_used_at,
            enabled:      r.enabled,
        }))
    }

    /// Look up an API key by its SHA-256 hash (used during authentication).
    pub async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DaoError> {
        let r = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, name, key_hash, key_prefix,
                    scopes, expires_at, last_used_at, enabled
             FROM api_key WHERE key_hash = $1",
            key_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(r.map(|r| ApiKey {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            user_id:      r.user_id,
            name:         r.name,
            key_hash:     r.key_hash,
            key_prefix:   r.key_prefix,
            scopes:       serde_json::from_value(r.scopes).unwrap_or_default(),
            expires_at:   r.expires_at,
            last_used_at: r.last_used_at,
            enabled:      r.enabled,
        }))
    }

    pub async fn find_by_user(
        &self,
        user_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<ApiKey>, DaoError> {
        let offset = page.offset();
        let limit  = page.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, user_id, name, key_hash, key_prefix,
                    scopes, expires_at, last_used_at, enabled
             FROM api_key WHERE user_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            user_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM api_key WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .unwrap_or(0);

        let data: Vec<ApiKey> = rows.into_iter().map(|r| ApiKey {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            user_id:      r.user_id,
            name:         r.name,
            key_hash:     r.key_hash,
            key_prefix:   r.key_prefix,
            scopes:       serde_json::from_value(r.scopes).unwrap_or_default(),
            expires_at:   r.expires_at,
            last_used_at: r.last_used_at,
            enabled:      r.enabled,
        }).collect();

        let page_link = PageLink::new(page.page, page.page_size);
        Ok(PageData::new(data, total, &page_link))
    }

    pub async fn save(&self, key: &ApiKey) -> Result<(), DaoError> {
        let scopes = serde_json::to_value(&key.scopes).unwrap_or_default();
        sqlx::query!(
            "INSERT INTO api_key
                (id, created_time, tenant_id, user_id, name, key_hash, key_prefix,
                 scopes, expires_at, last_used_at, enabled)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
             ON CONFLICT (id) DO UPDATE SET
                name         = EXCLUDED.name,
                scopes       = EXCLUDED.scopes,
                expires_at   = EXCLUDED.expires_at,
                enabled      = EXCLUDED.enabled",
            key.id,
            key.created_time,
            key.tenant_id,
            key.user_id,
            key.name,
            key.key_hash,
            key.key_prefix,
            scopes,
            key.expires_at,
            key.last_used_at,
            key.enabled,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("api_key_hash_unique") {
                DaoError::Constraint("API key hash already exists".into())
            } else {
                DaoError::Database(e)
            }
        })?;
        Ok(())
    }

    pub async fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<(), DaoError> {
        let rows = sqlx::query!(
            "UPDATE api_key SET enabled = $1 WHERE id = $2",
            enabled, id
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .rows_affected();

        if rows == 0 { Err(DaoError::NotFound) } else { Ok(()) }
    }

    pub async fn update_last_used_at(&self, id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE api_key SET last_used_at = $1 WHERE id = $2",
            ts, id
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let rows = sqlx::query!(
            "DELETE FROM api_key WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .rows_affected();

        if rows == 0 { Err(DaoError::NotFound) } else { Ok(()) }
    }
}
