use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use vl_core::entities::simulator::{
    CreateSimulatorRequest, CreateSchematicRequest, SaveSchematicNodeRequest,
    SchematicNodeConfig, SimulatorConfig, SimulatorSchematic,
};

use crate::error::DaoError;

pub struct SimulatorDao {
    pool: PgPool,
}

impl SimulatorDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SimulatorConfig>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, device_id, name, enabled, interval_ms,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      script, transport_mode, created_time, updated_time
               FROM simulator_config WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SimulatorConfig {
            id: r.id,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            name: r.name,
            enabled: r.enabled,
            interval_ms: r.interval_ms,
            telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
            script: r.script,
            transport_mode: vl_core::entities::simulator::TransportMode::from_str(&r.transport_mode),
            created_time: r.created_time,
            updated_time: r.updated_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<SimulatorConfig>, i64), DaoError> {
        let offset = page * page_size;

        let total = sqlx::query_scalar!(
            "SELECT count(*) FROM simulator_config WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, device_id, name, enabled, interval_ms,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      script, transport_mode, created_time, updated_time
               FROM simulator_config WHERE tenant_id = $1
               ORDER BY created_time DESC LIMIT $2 OFFSET $3"#,
            tenant_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows
            .into_iter()
            .map(|r| SimulatorConfig {
                id: r.id,
                tenant_id: r.tenant_id,
                device_id: r.device_id,
                name: r.name,
                enabled: r.enabled,
                interval_ms: r.interval_ms,
                telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
                script: r.script,
                transport_mode: vl_core::entities::simulator::TransportMode::from_str(
                    &r.transport_mode,
                ),
                created_time: r.created_time,
                updated_time: r.updated_time,
            })
            .collect();

        Ok((data, total))
    }

    #[instrument(skip(self))]
    pub async fn find_by_device(&self, device_id: Uuid) -> Result<Vec<SimulatorConfig>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, device_id, name, enabled, interval_ms,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      script, transport_mode, created_time, updated_time
               FROM simulator_config WHERE device_id = $1
               ORDER BY created_time DESC"#,
            device_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SimulatorConfig {
                id: r.id,
                tenant_id: r.tenant_id,
                device_id: r.device_id,
                name: r.name,
                enabled: r.enabled,
                interval_ms: r.interval_ms,
                telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
                script: r.script,
                transport_mode: vl_core::entities::simulator::TransportMode::from_str(
                    &r.transport_mode,
                ),
                created_time: r.created_time,
                updated_time: r.updated_time,
            })
            .collect())
    }

    #[instrument(skip(self))]
    pub async fn find_enabled(&self) -> Result<Vec<SimulatorConfig>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, device_id, name, enabled, interval_ms,
                      telemetry_schema as "telemetry_schema: serde_json::Value",
                      script, transport_mode, created_time, updated_time
               FROM simulator_config WHERE enabled = true
               ORDER BY created_time"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SimulatorConfig {
                id: r.id,
                tenant_id: r.tenant_id,
                device_id: r.device_id,
                name: r.name,
                enabled: r.enabled,
                interval_ms: r.interval_ms,
                telemetry_schema: serde_json::from_value(r.telemetry_schema).unwrap_or_default(),
                script: r.script,
                transport_mode: vl_core::entities::simulator::TransportMode::from_str(
                    &r.transport_mode,
                ),
                created_time: r.created_time,
                updated_time: r.updated_time,
            })
            .collect())
    }

    #[instrument(skip(self, req))]
    pub async fn insert(
        &self,
        tenant_id: Uuid,
        req: &CreateSimulatorRequest,
    ) -> Result<SimulatorConfig, DaoError> {
        let now = Utc::now().timestamp_millis();
        let schema_json =
            serde_json::to_value(&req.telemetry_schema).map_err(DaoError::Serialization)?;

        let row = sqlx::query!(
            r#"INSERT INTO simulator_config
               (tenant_id, device_id, name, enabled, interval_ms, telemetry_schema, script,
                transport_mode, created_time, updated_time)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING id, tenant_id, device_id, name, enabled, interval_ms,
                         telemetry_schema as "telemetry_schema: serde_json::Value",
                         script, transport_mode, created_time, updated_time"#,
            tenant_id,
            req.device_id,
            req.name,
            req.enabled,
            req.interval_ms,
            schema_json,
            req.script,
            req.transport_mode.as_str(),
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(SimulatorConfig {
            id: row.id,
            tenant_id: row.tenant_id,
            device_id: row.device_id,
            name: row.name,
            enabled: row.enabled,
            interval_ms: row.interval_ms,
            telemetry_schema: serde_json::from_value(row.telemetry_schema).unwrap_or_default(),
            script: row.script,
            transport_mode: vl_core::entities::simulator::TransportMode::from_str(&row.transport_mode),
            created_time: row.created_time,
            updated_time: row.updated_time,
        })
    }

    #[instrument(skip(self, req))]
    pub async fn update(
        &self,
        id: Uuid,
        req: &CreateSimulatorRequest,
    ) -> Result<SimulatorConfig, DaoError> {
        let now = Utc::now().timestamp_millis();
        let schema_json =
            serde_json::to_value(&req.telemetry_schema).map_err(DaoError::Serialization)?;

        let row = sqlx::query!(
            r#"UPDATE simulator_config
               SET device_id=$1, name=$2, enabled=$3, interval_ms=$4,
                   telemetry_schema=$5, script=$6, transport_mode=$7, updated_time=$8
               WHERE id=$9
               RETURNING id, tenant_id, device_id, name, enabled, interval_ms,
                         telemetry_schema as "telemetry_schema: serde_json::Value",
                         script, transport_mode, created_time, updated_time"#,
            req.device_id,
            req.name,
            req.enabled,
            req.interval_ms,
            schema_json,
            req.script,
            req.transport_mode.as_str(),
            now,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DaoError::NotFound,
            other => DaoError::Database(other),
        })?;

        Ok(SimulatorConfig {
            id: row.id,
            tenant_id: row.tenant_id,
            device_id: row.device_id,
            name: row.name,
            enabled: row.enabled,
            interval_ms: row.interval_ms,
            telemetry_schema: serde_json::from_value(row.telemetry_schema).unwrap_or_default(),
            script: row.script,
            transport_mode: vl_core::entities::simulator::TransportMode::from_str(&row.transport_mode),
            created_time: row.created_time,
            updated_time: row.updated_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<(), DaoError> {
        let now = Utc::now().timestamp_millis();
        let result = sqlx::query!(
            "UPDATE simulator_config SET enabled=$1, updated_time=$2 WHERE id=$3",
            enabled,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM simulator_config WHERE id=$1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Lookup device access token (credentials_id) from device_credentials table
    #[instrument(skip(self))]
    pub async fn find_device_token(&self, device_id: Uuid) -> Result<Option<String>, DaoError> {
        let row = sqlx::query_scalar!(
            "SELECT credentials_id FROM device_credentials WHERE device_id = $1 AND credentials_type = 'ACCESS_TOKEN' LIMIT 1",
            device_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }
}

// ── Phase 4: Schematic DAO ───────────────────────────────────────────────────

pub struct SchematicDao {
    pool: PgPool,
}

impl SchematicDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SimulatorSchematic>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, name,
                      graph_data as "graph_data: serde_json::Value",
                      created_time, updated_time
               FROM simulator_schematic WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SimulatorSchematic {
            id: r.id,
            tenant_id: r.tenant_id,
            name: r.name,
            graph_data: r.graph_data,
            created_time: r.created_time,
            updated_time: r.updated_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<SimulatorSchematic>, i64), DaoError> {
        let offset = page * page_size;

        let total = sqlx::query_scalar!(
            "SELECT count(*) FROM simulator_schematic WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, name,
                      graph_data as "graph_data: serde_json::Value",
                      created_time, updated_time
               FROM simulator_schematic WHERE tenant_id = $1
               ORDER BY created_time DESC LIMIT $2 OFFSET $3"#,
            tenant_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows
            .into_iter()
            .map(|r| SimulatorSchematic {
                id: r.id,
                tenant_id: r.tenant_id,
                name: r.name,
                graph_data: r.graph_data,
                created_time: r.created_time,
                updated_time: r.updated_time,
            })
            .collect();

        Ok((data, total))
    }

    #[instrument(skip(self, req))]
    pub async fn insert(
        &self,
        tenant_id: Uuid,
        req: &CreateSchematicRequest,
    ) -> Result<SimulatorSchematic, DaoError> {
        let now = Utc::now().timestamp_millis();

        let row = sqlx::query!(
            r#"INSERT INTO simulator_schematic (tenant_id, name, graph_data, created_time, updated_time)
               VALUES ($1,$2,$3,$4,$5)
               RETURNING id, tenant_id, name,
                         graph_data as "graph_data: serde_json::Value",
                         created_time, updated_time"#,
            tenant_id,
            req.name,
            req.graph_data,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(SimulatorSchematic {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            graph_data: row.graph_data,
            created_time: row.created_time,
            updated_time: row.updated_time,
        })
    }

    #[instrument(skip(self, req))]
    pub async fn update(
        &self,
        id: Uuid,
        req: &CreateSchematicRequest,
    ) -> Result<SimulatorSchematic, DaoError> {
        let now = Utc::now().timestamp_millis();

        let row = sqlx::query!(
            r#"UPDATE simulator_schematic SET name=$1, graph_data=$2, updated_time=$3
               WHERE id=$4
               RETURNING id, tenant_id, name,
                         graph_data as "graph_data: serde_json::Value",
                         created_time, updated_time"#,
            req.name,
            req.graph_data,
            now,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DaoError::NotFound,
            other => DaoError::Database(other),
        })?;

        Ok(SimulatorSchematic {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            graph_data: row.graph_data,
            created_time: row.created_time,
            updated_time: row.updated_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM simulator_schematic WHERE id=$1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    // ── Schematic nodes ──────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn find_nodes(&self, schematic_id: Uuid) -> Result<Vec<SchematicNodeConfig>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, schematic_id, node_id, simulator_config_id, node_type,
                      properties as "properties: serde_json::Value"
               FROM schematic_node_config WHERE schematic_id = $1"#,
            schematic_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SchematicNodeConfig {
                id: r.id,
                schematic_id: r.schematic_id,
                node_id: r.node_id,
                simulator_config_id: r.simulator_config_id,
                node_type: r.node_type,
                properties: r.properties,
            })
            .collect())
    }

    #[instrument(skip(self, req))]
    pub async fn save_node(
        &self,
        schematic_id: Uuid,
        req: &SaveSchematicNodeRequest,
    ) -> Result<SchematicNodeConfig, DaoError> {
        let row = sqlx::query!(
            r#"INSERT INTO schematic_node_config
               (schematic_id, node_id, simulator_config_id, node_type, properties)
               VALUES ($1,$2,$3,$4,$5)
               ON CONFLICT (id) DO UPDATE SET
                 simulator_config_id = EXCLUDED.simulator_config_id,
                 node_type = EXCLUDED.node_type,
                 properties = EXCLUDED.properties
               RETURNING id, schematic_id, node_id, simulator_config_id, node_type,
                         properties as "properties: serde_json::Value""#,
            schematic_id,
            req.node_id,
            req.simulator_config_id,
            req.node_type,
            req.properties
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SchematicNodeConfig {
            id: row.id,
            schematic_id: row.schematic_id,
            node_id: row.node_id,
            simulator_config_id: row.simulator_config_id,
            node_type: row.node_type,
            properties: row.properties,
        })
    }

    #[instrument(skip(self))]
    pub async fn delete_node(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM schematic_node_config WHERE id=$1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}
