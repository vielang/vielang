use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::Customer;
use crate::{DaoError, PageData, PageLink};

pub struct CustomerDao {
    pool: PgPool,
}

fn map_customer(r: CustomerRow) -> Customer {
    Customer {
        id: r.id,
        created_time: r.created_time,
        tenant_id: r.tenant_id,
        title: r.title,
        country: r.country,
        state: r.state,
        city: r.city,
        address: r.address,
        address2: r.address2,
        zip: r.zip,
        phone: r.phone,
        email: r.email,
        is_public: r.is_public,
        external_id: r.external_id,
        additional_info: r.additional_info
            .and_then(|s| serde_json::from_str(&s).ok()),
        version: r.version,
    }
}

impl CustomerDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Customer>, DaoError> {
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, created_time, tenant_id, title, country, state, city,
                   address, address2, zip, phone, email, is_public,
                   external_id, additional_info, version
            FROM customer WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_customer))
    }

    #[instrument(skip(self))]
    pub async fn find_public_customer(&self, tenant_id: Uuid) -> Result<Option<Customer>, DaoError> {
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, created_time, tenant_id, title, country, state, city,
                   address, address2, zip, phone, email, is_public,
                   external_id, additional_info, version
            FROM customer WHERE tenant_id = $1 AND is_public = TRUE
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_customer))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<Customer>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM customer
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))"#,
            tenant_id, text_search
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, created_time, tenant_id, title, country, state, city,
                   address, address2, zip, phone, email, is_public,
                   external_id, additional_info, version
            FROM customer
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id, text_search, page_link.page_size, page_link.offset()
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(map_customer).collect();
        Ok(PageData::new(data, total, page_link))
    }

    /// Export all customers for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Customer>, DaoError> {
        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, created_time, tenant_id, title, country, state, city,
                   address, address2, zip, phone, email, is_public,
                   external_id, additional_info, version
            FROM customer WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(map_customer).collect())
    }

    #[instrument(skip(self))]
    pub async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, DaoError> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM customer WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    #[instrument(skip(self))]
    pub async fn find_title_by_id(&self, id: Uuid) -> Result<Option<String>, DaoError> {
        let title = sqlx::query_scalar!(
            r#"SELECT title FROM customer WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(title)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(r#"DELETE FROM customer WHERE id = $1"#, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, customer: &Customer) -> Result<Customer, DaoError> {
        let additional_info = customer.additional_info.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO customer (
                id, created_time, tenant_id, title, country, state, city,
                address, address2, zip, phone, email, is_public,
                external_id, additional_info, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (id) DO UPDATE SET
                title           = EXCLUDED.title,
                country         = EXCLUDED.country,
                state           = EXCLUDED.state,
                city            = EXCLUDED.city,
                address         = EXCLUDED.address,
                address2        = EXCLUDED.address2,
                zip             = EXCLUDED.zip,
                phone           = EXCLUDED.phone,
                email           = EXCLUDED.email,
                is_public       = EXCLUDED.is_public,
                external_id     = EXCLUDED.external_id,
                additional_info = EXCLUDED.additional_info,
                version         = customer.version + 1
            "#,
            customer.id,
            customer.created_time,
            customer.tenant_id,
            customer.title,
            customer.country,
            customer.state,
            customer.city,
            customer.address,
            customer.address2,
            customer.zip,
            customer.phone,
            customer.email,
            customer.is_public,
            customer.external_id,
            additional_info,
            customer.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(customer.id).await?.ok_or(DaoError::NotFound)
    }
}

// ── Internal query struct ─────────────────────────────────────────────────────

struct CustomerRow {
    id:              Uuid,
    created_time:    i64,
    tenant_id:       Uuid,
    title:           String,
    country:         Option<String>,
    state:           Option<String>,
    city:            Option<String>,
    address:         Option<String>,
    address2:        Option<String>,
    zip:             Option<String>,
    phone:           Option<String>,
    email:           Option<String>,
    is_public:       bool,
    external_id:     Option<Uuid>,
    additional_info: Option<String>,
    version:         i64,
}
