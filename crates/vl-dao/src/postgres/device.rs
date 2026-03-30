use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Device, DeviceCredentials, DeviceCredentialsType, DeviceInfoView};
use crate::{DaoError, PageData, PageLink};

pub struct DeviceDao {
    pool: PgPool,
}

impl DeviceDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Device>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    /// GET /api/device/info/{deviceId} — device info view (JOIN device_profile + customer)
    #[instrument(skip(self))]
    pub async fn find_info_by_id(&self, id: Uuid) -> Result<Option<DeviceInfoView>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT d.id, d.created_time, d.tenant_id, d.customer_id, d.name, d.label,
                   d.device_profile_id, dp.name as device_profile_name,
                   c.title as "customer_title?",
                   d.firmware_id, d.software_id
            FROM device d
            JOIN device_profile dp ON d.device_profile_id = dp.id
            LEFT JOIN customer c ON d.customer_id = c.id
            WHERE d.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DeviceInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            label: r.label,
            device_profile_id: r.device_profile_id,
            device_profile_name: r.device_profile_name,
            customer_title: r.customer_title,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
        }))
    }

    /// GET /api/tenant/deviceInfos — paginated device info views for tenant
    #[instrument(skip(self))]
    pub async fn find_infos_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<DeviceInfoView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM device d
               WHERE d.tenant_id = $1
               AND ($2::text IS NULL OR LOWER(d.name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT d.id, d.created_time, d.tenant_id, d.customer_id, d.name, d.label,
                   d.device_profile_id, dp.name as device_profile_name,
                   c.title as "customer_title?",
                   d.firmware_id, d.software_id
            FROM device d
            JOIN device_profile dp ON d.device_profile_id = dp.id
            LEFT JOIN customer c ON d.customer_id = c.id
            WHERE d.tenant_id = $1
            AND ($2::text IS NULL OR LOWER(d.name) LIKE LOWER($2))
            ORDER BY d.created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| DeviceInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            label: r.label,
            device_profile_id: r.device_profile_id,
            device_profile_name: r.device_profile_name,
            customer_title: r.customer_title,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// GET /api/customer/{customerId}/deviceInfos — paginated device info views for customer
    #[instrument(skip(self))]
    pub async fn find_infos_by_customer(
        &self,
        customer_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<DeviceInfoView>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM device d
               WHERE d.customer_id = $1
               AND ($2::text IS NULL OR LOWER(d.name) LIKE LOWER($2))"#,
            customer_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT d.id, d.created_time, d.tenant_id, d.customer_id, d.name, d.label,
                   d.device_profile_id, dp.name as device_profile_name,
                   c.title as "customer_title?",
                   d.firmware_id, d.software_id
            FROM device d
            JOIN device_profile dp ON d.device_profile_id = dp.id
            LEFT JOIN customer c ON d.customer_id = c.id
            WHERE d.customer_id = $1
            AND ($2::text IS NULL OR LOWER(d.name) LIKE LOWER($2))
            ORDER BY d.created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            customer_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| DeviceInfoView {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            name: r.name,
            label: r.label,
            device_profile_id: r.device_profile_id,
            device_profile_name: r.device_profile_name,
            customer_title: r.customer_title,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Device>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM device
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Export all devices for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Device>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info.and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, device: &Device) -> Result<Device, DaoError> {
        let additional_info = device.additional_info.as_ref()
            .map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO device (
                id, created_time, tenant_id, customer_id, device_profile_id,
                name, type, label, device_data, firmware_id, software_id,
                external_id, additional_info, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (id) DO UPDATE SET
                name              = EXCLUDED.name,
                type              = EXCLUDED.type,
                label             = EXCLUDED.label,
                customer_id       = EXCLUDED.customer_id,
                device_profile_id = EXCLUDED.device_profile_id,
                device_data       = EXCLUDED.device_data,
                firmware_id       = EXCLUDED.firmware_id,
                software_id       = EXCLUDED.software_id,
                external_id       = EXCLUDED.external_id,
                additional_info   = EXCLUDED.additional_info,
                version           = device.version + 1
            "#,
            device.id,
            device.created_time,
            device.tenant_id,
            device.customer_id,
            device.device_profile_id,
            device.name,
            device.device_type,
            device.label,
            device.device_data,
            device.firmware_id,
            device.software_id,
            device.external_id,
            additional_info,
            device.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(device.id).await?
            .ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM device WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Tìm nhiều devices theo list IDs — dùng cho batch GET /api/devices?deviceIds=...
    #[instrument(skip(self))]
    pub async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Device>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device WHERE id = ANY($1)
            "#,
            ids,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect())
    }

    /// Lấy devices của một customer — dùng cho GET /api/customer/{id}/devices
    #[instrument(skip(self))]
    pub async fn find_by_customer(
        &self,
        customer_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Device>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM device
               WHERE customer_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            customer_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device
            WHERE customer_id = $1
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            customer_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Lấy credentials của device — GET /api/device/{id}/credentials
    #[instrument(skip(self))]
    pub async fn get_credentials(
        &self,
        device_id: Uuid,
    ) -> Result<Option<DeviceCredentials>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, device_id, credentials_type,
                   credentials_id, credentials_value
            FROM device_credentials WHERE device_id = $1
            "#,
            device_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DeviceCredentials {
            id: r.id,
            created_time: r.created_time,
            device_id: r.device_id,
            credentials_type: parse_cred_type(&r.credentials_type),
            credentials_id: r.credentials_id,
            credentials_value: r.credentials_value,
        }))
    }

    /// Upsert credentials — POST /api/device/{id}/credentials
    #[instrument(skip(self))]
    pub async fn save_credentials(
        &self,
        creds: &DeviceCredentials,
    ) -> Result<DeviceCredentials, DaoError> {
        let cred_type = cred_type_str(&creds.credentials_type);
        sqlx::query!(
            r#"
            INSERT INTO device_credentials (
                id, created_time, device_id, credentials_type,
                credentials_id, credentials_value
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (device_id) DO UPDATE SET
                credentials_type  = EXCLUDED.credentials_type,
                credentials_id    = EXCLUDED.credentials_id,
                credentials_value = EXCLUDED.credentials_value
            "#,
            creds.id,
            creds.created_time,
            creds.device_id,
            cred_type,
            creds.credentials_id,
            creds.credentials_value,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.get_credentials(creds.device_id).await?.ok_or(DaoError::NotFound)
    }

    /// Tìm device theo tên trong tenant — dùng cho bulk import
    #[instrument(skip(self))]
    pub async fn find_by_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<Device>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device WHERE tenant_id = $1 AND name = $2
            "#,
            tenant_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Device {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            customer_id: r.customer_id,
            device_profile_id: r.device_profile_id,
            name: r.name,
            device_type: r.device_type.unwrap_or_default(),
            label: r.label,
            device_data: r.device_data,
            firmware_id: r.firmware_id,
            software_id: r.software_id,
            external_id: r.external_id,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    /// POST /api/customer/{customerId}/device/{deviceId} — gán device cho customer
    #[instrument(skip(self))]
    pub async fn assign_to_customer(&self, device_id: Uuid, customer_id: Uuid) -> Result<Device, DaoError> {
        sqlx::query!(
            "UPDATE device SET customer_id = $1 WHERE id = $2",
            customer_id, device_id
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(device_id).await?.ok_or(DaoError::NotFound)
    }

    /// DELETE /api/customer/device/{deviceId} — bỏ gán customer khỏi device
    #[instrument(skip(self))]
    pub async fn unassign_from_customer(&self, device_id: Uuid) -> Result<Device, DaoError> {
        sqlx::query!(
            "UPDATE device SET customer_id = NULL WHERE id = $1",
            device_id
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(device_id).await?.ok_or(DaoError::NotFound)
    }

    /// GET /api/device/types — danh sách device types của tenant
    #[instrument(skip(self))]
    pub async fn find_types_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query_scalar!(
            "SELECT DISTINCT type FROM device WHERE tenant_id = $1 AND type IS NOT NULL ORDER BY type",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;
        // type nullable → filter None
        Ok(rows.into_iter().flatten().collect())
    }

    /// GET /api/device/{deviceId}/activity — lấy lastActivityTime từ server attribute
    #[instrument(skip(self))]
    pub async fn find_last_activity_time(&self, device_id: Uuid) -> Result<Option<i64>, DaoError> {
        let val = sqlx::query_scalar!(
            r#"
            SELECT akv.long_v
            FROM attribute_kv akv
            JOIN key_dictionary k ON k.key_id = akv.attribute_key
            WHERE akv.entity_id = $1
              AND k.key = 'lastActivityTime'
              AND akv.attribute_type = 1
            "#,
            device_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(val.flatten())
    }

    /// Tìm device theo credentials_id (access token) — dùng cho MQTT auth
    #[instrument(skip(self))]
    pub async fn find_by_credentials_id(
        &self,
        credentials_id: &str,
    ) -> Result<Option<(Device, DeviceCredentials)>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT
                d.id, d.created_time, d.tenant_id, d.customer_id,
                d.device_profile_id, d.name, d.type as device_type, d.label,
                d.device_data, d.firmware_id, d.software_id,
                d.external_id, d.additional_info, d.version,
                dc.id as cred_id, dc.created_time as cred_created_time,
                dc.credentials_type, dc.credentials_id as cred_id_str,
                dc.credentials_value
            FROM device d
            JOIN device_credentials dc ON dc.device_id = d.id
            WHERE dc.credentials_id = $1
            "#,
            credentials_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let device = Device {
                id: r.id,
                created_time: r.created_time,
                tenant_id: r.tenant_id,
                customer_id: r.customer_id,
                device_profile_id: r.device_profile_id,
                name: r.name,
                device_type: r.device_type.unwrap_or_default(),
                label: r.label,
                device_data: r.device_data,
                firmware_id: r.firmware_id,
                software_id: r.software_id,
                external_id: r.external_id,
                additional_info: r.additional_info
                    .and_then(|s| serde_json::from_str(&s).ok()),
                version: r.version,
            };
            let creds = DeviceCredentials {
                id: r.cred_id,
                created_time: r.cred_created_time,
                device_id: device.id,
                credentials_type: parse_cred_type(&r.credentials_type),
                credentials_id: r.cred_id_str,
                credentials_value: r.credentials_value,
            };
            (device, creds)
        }))
    }

    /// Lưu claiming data cho device — tenant admin pre-provision device cho customer claiming
    #[instrument(skip(self))]
    pub async fn set_claiming_data(
        &self,
        device_id: Uuid,
        secret_key: &str,
        expiry_ts: i64,
    ) -> Result<(), DaoError> {
        let data = serde_json::json!({ "secretKey": secret_key });
        sqlx::query!(
            "UPDATE device SET claiming_data = $1, claim_expiry_ts = $2 WHERE id = $3",
            data,
            expiry_ts,
            device_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Customer claim device bằng tên device + secret key.
    /// Gán device về customer và xóa claiming data.
    #[instrument(skip(self))]
    pub async fn claim_device(
        &self,
        tenant_id:   Uuid,
        device_name: &str,
        secret_key:  &str,
        customer_id: Uuid,
        now_ms:      i64,
    ) -> Result<Device, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, claiming_data, claim_expiry_ts
               FROM device
               WHERE tenant_id = $1 AND name = $2"#,
            tenant_id,
            device_name
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DaoError::NotFound)?;

        // Kiểm tra expiry
        if let Some(exp) = row.claim_expiry_ts {
            if now_ms > exp {
                return Err(DaoError::Constraint("Claiming data has expired".into()));
            }
        }

        // Kiểm tra secret key
        let stored = row.claiming_data
            .as_ref()
            .and_then(|d| d.get("secretKey"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored != secret_key {
            return Err(DaoError::Constraint("Invalid claiming secret key".into()));
        }

        // Assign device → customer và clear claiming data
        sqlx::query!(
            "UPDATE device
             SET customer_id = $1, claiming_data = NULL, claim_expiry_ts = NULL
             WHERE id = $2",
            customer_id,
            row.id
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(row.id).await?.ok_or(DaoError::NotFound)
    }

    /// Xóa claim — trả device về trạng thái unassigned
    #[instrument(skip(self))]
    pub async fn reclaim_device(
        &self,
        tenant_id:   Uuid,
        device_name: &str,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE device
             SET customer_id = NULL, claiming_data = NULL, claim_expiry_ts = NULL
             WHERE tenant_id = $1 AND name = $2",
            tenant_id,
            device_name
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── P16: LoRaWAN ─────────────────────────────────────────────────────────

    /// Find a device by its LoRaWAN Device EUI (set via `set_lora_dev_eui`).
    #[instrument(skip(self))]
    pub async fn find_by_lora_dev_eui(&self, dev_eui: &str) -> Result<Option<Device>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id,
                   device_profile_id, name, type as device_type, label,
                   device_data, firmware_id, software_id,
                   external_id, additional_info, version
            FROM device
            WHERE lora_dev_eui = $1
            "#,
            dev_eui
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Device {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            customer_id:       r.customer_id,
            device_profile_id: r.device_profile_id,
            name:              r.name,
            device_type:       r.device_type.unwrap_or_default(),
            label:             r.label,
            device_data:       r.device_data,
            firmware_id:       r.firmware_id,
            software_id:       r.software_id,
            external_id:       r.external_id,
            additional_info:   r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version:           r.version,
        }))
    }

    /// Link a device to a LoRaWAN Device EUI. Pass `None` to unlink.
    #[instrument(skip(self))]
    pub async fn set_lora_dev_eui(
        &self,
        device_id: Uuid,
        dev_eui:   Option<&str>,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE device SET lora_dev_eui = $1 WHERE id = $2",
            dev_eui,
            device_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get the LoRaWAN dev_eui currently linked to a device (if any).
    #[instrument(skip(self))]
    pub async fn get_lora_dev_eui(&self, device_id: Uuid) -> Result<Option<String>, DaoError> {
        let val = sqlx::query_scalar!(
            "SELECT lora_dev_eui FROM device WHERE id = $1",
            device_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(val.flatten())
    }
}

fn parse_cred_type(s: &str) -> DeviceCredentialsType {
    match s {
        "ACCESS_TOKEN"      => DeviceCredentialsType::AccessToken,
        "X509_CERTIFICATE"  => DeviceCredentialsType::X509Certificate,
        "MQTT_BASIC"        => DeviceCredentialsType::MqttBasic,
        "LWM2M_CREDENTIALS" => DeviceCredentialsType::Lwm2mCredentials,
        _                   => DeviceCredentialsType::AccessToken,
    }
}

fn cred_type_str(t: &DeviceCredentialsType) -> &'static str {
    match t {
        DeviceCredentialsType::AccessToken      => "ACCESS_TOKEN",
        DeviceCredentialsType::X509Certificate  => "X509_CERTIFICATE",
        DeviceCredentialsType::MqttBasic        => "MQTT_BASIC",
        DeviceCredentialsType::Lwm2mCredentials => "LWM2M_CREDENTIALS",
    }
}
