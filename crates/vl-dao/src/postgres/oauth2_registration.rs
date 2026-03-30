use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{OAuth2ClientRegistration, OAuth2MapperConfig};

use crate::{DaoError, PageData, PageLink};

pub struct OAuth2RegistrationDao {
    pool: PgPool,
}

impl OAuth2RegistrationDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuth2ClientRegistration>, DaoError> {
        let r = sqlx::query!(
            "SELECT id, created_time, tenant_id, provider_name, client_id, client_secret,
                    authorization_uri, token_uri, user_info_uri, scope, user_name_attribute,
                    mapper_config, enabled
             FROM oauth2_client_registration WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(r.map(|r| {
            let scope: Vec<String> = serde_json::from_value(r.scope).unwrap_or_default();
            let mapper_config: OAuth2MapperConfig =
                serde_json::from_value(r.mapper_config).unwrap_or_default();
            OAuth2ClientRegistration {
                id:                 r.id,
                created_time:       r.created_time,
                tenant_id:          r.tenant_id,
                provider_name:      r.provider_name,
                client_id:          r.client_id,
                client_secret:      r.client_secret,
                authorization_uri:  r.authorization_uri,
                token_uri:          r.token_uri,
                user_info_uri:      r.user_info_uri,
                scope,
                user_name_attribute: r.user_name_attribute,
                mapper_config,
                enabled:            r.enabled,
            }
        }))
    }

    pub async fn find_enabled_by_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<OAuth2ClientRegistration>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, provider_name, client_id, client_secret,
                    authorization_uri, token_uri, user_info_uri, scope, user_name_attribute,
                    mapper_config, enabled
             FROM oauth2_client_registration
             WHERE tenant_id = $1 AND enabled = true
             ORDER BY created_time ASC",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(rows.into_iter().map(|r| {
            let scope: Vec<String> = serde_json::from_value(r.scope).unwrap_or_default();
            let mapper_config: OAuth2MapperConfig =
                serde_json::from_value(r.mapper_config).unwrap_or_default();
            OAuth2ClientRegistration {
                id:                 r.id,
                created_time:       r.created_time,
                tenant_id:          r.tenant_id,
                provider_name:      r.provider_name,
                client_id:          r.client_id,
                client_secret:      r.client_secret,
                authorization_uri:  r.authorization_uri,
                token_uri:          r.token_uri,
                user_info_uri:      r.user_info_uri,
                scope,
                user_name_attribute: r.user_name_attribute,
                mapper_config,
                enabled:            r.enabled,
            }
        }).collect())
    }

    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<OAuth2ClientRegistration>, DaoError> {
        let offset = page.offset();
        let limit  = page.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, provider_name, client_id, client_secret,
                    authorization_uri, token_uri, user_info_uri, scope, user_name_attribute,
                    mapper_config, enabled
             FROM oauth2_client_registration
             WHERE tenant_id = $1
             ORDER BY created_time ASC
             LIMIT $2 OFFSET $3",
            tenant_id, limit, offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM oauth2_client_registration WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .unwrap_or(0);

        let data: Vec<OAuth2ClientRegistration> = rows.into_iter().map(|r| {
            let scope: Vec<String> = serde_json::from_value(r.scope).unwrap_or_default();
            let mapper_config: OAuth2MapperConfig =
                serde_json::from_value(r.mapper_config).unwrap_or_default();
            OAuth2ClientRegistration {
                id:                 r.id,
                created_time:       r.created_time,
                tenant_id:          r.tenant_id,
                provider_name:      r.provider_name,
                client_id:          r.client_id,
                client_secret:      r.client_secret,
                authorization_uri:  r.authorization_uri,
                token_uri:          r.token_uri,
                user_info_uri:      r.user_info_uri,
                scope,
                user_name_attribute: r.user_name_attribute,
                mapper_config,
                enabled:            r.enabled,
            }
        }).collect();

        let page_link = PageLink::new(page.page, page.page_size);
        Ok(PageData::new(data, total, &page_link))
    }

    pub async fn save(&self, reg: &OAuth2ClientRegistration) -> Result<(), DaoError> {
        let scope         = serde_json::to_value(&reg.scope).unwrap_or_default();
        let mapper_config = serde_json::to_value(&reg.mapper_config).unwrap_or_default();

        sqlx::query!(
            "INSERT INTO oauth2_client_registration
                (id, created_time, tenant_id, provider_name, client_id, client_secret,
                 authorization_uri, token_uri, user_info_uri, scope, user_name_attribute,
                 mapper_config, enabled)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
             ON CONFLICT (id) DO UPDATE SET
                provider_name       = EXCLUDED.provider_name,
                client_id           = EXCLUDED.client_id,
                client_secret       = EXCLUDED.client_secret,
                authorization_uri   = EXCLUDED.authorization_uri,
                token_uri           = EXCLUDED.token_uri,
                user_info_uri       = EXCLUDED.user_info_uri,
                scope               = EXCLUDED.scope,
                user_name_attribute = EXCLUDED.user_name_attribute,
                mapper_config       = EXCLUDED.mapper_config,
                enabled             = EXCLUDED.enabled",
            reg.id,
            reg.created_time,
            reg.tenant_id,
            reg.provider_name,
            reg.client_id,
            reg.client_secret,
            reg.authorization_uri,
            reg.token_uri,
            reg.user_info_uri,
            scope,
            reg.user_name_attribute,
            mapper_config,
            reg.enabled,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let rows = sqlx::query!(
            "DELETE FROM oauth2_client_registration WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?
        .rows_affected();

        if rows == 0 { Err(DaoError::NotFound) } else { Ok(()) }
    }
}
