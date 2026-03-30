use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use vl_core::entities::calculated_field::{CalculatedField, CreateCalculatedFieldRequest};

use crate::error::DaoError;

pub struct CalculatedFieldDao {
    pool: PgPool,
}

impl CalculatedFieldDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn save(
        &self,
        tenant_id: Uuid,
        req: &CreateCalculatedFieldRequest,
    ) -> Result<CalculatedField, DaoError> {
        let now = Utc::now().timestamp_millis();
        let trigger_mode = req.trigger_mode.as_deref().unwrap_or("ANY_CHANGE");
        let enabled = req.enabled.unwrap_or(true);

        let row = sqlx::query!(
            r#"INSERT INTO calculated_field
               (tenant_id, entity_id, entity_type, name, expression, output_key,
                input_keys, trigger_mode, output_ttl_ms, enabled, created_time)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               RETURNING id, tenant_id, entity_id, entity_type, name, expression,
                         output_key, input_keys, trigger_mode, output_ttl_ms, enabled, created_time"#,
            tenant_id,
            req.entity_id,
            req.entity_type,
            req.name,
            req.expression,
            req.output_key,
            &req.input_keys as &[String],
            trigger_mode,
            req.output_ttl_ms,
            enabled,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(CalculatedField {
            id: row.id,
            tenant_id: row.tenant_id,
            entity_id: row.entity_id,
            entity_type: row.entity_type,
            name: row.name,
            expression: row.expression,
            output_key: row.output_key,
            input_keys: row.input_keys,
            trigger_mode: row.trigger_mode,
            output_ttl_ms: row.output_ttl_ms,
            enabled: row.enabled,
            created_time: row.created_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<CalculatedField>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, entity_id, entity_type, name, expression,
                      output_key, input_keys, trigger_mode, output_ttl_ms, enabled, created_time
               FROM calculated_field WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CalculatedField {
            id: r.id,
            tenant_id: r.tenant_id,
            entity_id: r.entity_id,
            entity_type: r.entity_type,
            name: r.name,
            expression: r.expression,
            output_key: r.output_key,
            input_keys: r.input_keys,
            trigger_mode: r.trigger_mode,
            output_ttl_ms: r.output_ttl_ms,
            enabled: r.enabled,
            created_time: r.created_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_entity(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<CalculatedField>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, entity_id, entity_type, name, expression,
                      output_key, input_keys, trigger_mode, output_ttl_ms, enabled, created_time
               FROM calculated_field WHERE entity_id = $1 ORDER BY created_time"#,
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CalculatedField {
                id: r.id,
                tenant_id: r.tenant_id,
                entity_id: r.entity_id,
                entity_type: r.entity_type,
                name: r.name,
                expression: r.expression,
                output_key: r.output_key,
                input_keys: r.input_keys,
                trigger_mode: r.trigger_mode,
                output_ttl_ms: r.output_ttl_ms,
                enabled: r.enabled,
                created_time: r.created_time,
            })
            .collect())
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CalculatedField>, i64), DaoError> {
        let offset = page * page_size;

        let total = sqlx::query_scalar!(
            "SELECT count(*) FROM calculated_field WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"SELECT id, tenant_id, entity_id, entity_type, name, expression,
                      output_key, input_keys, trigger_mode, output_ttl_ms, enabled, created_time
               FROM calculated_field WHERE tenant_id = $1
               ORDER BY created_time LIMIT $2 OFFSET $3"#,
            tenant_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows
            .into_iter()
            .map(|r| CalculatedField {
                id: r.id,
                tenant_id: r.tenant_id,
                entity_id: r.entity_id,
                entity_type: r.entity_type,
                name: r.name,
                expression: r.expression,
                output_key: r.output_key,
                input_keys: r.input_keys,
                trigger_mode: r.trigger_mode,
                output_ttl_ms: r.output_ttl_ms,
                enabled: r.enabled,
                created_time: r.created_time,
            })
            .collect();

        Ok((data, total))
    }

    #[instrument(skip(self))]
    pub async fn update(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        req: &CreateCalculatedFieldRequest,
    ) -> Result<CalculatedField, DaoError> {
        let trigger_mode = req.trigger_mode.as_deref().unwrap_or("ANY_CHANGE");
        let enabled = req.enabled.unwrap_or(true);

        let row = sqlx::query!(
            r#"UPDATE calculated_field
               SET name=$1, expression=$2, output_key=$3, input_keys=$4,
                   trigger_mode=$5, output_ttl_ms=$6, enabled=$7
               WHERE id=$8 AND tenant_id=$9
               RETURNING id, tenant_id, entity_id, entity_type, name, expression,
                         output_key, input_keys, trigger_mode, output_ttl_ms, enabled, created_time"#,
            req.name,
            req.expression,
            req.output_key,
            &req.input_keys as &[String],
            trigger_mode,
            req.output_ttl_ms,
            enabled,
            id,
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DaoError::NotFound,
            other => DaoError::Database(other),
        })?;

        Ok(CalculatedField {
            id: row.id,
            tenant_id: row.tenant_id,
            entity_id: row.entity_id,
            entity_type: row.entity_type,
            name: row.name,
            expression: row.expression,
            output_key: row.output_key,
            input_keys: row.input_keys,
            trigger_mode: row.trigger_mode,
            output_ttl_ms: row.output_ttl_ms,
            enabled: row.enabled,
            created_time: row.created_time,
        })
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<(), DaoError> {
        let result =
            sqlx::query!("DELETE FROM calculated_field WHERE id=$1 AND tenant_id=$2", id, tenant_id)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}
