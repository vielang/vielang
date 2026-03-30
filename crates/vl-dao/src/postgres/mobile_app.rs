use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{
    MobileApp, MobileAppBundle, MobileAppStatus, PlatformType, QrCodeSettings,
};

use crate::{error::DaoError, PageData, PageLink};

fn parse_platform(s: &str) -> PlatformType {
    match s {
        "IOS" => PlatformType::Ios,
        _     => PlatformType::Android,
    }
}

fn platform_str(p: &PlatformType) -> &'static str {
    match p {
        PlatformType::Android => "ANDROID",
        PlatformType::Ios     => "IOS",
    }
}

fn parse_status(s: &str) -> MobileAppStatus {
    match s {
        "PUBLISHED"  => MobileAppStatus::Published,
        "DEPRECATED" => MobileAppStatus::Deprecated,
        "SUSPENDED"  => MobileAppStatus::Suspended,
        _            => MobileAppStatus::Draft,
    }
}

fn status_str(s: &MobileAppStatus) -> &'static str {
    match s {
        MobileAppStatus::Published  => "PUBLISHED",
        MobileAppStatus::Deprecated => "DEPRECATED",
        MobileAppStatus::Suspended  => "SUSPENDED",
        MobileAppStatus::Draft      => "DRAFT",
    }
}

// ── MobileAppDao ──────────────────────────────────────────────────────────────

pub struct MobileAppDao {
    pool: PgPool,
}

impl MobileAppDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MobileApp>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, created_time, tenant_id, pkg_name, title, app_secret,
                    platform_type, status, version_info, store_info
             FROM mobile_app WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| MobileApp {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            pkg_name:      r.pkg_name,
            title:         r.title,
            app_secret:    r.app_secret,
            platform_type: parse_platform(&r.platform_type),
            status:        parse_status(&r.status),
            version_info:  r.version_info,
            store_info:    r.store_info,
        }))
    }

    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<MobileApp>, DaoError> {
        let offset = page_link.page * page_link.page_size;

        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, pkg_name, title, app_secret,
                    platform_type, status, version_info, store_info
             FROM mobile_app
             WHERE tenant_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            tenant_id,
            page_link.page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let total: Option<i64> = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mobile_app WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?;
        let total = total.unwrap_or(0);
        let total_pages = (total + page_link.page_size - 1) / page_link.page_size;

        let data = rows.into_iter().map(|r| MobileApp {
            id:            r.id,
            created_time:  r.created_time,
            tenant_id:     r.tenant_id,
            pkg_name:      r.pkg_name,
            title:         r.title,
            app_secret:    r.app_secret,
            platform_type: parse_platform(&r.platform_type),
            status:        parse_status(&r.status),
            version_info:  r.version_info,
            store_info:    r.store_info,
        }).collect();

        Ok(PageData {
            data,
            total_pages,
            total_elements: total,
            has_next: page_link.page + 1 < total_pages,
        })
    }

    pub async fn save(&self, app: &MobileApp) -> Result<MobileApp, DaoError> {
        sqlx::query!(
            r#"INSERT INTO mobile_app
               (id, created_time, tenant_id, pkg_name, title, app_secret,
                platform_type, status, version_info, store_info)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               ON CONFLICT (tenant_id, pkg_name, platform_type)
               DO UPDATE SET title = EXCLUDED.title,
                             app_secret = EXCLUDED.app_secret,
                             status = EXCLUDED.status,
                             version_info = EXCLUDED.version_info,
                             store_info = EXCLUDED.store_info"#,
            app.id,
            app.created_time,
            app.tenant_id,
            app.pkg_name,
            app.title,
            app.app_secret,
            platform_str(&app.platform_type),
            status_str(&app.status),
            app.version_info,
            app.store_info,
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(app.id).await?.ok_or(DaoError::NotFound)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM mobile_app WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ── MobileAppBundleDao ────────────────────────────────────────────────────────

pub struct MobileAppBundleDao {
    pool: PgPool,
}

impl MobileAppBundleDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MobileAppBundle>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, created_time, tenant_id, title, android_app_id, ios_app_id,
                    layout_config, oauth2_client_ids
             FROM mobile_app_bundle WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            let ids: Vec<Uuid> = serde_json::from_value(r.oauth2_client_ids)
                .unwrap_or_default();
            MobileAppBundle {
                id:                r.id,
                created_time:      r.created_time,
                tenant_id:         r.tenant_id,
                title:             r.title,
                android_app_id:    r.android_app_id,
                ios_app_id:        r.ios_app_id,
                layout_config:     r.layout_config,
                oauth2_client_ids: ids,
            }
        }))
    }

    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<MobileAppBundle>, DaoError> {
        let offset = page_link.page * page_link.page_size;
        let rows = sqlx::query!(
            "SELECT id, created_time, tenant_id, title, android_app_id, ios_app_id,
                    layout_config, oauth2_client_ids
             FROM mobile_app_bundle
             WHERE tenant_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            tenant_id,
            page_link.page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        let total: Option<i64> = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mobile_app_bundle WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?;
        let total = total.unwrap_or(0);
        let total_pages = (total + page_link.page_size - 1) / page_link.page_size;

        let data = rows.into_iter().map(|r| {
            let ids: Vec<Uuid> = serde_json::from_value(r.oauth2_client_ids)
                .unwrap_or_default();
            MobileAppBundle {
                id:                r.id,
                created_time:      r.created_time,
                tenant_id:         r.tenant_id,
                title:             r.title,
                android_app_id:    r.android_app_id,
                ios_app_id:        r.ios_app_id,
                layout_config:     r.layout_config,
                oauth2_client_ids: ids,
            }
        }).collect();

        Ok(PageData { data, total_pages, total_elements: total, has_next: page_link.page + 1 < total_pages })
    }

    pub async fn save(&self, bundle: &MobileAppBundle) -> Result<MobileAppBundle, DaoError> {
        let ids = serde_json::to_value(&bundle.oauth2_client_ids)
            .unwrap_or(serde_json::Value::Array(vec![]));
        sqlx::query!(
            r#"INSERT INTO mobile_app_bundle
               (id, created_time, tenant_id, title, android_app_id, ios_app_id,
                layout_config, oauth2_client_ids)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (id)
               DO UPDATE SET title = EXCLUDED.title,
                             android_app_id = EXCLUDED.android_app_id,
                             ios_app_id = EXCLUDED.ios_app_id,
                             layout_config = EXCLUDED.layout_config,
                             oauth2_client_ids = EXCLUDED.oauth2_client_ids"#,
            bundle.id,
            bundle.created_time,
            bundle.tenant_id,
            bundle.title,
            bundle.android_app_id,
            bundle.ios_app_id,
            bundle.layout_config,
            ids,
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(bundle.id).await?.ok_or(DaoError::NotFound)
    }

    pub async fn update_oauth2_clients(
        &self,
        bundle_id: Uuid,
        client_ids: Vec<Uuid>,
    ) -> Result<(), DaoError> {
        let ids = serde_json::to_value(&client_ids)
            .unwrap_or(serde_json::Value::Array(vec![]));
        sqlx::query!(
            "UPDATE mobile_app_bundle SET oauth2_client_ids = $1 WHERE id = $2",
            ids,
            bundle_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        sqlx::query!("DELETE FROM mobile_app_bundle WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ── QrCodeSettingsDao ─────────────────────────────────────────────────────────

pub struct QrCodeSettingsDao {
    pool: PgPool,
}

impl QrCodeSettingsDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Option<QrCodeSettings>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, created_time, tenant_id, use_system_settings, use_default_app,
                    mobile_app_bundle_id, qr_code_config, android_enabled, ios_enabled
             FROM qr_code_settings WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| QrCodeSettings {
            id:                   r.id,
            created_time:         r.created_time,
            tenant_id:            r.tenant_id,
            use_system_settings:  r.use_system_settings,
            use_default_app:      r.use_default_app,
            mobile_app_bundle_id: r.mobile_app_bundle_id,
            qr_code_config:       r.qr_code_config,
            android_enabled:      r.android_enabled,
            ios_enabled:          r.ios_enabled,
            google_play_link:     None,
            app_store_link:       None,
        }))
    }

    pub async fn save(&self, s: &QrCodeSettings) -> Result<QrCodeSettings, DaoError> {
        sqlx::query!(
            r#"INSERT INTO qr_code_settings
               (id, created_time, tenant_id, use_system_settings, use_default_app,
                mobile_app_bundle_id, qr_code_config, android_enabled, ios_enabled)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT (tenant_id)
               DO UPDATE SET use_system_settings = EXCLUDED.use_system_settings,
                             use_default_app = EXCLUDED.use_default_app,
                             mobile_app_bundle_id = EXCLUDED.mobile_app_bundle_id,
                             qr_code_config = EXCLUDED.qr_code_config,
                             android_enabled = EXCLUDED.android_enabled,
                             ios_enabled = EXCLUDED.ios_enabled"#,
            s.id,
            s.created_time,
            s.tenant_id,
            s.use_system_settings,
            s.use_default_app,
            s.mobile_app_bundle_id,
            s.qr_code_config,
            s.android_enabled,
            s.ios_enabled,
        )
        .execute(&self.pool)
        .await?;
        self.find_by_tenant(s.tenant_id).await?.ok_or(DaoError::NotFound)
    }
}
