use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::AiModel;

use crate::{DaoError, PageData, PageLink};

pub struct AiModelDao {
    pool: PgPool,
}

impl AiModelDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map(r: AiModelRow) -> AiModel {
        AiModel {
            id:              r.id,
            created_time:    r.created_time,
            tenant_id:       r.tenant_id,
            name:            r.name,
            configuration:   r.configuration,
            additional_info: r.additional_info,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AiModel>, DaoError> {
        let row = sqlx::query_as!(
            AiModelRow,
            "SELECT id, created_time, tenant_id, name, configuration, additional_info
             FROM ai_model WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Self::map))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Option<Uuid>,
        page: &PageLink,
    ) -> Result<PageData<AiModel>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM ai_model
             WHERE ($1::uuid IS NULL AND tenant_id IS NULL) OR tenant_id = $1",
            tenant_id as Option<Uuid>
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            AiModelRow,
            "SELECT id, created_time, tenant_id, name, configuration, additional_info
             FROM ai_model
             WHERE ($1::uuid IS NULL AND tenant_id IS NULL) OR tenant_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            tenant_id as Option<Uuid>,
            page.page_size,
            page.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PageData::new(rows.into_iter().map(Self::map).collect(), total, page))
    }

    #[instrument(skip(self, model))]
    pub async fn save(&self, model: &AiModel) -> Result<AiModel, DaoError> {
        sqlx::query!(
            "INSERT INTO ai_model (id, created_time, tenant_id, name, configuration, additional_info)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
                 name            = EXCLUDED.name,
                 configuration   = EXCLUDED.configuration,
                 additional_info = EXCLUDED.additional_info",
            model.id,
            model.created_time,
            model.tenant_id as Option<Uuid>,
            &model.name,
            model.configuration.as_ref() as Option<&serde_json::Value>,
            model.additional_info.as_ref() as Option<&serde_json::Value>,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(model.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let r = sqlx::query!("DELETE FROM ai_model WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 { return Err(DaoError::NotFound); }
        Ok(())
    }
}

struct AiModelRow {
    id:              Uuid,
    created_time:    i64,
    tenant_id:       Option<Uuid>,
    name:            String,
    configuration:   Option<serde_json::Value>,
    additional_info: Option<serde_json::Value>,
}
