use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{AdminSettings, UsageInfo};

use crate::error::DaoError;

// ── AdminSettingsDao ──────────────────────────────────────────────────────────

pub struct AdminSettingsDao {
    pool: PgPool,
}

impl AdminSettingsDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find which tenant has registered a given partner API key
    /// (admin_settings key='partnerApiKey', json_value->>'key' = $1)
    pub async fn find_tenant_by_partner_key(&self, api_key: &str) -> Result<Option<Uuid>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT tenant_id FROM admin_settings
               WHERE key = 'partnerApiKey' AND json_value->>'key' = $1"#,
            api_key,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.tenant_id))
    }

    /// Find which tenant has registered a given white-label domain
    /// (admin_settings key='whitelabelDomain', json_value->>'domain' = $1)
    pub async fn find_tenant_by_whitelabel_domain(&self, domain: &str) -> Result<Option<Uuid>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT tenant_id FROM admin_settings
               WHERE key = 'whitelabelDomain' AND json_value->>'domain' = $1"#,
            domain,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.tenant_id))
    }

    /// Find all sub-tenants that were onboarded by a given partner tenant
    /// (admin_settings key='partnerTenantId', json_value->>'tenantId' = partner_id_str)
    pub async fn find_partner_sub_tenants(&self, partner_tenant_id: Uuid) -> Result<Vec<Uuid>, DaoError> {
        let id_str = partner_tenant_id.to_string();
        let rows = sqlx::query!(
            r#"SELECT tenant_id FROM admin_settings
               WHERE key = 'partnerTenantId' AND json_value->>'tenantId' = $1"#,
            id_str,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.tenant_id).collect())
    }

    /// GET /api/admin/settings/{key}
    pub async fn find_by_key(
        &self,
        tenant_id: Uuid,
        key: &str,
    ) -> Result<Option<AdminSettings>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, key, json_value
               FROM admin_settings
               WHERE tenant_id = $1 AND key = $2"#,
            tenant_id as Uuid,
            key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| AdminSettings {
            id:           r.id,
            created_time: r.created_time,
            tenant_id:    r.tenant_id,
            key:          r.key,
            json_value:   r.json_value,
        }))
    }

    /// POST /api/admin/settings
    pub async fn save(&self, s: &AdminSettings) -> Result<AdminSettings, DaoError> {
        sqlx::query!(
            r#"INSERT INTO admin_settings (id, created_time, tenant_id, key, json_value)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (tenant_id, key)
               DO UPDATE SET json_value = EXCLUDED.json_value,
                             created_time = EXCLUDED.created_time"#,
            s.id,
            s.created_time,
            s.tenant_id,
            s.key,
            s.json_value,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_key(s.tenant_id, &s.key)
            .await?
            .ok_or(DaoError::NotFound)
    }

    /// DELETE admin_settings by tenant_id + key
    #[instrument(skip(self))]
    pub async fn delete_by_key(
        &self,
        tenant_id: Uuid,
        key: &str,
    ) -> Result<bool, DaoError> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            "DELETE FROM admin_settings WHERE tenant_id = $1 AND key = $2",
            tenant_id,
            key,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ── UsageInfoDao ──────────────────────────────────────────────────────────────

pub struct UsageInfoDao {
    pool: PgPool,
}

impl UsageInfoDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// GET /api/usage — aggregate counts for a tenant
    pub async fn get_tenant_usage(&self, tenant_id: Uuid) -> Result<UsageInfo, DaoError> {
        let devices = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM device WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let assets = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM asset WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let customers = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM customer WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let users = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tb_user WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let dashboards = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM dashboard WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let alarms = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM alarm WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(UsageInfo {
            devices,
            max_devices:            -1, // unlimited (no tenant profile limits implemented yet)
            assets,
            max_assets:             -1,
            customers,
            max_customers:          -1,
            users,
            max_users:              -1,
            dashboards,
            max_dashboards:         -1,
            edges:                  0,
            max_edges:              -1,
            transport_messages:     0,
            max_transport_messages: -1,
            js_executions:          0,
            tbel_executions:        0,
            max_js_executions:      -1,
            max_tbel_executions:    -1,
            emails:                 0,
            max_emails:             -1,
            sms:                    0,
            max_sms:                -1,
            sms_enabled:            Some(false),
            alarms,
            max_alarms:             -1,
        })
    }
}
