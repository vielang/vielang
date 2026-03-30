use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Edge, EdgeEvent, EdgeInfo};
use crate::{DaoError, PageData, PageLink};

pub struct EdgeDao {
    pool: PgPool,
}

impl EdgeDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: EdgeRow) -> Edge {
        Edge {
            id:                 r.id,
            created_time:       r.created_time,
            tenant_id:          r.tenant_id,
            customer_id:        r.customer_id,
            root_rule_chain_id: r.root_rule_chain_id,
            name:               r.name,
            edge_type:          r.edge_type,
            label:              r.label,
            routing_key:        r.routing_key,
            secret:             r.secret,
            additional_info:    r.additional_info,
            external_id:        r.external_id,
            version:            r.version,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Edge>, DaoError> {
        let row = sqlx::query_as!(
            EdgeRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id, root_rule_chain_id,
                   name, type AS edge_type, label, routing_key, secret,
                   additional_info, external_id, version
            FROM edge WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_routing_key(&self, routing_key: &str) -> Result<Option<Edge>, DaoError> {
        let row = sqlx::query_as!(
            EdgeRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id, root_rule_chain_id,
                   name, type AS edge_type, label, routing_key, secret,
                   additional_info, external_id, version
            FROM edge WHERE routing_key = $1
            "#,
            routing_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant_and_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<Option<Edge>, DaoError> {
        let row = sqlx::query_as!(
            EdgeRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id, root_rule_chain_id,
                   name, type AS edge_type, label, routing_key, secret,
                   additional_info, external_id, version
            FROM edge WHERE tenant_id = $1 AND name = $2
            "#,
            tenant_id, name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Self::map_row))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        edge_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<Edge>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM edge
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR type = $2)
               AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))"#,
            tenant_id, edge_type, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            EdgeRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id, root_rule_chain_id,
                   name, type AS edge_type, label, routing_key, secret,
                   additional_info, external_id, version
            FROM edge
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR type = $2)
            AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))
            ORDER BY created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id, edge_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_infos_by_tenant(
        &self,
        tenant_id: Uuid,
        edge_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<EdgeInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM edge
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR type = $2)
               AND ($3::text IS NULL OR LOWER(name) LIKE LOWER($3))"#,
            tenant_id, edge_type, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT e.id, e.created_time, e.tenant_id, e.customer_id,
                   e.root_rule_chain_id, e.name, e.type AS edge_type,
                   e.label, e.routing_key, e.secret,
                   e.additional_info, e.external_id, e.version,
                   c.title AS "customer_title: Option<String>",
                   c.is_public AS "customer_is_public: Option<bool>"
            FROM edge e
            LEFT JOIN customer c ON c.id = e.customer_id
            WHERE e.tenant_id = $1
            AND ($2::text IS NULL OR e.type = $2)
            AND ($3::text IS NULL OR LOWER(e.name) LIKE LOWER($3))
            ORDER BY e.created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id, edge_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| EdgeInfo {
            edge: Edge {
                id:                 r.id,
                created_time:       r.created_time,
                tenant_id:          r.tenant_id,
                customer_id:        r.customer_id,
                root_rule_chain_id: r.root_rule_chain_id,
                name:               r.name,
                edge_type:          r.edge_type,
                label:              r.label,
                routing_key:        r.routing_key,
                secret:             r.secret,
                additional_info:    r.additional_info,
                external_id:        r.external_id,
                version:            r.version,
            },
            customer_title:     r.customer_title,
            customer_is_public: r.customer_is_public.unwrap_or(false),
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_by_customer(
        &self,
        tenant_id: Uuid,
        customer_id: Uuid,
        edge_type: Option<&str>,
        page_link: &PageLink,
    ) -> Result<PageData<Edge>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM edge
               WHERE tenant_id = $1 AND customer_id = $2
               AND ($3::text IS NULL OR type = $3)
               AND ($4::text IS NULL OR LOWER(name) LIKE LOWER($4))"#,
            tenant_id, customer_id, edge_type, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            EdgeRow,
            r#"
            SELECT id, created_time, tenant_id, customer_id, root_rule_chain_id,
                   name, type AS edge_type, label, routing_key, secret,
                   additional_info, external_id, version
            FROM edge
            WHERE tenant_id = $1 AND customer_id = $2
            AND ($3::text IS NULL OR type = $3)
            AND ($4::text IS NULL OR LOWER(name) LIKE LOWER($4))
            ORDER BY created_time DESC
            LIMIT $5 OFFSET $6
            "#,
            tenant_id, customer_id, edge_type, text_search,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_types_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<String>, DaoError> {
        let rows = sqlx::query!(
            "SELECT DISTINCT type FROM edge WHERE tenant_id = $1 ORDER BY type",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.r#type).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, edge: &Edge) -> Result<Edge, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO edge (
                id, created_time, tenant_id, customer_id, root_rule_chain_id,
                name, type, label, routing_key, secret,
                additional_info, external_id, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            ON CONFLICT (id) DO UPDATE SET
                name               = EXCLUDED.name,
                type               = EXCLUDED.type,
                label              = EXCLUDED.label,
                customer_id        = EXCLUDED.customer_id,
                root_rule_chain_id = EXCLUDED.root_rule_chain_id,
                additional_info    = EXCLUDED.additional_info,
                external_id        = EXCLUDED.external_id,
                version            = edge.version + 1
            "#,
            edge.id,
            edge.created_time,
            edge.tenant_id,
            edge.customer_id,
            edge.root_rule_chain_id,
            edge.name,
            edge.edge_type,
            edge.label,
            edge.routing_key,
            edge.secret,
            edge.additional_info,
            edge.external_id,
            edge.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(edge.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn assign_to_customer(
        &self,
        edge_id: Uuid,
        customer_id: Uuid,
    ) -> Result<Edge, DaoError> {
        sqlx::query!(
            "UPDATE edge SET customer_id = $1, version = version + 1 WHERE id = $2",
            customer_id, edge_id
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(edge_id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn unassign_from_customer(&self, edge_id: Uuid) -> Result<Edge, DaoError> {
        sqlx::query!(
            "UPDATE edge SET customer_id = NULL, version = version + 1 WHERE id = $1",
            edge_id
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(edge_id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn set_root_rule_chain(
        &self,
        edge_id: Uuid,
        rule_chain_id: Uuid,
    ) -> Result<Edge, DaoError> {
        sqlx::query!(
            "UPDATE edge SET root_rule_chain_id = $1, version = version + 1 WHERE id = $2",
            rule_chain_id, edge_id
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(edge_id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM edge WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }
}

pub struct EdgeEventDao {
    pool: PgPool,
}

impl EdgeEventDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map_row(r: EdgeEventRow) -> EdgeEvent {
        EdgeEvent {
            id:                 r.id,
            created_time:       r.created_time,
            seq_id:             r.seq_id,
            tenant_id:          r.tenant_id,
            edge_id:            r.edge_id,
            edge_event_type:    r.edge_event_type,
            edge_event_action:  r.edge_event_action,
            entity_id:          r.entity_id,
            body:               r.body,
            uid:                r.uid,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_edge(
        &self,
        edge_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<EdgeEvent>, DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM edge_event WHERE edge_id = $1",
            edge_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            EdgeEventRow,
            r#"
            SELECT id, created_time, seq_id, tenant_id, edge_id,
                   edge_event_type, edge_event_action, entity_id, body, uid
            FROM edge_event
            WHERE edge_id = $1
            ORDER BY seq_id DESC
            LIMIT $2 OFFSET $3
            "#,
            edge_id,
            page_link.page_size, page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(Self::map_row).collect();
        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, event: &EdgeEvent) -> Result<EdgeEvent, DaoError> {
        let row = sqlx::query_as!(
            EdgeEventRow,
            r#"
            INSERT INTO edge_event (
                id, created_time, tenant_id, edge_id,
                edge_event_type, edge_event_action, entity_id, body, uid
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id, created_time, seq_id, tenant_id, edge_id,
                      edge_event_type, edge_event_action, entity_id, body, uid
            "#,
            event.id,
            event.created_time,
            event.tenant_id,
            event.edge_id,
            event.edge_event_type,
            event.edge_event_action,
            event.entity_id,
            event.body,
            event.uid,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(Self::map_row(row))
    }
}

// ── Internal query structs ────────────────────────────────────────────────────

struct EdgeRow {
    id:                 Uuid,
    created_time:       i64,
    tenant_id:          Uuid,
    customer_id:        Option<Uuid>,
    root_rule_chain_id: Option<Uuid>,
    name:               String,
    edge_type:          String,
    label:              Option<String>,
    routing_key:        String,
    secret:             String,
    additional_info:    Option<serde_json::Value>,
    external_id:        Option<Uuid>,
    version:            i64,
}

struct EdgeEventRow {
    id:                 Uuid,
    created_time:       i64,
    seq_id:             i64,
    tenant_id:          Uuid,
    edge_id:            Uuid,
    edge_event_type:    String,
    edge_event_action:  String,
    entity_id:          Option<Uuid>,
    body:               Option<serde_json::Value>,
    uid:                Option<String>,
}
