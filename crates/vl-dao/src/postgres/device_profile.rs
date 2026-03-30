use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{DeviceProfile, DeviceProfileType, DeviceTransportType, DeviceProvisionType};
use crate::{DaoError, PageData, PageLink};

pub struct DeviceProfileDao {
    pool: PgPool,
}

impl DeviceProfileDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: &DeviceProfileRow) -> DeviceProfile {
        DeviceProfile {
            id:                        r.id,
            created_time:              r.created_time,
            tenant_id:                 r.tenant_id,
            name:                      r.name.clone(),
            description:               r.description.clone(),
            image:                     r.image.clone(),
            is_default:                r.is_default,
            device_profile_type:       parse_profile_type(r.profile_type.as_deref().unwrap_or("DEFAULT")),
            transport_type:            parse_transport_type(r.transport_type.as_deref().unwrap_or("DEFAULT")),
            provision_type:            parse_provision_type(r.provision_type.as_deref().unwrap_or("DISABLED")),
            profile_data:              r.profile_data.clone(),
            default_rule_chain_id:     r.default_rule_chain_id,
            default_dashboard_id:      r.default_dashboard_id,
            default_queue_name:        r.default_queue_name.clone(),
            default_edge_rule_chain_id: r.default_edge_rule_chain_id,
            provision_device_key:      r.provision_device_key.clone(),
            firmware_id:               r.firmware_id,
            software_id:               r.software_id,
            external_id:               r.external_id,
            version:                   r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DeviceProfile>, DaoError> {
        let row = sqlx::query_as!(
            DeviceProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default,
                   type        AS "profile_type: _",
                   transport_type AS "transport_type: _",
                   provision_type AS "provision_type: _",
                   profile_data, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   provision_device_key, firmware_id, software_id,
                   external_id, version
            FROM device_profile WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_default(&self, tenant_id: Uuid) -> Result<Option<DeviceProfile>, DaoError> {
        let row = sqlx::query_as!(
            DeviceProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default,
                   type        AS "profile_type: _",
                   transport_type AS "transport_type: _",
                   provision_type AS "provision_type: _",
                   profile_data, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   provision_device_key, firmware_id, software_id,
                   external_id, version
            FROM device_profile WHERE tenant_id = $1 AND is_default = TRUE
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<DeviceProfile>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM device_profile
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            DeviceProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default,
                   type        AS "profile_type: _",
                   transport_type AS "transport_type: _",
                   provision_type AS "provision_type: _",
                   profile_data, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   provision_device_key, firmware_id, software_id,
                   external_id, version
            FROM device_profile
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

        let data = rows.iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// Lấy danh sách tên profile (id + name) theo tenant
    #[instrument(skip(self))]
    pub async fn find_names_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<(Uuid, String)>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id, name FROM device_profile WHERE tenant_id = $1 ORDER BY name",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.name)).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, profile: &DeviceProfile) -> Result<DeviceProfile, DaoError> {
        let profile_type = format!("{:?}", profile.device_profile_type).to_uppercase();
        let transport_type = format!("{:?}", profile.transport_type).to_uppercase();
        let provision_type = format!("{:?}", profile.provision_type).to_uppercase();

        sqlx::query!(
            r#"
            INSERT INTO device_profile (
                id, created_time, tenant_id, name, description, image,
                is_default, type, transport_type, provision_type, profile_data,
                default_rule_chain_id, default_dashboard_id, default_queue_name,
                default_edge_rule_chain_id, provision_device_key,
                firmware_id, software_id, external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
            ON CONFLICT (id) DO UPDATE SET
                name                     = EXCLUDED.name,
                description              = EXCLUDED.description,
                image                    = EXCLUDED.image,
                is_default               = EXCLUDED.is_default,
                type                     = EXCLUDED.type,
                transport_type           = EXCLUDED.transport_type,
                provision_type           = EXCLUDED.provision_type,
                profile_data             = EXCLUDED.profile_data,
                default_rule_chain_id    = EXCLUDED.default_rule_chain_id,
                default_dashboard_id     = EXCLUDED.default_dashboard_id,
                default_queue_name       = EXCLUDED.default_queue_name,
                default_edge_rule_chain_id = EXCLUDED.default_edge_rule_chain_id,
                provision_device_key     = EXCLUDED.provision_device_key,
                firmware_id              = EXCLUDED.firmware_id,
                software_id              = EXCLUDED.software_id,
                external_id              = EXCLUDED.external_id,
                version                  = device_profile.version + 1
            "#,
            profile.id,
            profile.created_time,
            profile.tenant_id,
            profile.name,
            profile.description,
            profile.image,
            profile.is_default,
            profile_type,
            transport_type,
            provision_type,
            profile.profile_data,
            profile.default_rule_chain_id,
            profile.default_dashboard_id,
            profile.default_queue_name,
            profile.default_edge_rule_chain_id,
            profile.provision_device_key,
            profile.firmware_id,
            profile.software_id,
            profile.external_id,
            profile.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(profile.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn set_default(&self, tenant_id: Uuid, profile_id: Uuid) -> Result<(), DaoError> {
        // Bỏ default cũ
        sqlx::query!(
            "UPDATE device_profile SET is_default = FALSE WHERE tenant_id = $1 AND is_default = TRUE",
            tenant_id
        )
        .execute(&self.pool)
        .await?;

        // Đặt default mới
        sqlx::query!(
            "UPDATE device_profile SET is_default = TRUE WHERE id = $1",
            profile_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    /// Find device profile by provision_device_key (used during device provisioning).
    #[instrument(skip(self))]
    pub async fn find_by_provision_key(
        &self,
        provision_key: &str,
    ) -> Result<Option<DeviceProfile>, DaoError> {
        let row = sqlx::query_as!(
            DeviceProfileRow,
            r#"
            SELECT id, created_time, tenant_id, name, description, image,
                   is_default,
                   type        AS "profile_type: _",
                   transport_type AS "transport_type: _",
                   provision_type AS "provision_type: _",
                   profile_data, default_rule_chain_id, default_dashboard_id,
                   default_queue_name, default_edge_rule_chain_id,
                   provision_device_key, firmware_id, software_id,
                   external_id, version
            FROM device_profile WHERE provision_device_key = $1
            "#,
            provision_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM device_profile WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct DeviceProfileRow {
    id:                        Uuid,
    created_time:              i64,
    tenant_id:                 Uuid,
    name:                      String,
    description:               Option<String>,
    image:                     Option<String>,
    is_default:                bool,
    profile_type:              Option<String>,
    transport_type:            Option<String>,
    provision_type:            Option<String>,
    profile_data:              Option<serde_json::Value>,
    default_rule_chain_id:     Option<Uuid>,
    default_dashboard_id:      Option<Uuid>,
    default_queue_name:        Option<String>,
    default_edge_rule_chain_id: Option<Uuid>,
    provision_device_key:      Option<String>,
    firmware_id:               Option<Uuid>,
    software_id:               Option<Uuid>,
    external_id:               Option<Uuid>,
    version:                   i64,
}

fn parse_profile_type(s: &str) -> DeviceProfileType {
    match s {
        _ => DeviceProfileType::Default,
    }
}

fn parse_transport_type(s: &str) -> DeviceTransportType {
    match s {
        "MQTT"    => DeviceTransportType::Mqtt,
        "COAP"    => DeviceTransportType::Coap,
        "LWM2M"   => DeviceTransportType::Lwm2m,
        "SNMP"    => DeviceTransportType::Snmp,
        _         => DeviceTransportType::Default,
    }
}

fn parse_provision_type(s: &str) -> DeviceProvisionType {
    match s {
        "ALLOW_CREATE_NEW_DEVICES"      => DeviceProvisionType::AllowCreateNewDevices,
        "CHECK_PRE_PROVISIONED_DEVICES" => DeviceProvisionType::CheckPreProvisionedDevices,
        "X509_CERTIFICATE_CHAIN"        => DeviceProvisionType::X509CertificateChain,
        _                               => DeviceProvisionType::Disabled,
    }
}
