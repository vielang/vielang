use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::DomainEntry;

use crate::{DaoError, PageData, PageLink};

pub struct DomainDao {
    pool: PgPool,
}

impl DomainDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map(r: DomainRow) -> DomainEntry {
        DomainEntry {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            oauth2_enabled:    r.oauth2_enabled,
            propagate_to_edge: r.propagate_to_edge,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DomainEntry>, DaoError> {
        let row = sqlx::query_as!(
            DomainRow,
            "SELECT id, created_time, tenant_id, name, oauth2_enabled, propagate_to_edge
             FROM domain WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Self::map))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page: &PageLink,
    ) -> Result<PageData<DomainEntry>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM domain WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            DomainRow,
            "SELECT id, created_time, tenant_id, name, oauth2_enabled, propagate_to_edge
             FROM domain
             WHERE tenant_id = $1
             ORDER BY created_time DESC
             LIMIT $2 OFFSET $3",
            tenant_id,
            page.page_size,
            page.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PageData::new(rows.into_iter().map(Self::map).collect(), total, page))
    }

    #[instrument(skip(self, d))]
    pub async fn save(&self, d: &DomainEntry) -> Result<DomainEntry, DaoError> {
        sqlx::query!(
            "INSERT INTO domain (id, created_time, tenant_id, name, oauth2_enabled, propagate_to_edge)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
                 name              = EXCLUDED.name,
                 oauth2_enabled    = EXCLUDED.oauth2_enabled,
                 propagate_to_edge = EXCLUDED.propagate_to_edge",
            d.id,
            d.created_time,
            d.tenant_id,
            d.name,
            d.oauth2_enabled,
            d.propagate_to_edge,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(d.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let r = sqlx::query!("DELETE FROM domain WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 { return Err(DaoError::NotFound); }
        Ok(())
    }

    /// Gán danh sách oauth2_client_id vào domain (replace toàn bộ)
    #[instrument(skip(self, client_ids))]
    pub async fn set_oauth2_clients(
        &self,
        domain_id: Uuid,
        client_ids: &[Uuid],
    ) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM domain_oauth2_client WHERE domain_id = $1",
            domain_id
        )
        .execute(&self.pool)
        .await?;

        for &client_id in client_ids {
            sqlx::query!(
                "INSERT INTO domain_oauth2_client (domain_id, oauth2_client_id)
                 VALUES ($1, $2) ON CONFLICT DO NOTHING",
                domain_id,
                client_id
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Lấy danh sách oauth2_client_id đã gán cho domain
    #[instrument(skip(self))]
    pub async fn get_oauth2_clients(
        &self,
        domain_id: Uuid,
    ) -> Result<Vec<Uuid>, DaoError> {
        let rows = sqlx::query!(
            "SELECT oauth2_client_id FROM domain_oauth2_client WHERE domain_id = $1",
            domain_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.oauth2_client_id).collect())
    }
}

struct DomainRow {
    id:                Uuid,
    created_time:      i64,
    tenant_id:         Uuid,
    name:              String,
    oauth2_enabled:    bool,
    propagate_to_edge: bool,
}
