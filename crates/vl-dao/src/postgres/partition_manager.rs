use chrono::{Datelike, TimeZone, Utc};
use sqlx::PgPool;
use tracing::instrument;

use crate::DaoError;

/// Manages PostgreSQL range partitions for ts_kv automatically.
/// Ensures yearly partitions exist for the current year + 3 years ahead,
/// so inserts never fail due to a missing partition boundary.
pub struct PartitionManagerDao {
    pool: PgPool,
}

impl PartitionManagerDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check and create any missing yearly partitions for ts_kv.
    /// Safe to call repeatedly — uses CREATE TABLE IF NOT EXISTS equivalent
    /// by first checking pg_tables so DDL is only issued when needed.
    #[instrument(skip(self))]
    pub async fn ensure_future_partitions(&self) -> Result<(), DaoError> {
        let now_year = Utc::now().year();
        for year in now_year..=(now_year + 3) {
            self.ensure_yearly_partition(year).await?;
        }
        Ok(())
    }

    async fn ensure_yearly_partition(&self, year: i32) -> Result<(), DaoError> {
        let part_name = format!("ts_kv_{}", year);

        // Check if the partition table already exists
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = $1",
        )
        .bind(&part_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        if exists.is_some() {
            return Ok(());
        }

        // Calculate epoch-millis boundaries for the year
        let start_ms = Utc
            .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
            .single()
            .ok_or_else(|| DaoError::Constraint(format!("invalid year {}", year)))?
            .timestamp_millis();

        let end_ms = Utc
            .with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
            .single()
            .ok_or_else(|| DaoError::Constraint(format!("invalid year {}", year + 1)))?
            .timestamp_millis();

        // DDL cannot go through query!() macro — use non-macro query()
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} PARTITION OF ts_kv \
             FOR VALUES FROM ({}) TO ({})",
            part_name, start_ms, end_ms
        );
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map_err(DaoError::from_sqlx)?;

        tracing::info!(
            partition = %part_name,
            start_ms,
            end_ms,
            "Created ts_kv partition"
        );
        Ok(())
    }
}
