use sqlx::PgPool;
use uuid::Uuid;

use crate::DaoError;

#[derive(Debug, Clone)]
pub struct TenantSamlConfig {
    pub id:               Uuid,
    pub tenant_id:        Uuid,
    pub enabled:          bool,
    pub entity_id:        String,
    pub sso_url:          String,
    pub slo_url:          Option<String>,
    pub idp_certificate:  String,
    pub sp_private_key:   Option<String>,
    pub sp_certificate:   Option<String>,
    pub email_attr:       String,
    pub first_name_attr:  String,
    pub last_name_attr:   String,
    pub force_authn:      bool,
    pub name_id_format:   String,
    pub default_authority: String,
    pub created_time:     i64,
    pub updated_time:     i64,
}

pub struct SamlConfigDao {
    pool: PgPool,
}

impl SamlConfigDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantSamlConfig>, DaoError> {
        let r = sqlx::query!(
            r#"SELECT id, tenant_id, enabled, entity_id, sso_url, slo_url,
                      idp_certificate, sp_private_key, sp_certificate,
                      email_attr, first_name_attr, last_name_attr,
                      force_authn, name_id_format, default_authority,
                      created_time, updated_time
               FROM tenant_saml_config WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| TenantSamlConfig {
            id:               r.id,
            tenant_id:        r.tenant_id,
            enabled:          r.enabled,
            entity_id:        r.entity_id,
            sso_url:          r.sso_url,
            slo_url:          r.slo_url,
            idp_certificate:  r.idp_certificate,
            sp_private_key:   r.sp_private_key,
            sp_certificate:   r.sp_certificate,
            email_attr:       r.email_attr,
            first_name_attr:  r.first_name_attr,
            last_name_attr:   r.last_name_attr,
            force_authn:      r.force_authn,
            name_id_format:   r.name_id_format,
            default_authority: r.default_authority,
            created_time:     r.created_time,
            updated_time:     r.updated_time,
        }))
    }

    pub async fn upsert(&self, cfg: &TenantSamlConfig) -> Result<(), DaoError> {
        sqlx::query!(
            r#"INSERT INTO tenant_saml_config
                (id, tenant_id, enabled, entity_id, sso_url, slo_url,
                 idp_certificate, sp_private_key, sp_certificate,
                 email_attr, first_name_attr, last_name_attr,
                 force_authn, name_id_format, default_authority, created_time, updated_time)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
               ON CONFLICT (tenant_id) DO UPDATE SET
                 enabled          = EXCLUDED.enabled,
                 entity_id        = EXCLUDED.entity_id,
                 sso_url          = EXCLUDED.sso_url,
                 slo_url          = EXCLUDED.slo_url,
                 idp_certificate  = EXCLUDED.idp_certificate,
                 sp_private_key   = EXCLUDED.sp_private_key,
                 sp_certificate   = EXCLUDED.sp_certificate,
                 email_attr       = EXCLUDED.email_attr,
                 first_name_attr  = EXCLUDED.first_name_attr,
                 last_name_attr   = EXCLUDED.last_name_attr,
                 force_authn      = EXCLUDED.force_authn,
                 name_id_format   = EXCLUDED.name_id_format,
                 default_authority = EXCLUDED.default_authority,
                 updated_time     = EXCLUDED.updated_time"#,
            cfg.id,
            cfg.tenant_id,
            cfg.enabled,
            cfg.entity_id,
            cfg.sso_url,
            cfg.slo_url,
            cfg.idp_certificate,
            cfg.sp_private_key,
            cfg.sp_certificate,
            cfg.email_attr,
            cfg.first_name_attr,
            cfg.last_name_attr,
            cfg.force_authn,
            cfg.name_id_format,
            cfg.default_authority,
            cfg.created_time,
            cfg.updated_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_by_tenant(&self, tenant_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM tenant_saml_config WHERE tenant_id = $1", tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
