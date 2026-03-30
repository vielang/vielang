use sqlx::PgPool;
use uuid::Uuid;

use crate::{DaoError, page::PageData};

#[derive(Debug, Clone)]
pub struct TenantLdapConfig {
    pub id:               Uuid,
    pub tenant_id:        Uuid,
    pub enabled:          bool,
    pub server_url:       String,
    pub use_tls:          bool,
    pub base_dn:          String,
    pub search_filter:    String,
    pub bind_dn:          String,
    pub bind_password:    String,
    pub username_attr:    String,
    pub first_name_attr:  String,
    pub last_name_attr:   String,
    pub email_attr:       String,
    pub default_authority: String,
    pub created_time:     i64,
    pub updated_time:     i64,
    /// Optional group search base for periodic LDAP sync (P4)
    pub group_search_base: Option<String>,
    /// Optional group filter for periodic LDAP sync (P4)
    pub group_filter:      Option<String>,
}

pub struct LdapConfigDao {
    pool: PgPool,
}

impl LdapConfigDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantLdapConfig>, DaoError> {
        let r = sqlx::query!(
            r#"SELECT id, tenant_id, enabled, server_url, use_tls, base_dn,
                      search_filter, bind_dn, bind_password, username_attr,
                      first_name_attr, last_name_attr, email_attr, default_authority,
                      created_time, updated_time,
                      group_search_base, group_filter
               FROM tenant_ldap_config WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| TenantLdapConfig {
            id:               r.id,
            tenant_id:        r.tenant_id,
            enabled:          r.enabled,
            server_url:       r.server_url,
            use_tls:          r.use_tls,
            base_dn:          r.base_dn,
            search_filter:    r.search_filter,
            bind_dn:          r.bind_dn,
            bind_password:    r.bind_password,
            username_attr:    r.username_attr,
            first_name_attr:  r.first_name_attr,
            last_name_attr:   r.last_name_attr,
            email_attr:       r.email_attr,
            default_authority: r.default_authority,
            created_time:     r.created_time,
            updated_time:     r.updated_time,
            group_search_base: r.group_search_base,
            group_filter:      r.group_filter,
        }))
    }

    /// Return all tenant IDs that have an enabled LDAP config — used for periodic sync.
    pub async fn find_all_tenants_with_ldap(&self) -> Result<Vec<Uuid>, DaoError> {
        let rows = sqlx::query!(
            "SELECT DISTINCT tenant_id FROM tenant_ldap_config WHERE enabled = TRUE"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.tenant_id).collect())
    }

    pub async fn upsert(&self, cfg: &TenantLdapConfig) -> Result<(), DaoError> {
        sqlx::query!(
            r#"INSERT INTO tenant_ldap_config
                (id, tenant_id, enabled, server_url, use_tls, base_dn, search_filter,
                 bind_dn, bind_password, username_attr, first_name_attr, last_name_attr,
                 email_attr, default_authority, created_time, updated_time,
                 group_search_base, group_filter)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
               ON CONFLICT (tenant_id) DO UPDATE SET
                 enabled          = EXCLUDED.enabled,
                 server_url       = EXCLUDED.server_url,
                 use_tls          = EXCLUDED.use_tls,
                 base_dn          = EXCLUDED.base_dn,
                 search_filter    = EXCLUDED.search_filter,
                 bind_dn          = EXCLUDED.bind_dn,
                 bind_password    = EXCLUDED.bind_password,
                 username_attr    = EXCLUDED.username_attr,
                 first_name_attr  = EXCLUDED.first_name_attr,
                 last_name_attr   = EXCLUDED.last_name_attr,
                 email_attr       = EXCLUDED.email_attr,
                 default_authority = EXCLUDED.default_authority,
                 updated_time     = EXCLUDED.updated_time,
                 group_search_base = EXCLUDED.group_search_base,
                 group_filter      = EXCLUDED.group_filter"#,
            cfg.id,
            cfg.tenant_id,
            cfg.enabled,
            cfg.server_url,
            cfg.use_tls,
            cfg.base_dn,
            cfg.search_filter,
            cfg.bind_dn,
            cfg.bind_password,
            cfg.username_attr,
            cfg.first_name_attr,
            cfg.last_name_attr,
            cfg.email_attr,
            cfg.default_authority,
            cfg.created_time,
            cfg.updated_time,
            cfg.group_search_base,
            cfg.group_filter,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_by_tenant(&self, tenant_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM tenant_ldap_config WHERE tenant_id = $1", tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
