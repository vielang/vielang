use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use vl_core::entities::simulator::{CreateDeviceTemplateRequest, DeviceTemplate};

use crate::error::DaoError;

pub struct DeviceTemplateDao {
    pool: PgPool,
}

impl DeviceTemplateDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DeviceTemplate>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, name, description, category,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      diagram as "diagram: serde_json::Value",
                      is_builtin, tenant_id, created_time
               FROM device_template WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DeviceTemplate {
            id: r.id,
            name: r.name,
            description: r.description,
            category: r.category,
            telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
            diagram: r.diagram,
            is_builtin: r.is_builtin,
            tenant_id: r.tenant_id,
            created_time: r.created_time,
        }))
    }

    /// List all templates visible to a tenant: builtin + tenant-owned.
    #[instrument(skip(self))]
    pub async fn find_all_for_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<DeviceTemplate>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, name, description, category,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      diagram as "diagram: serde_json::Value",
                      is_builtin, tenant_id, created_time
               FROM device_template
               WHERE is_builtin = true OR tenant_id = $1
               ORDER BY is_builtin DESC, name"#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DeviceTemplate {
                id: r.id,
                name: r.name,
                description: r.description,
                category: r.category,
                telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
                diagram: r.diagram,
                is_builtin: r.is_builtin,
                tenant_id: r.tenant_id,
                created_time: r.created_time,
            })
            .collect())
    }

    #[instrument(skip(self, req))]
    pub async fn insert(
        &self,
        tenant_id: Uuid,
        req: &CreateDeviceTemplateRequest,
    ) -> Result<DeviceTemplate, DaoError> {
        let now = Utc::now().timestamp_millis();
        let schema_json =
            serde_json::to_value(&req.telemetry_schema).map_err(DaoError::Serialization)?;

        let row = sqlx::query!(
            r#"INSERT INTO device_template
               (name, description, category, telemetry_schema, diagram, is_builtin, tenant_id, created_time)
               VALUES ($1,$2,$3,$4,$5,false,$6,$7)
               RETURNING id, name, description, category,
                         telemetry_schema as "telemetry_schema: serde_json::Value",
                         diagram as "diagram: serde_json::Value",
                         is_builtin, tenant_id, created_time"#,
            req.name,
            req.description,
            req.category,
            schema_json,
            req.diagram,
            tenant_id,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(DeviceTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            category: row.category,
            telemetry_schema: serde_json::from_value(row.telemetry_schema).unwrap_or_default(),
            diagram: row.diagram,
            is_builtin: row.is_builtin,
            tenant_id: row.tenant_id,
            created_time: row.created_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!(
            "DELETE FROM device_template WHERE id=$1 AND is_builtin = false",
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}
