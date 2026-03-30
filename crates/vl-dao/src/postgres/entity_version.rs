use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{CommitRequest, EntityVersion};
use crate::DaoError;

pub struct EntityVersionDao {
    pool: PgPool,
}

impl EntityVersionDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new version snapshot for an entity.
    /// Auto-increments version_number and computes JSON diff vs previous version.
    #[instrument(skip(self, req))]
    pub async fn commit(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        req: &CommitRequest,
    ) -> Result<EntityVersion, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();

        // Get latest version number (COALESCE returns 0 if no rows)
        let prev_version: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version_number), 0) FROM entity_version WHERE entity_id = $1",
            req.entity_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let new_version = prev_version + 1;

        // Compute diff vs previous version if one exists
        let diff_value: Option<serde_json::Value> = if prev_version > 0 {
            let prev_row = sqlx::query!(
                r#"SELECT snapshot as "snapshot: serde_json::Value"
                   FROM entity_version
                   WHERE entity_id = $1 AND version_number = $2"#,
                req.entity_id,
                prev_version
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = prev_row {
                let patch = json_patch::diff(&row.snapshot, &req.snapshot);
                serde_json::to_value(&patch).ok()
            } else {
                None
            }
        } else {
            None
        };

        let row = sqlx::query!(
            r#"INSERT INTO entity_version
               (tenant_id, entity_id, entity_type, version_number, commit_msg, snapshot, diff, created_by, created_time)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id,
                         tenant_id,
                         entity_id,
                         entity_type,
                         version_number,
                         commit_msg,
                         snapshot as "snapshot: serde_json::Value",
                         diff,
                         created_by,
                         created_time"#,
            tenant_id,
            req.entity_id,
            &req.entity_type,
            new_version,
            req.commit_msg,
            req.snapshot,
            diff_value,
            user_id,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EntityVersion {
            id: row.id,
            tenant_id: row.tenant_id,
            entity_id: row.entity_id,
            entity_type: row.entity_type,
            version_number: row.version_number,
            commit_msg: row.commit_msg,
            snapshot: row.snapshot,
            diff: row.diff,
            created_by: row.created_by,
            created_time: row.created_time,
        })
    }

    /// List versions for an entity, newest first, paginated.
    #[instrument(skip(self))]
    pub async fn list_versions(
        &self,
        entity_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<EntityVersion>, i64), DaoError> {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM entity_version WHERE entity_id = $1",
            entity_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let offset = page * page_size;
        let rows = sqlx::query!(
            r#"SELECT id,
                      tenant_id,
                      entity_id,
                      entity_type,
                      version_number,
                      commit_msg,
                      snapshot as "snapshot: serde_json::Value",
                      diff,
                      created_by,
                      created_time
               FROM entity_version
               WHERE entity_id = $1
               ORDER BY version_number DESC
               LIMIT $2 OFFSET $3"#,
            entity_id,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let versions = rows
            .into_iter()
            .map(|r| EntityVersion {
                id: r.id,
                tenant_id: r.tenant_id,
                entity_id: r.entity_id,
                entity_type: r.entity_type,
                version_number: r.version_number,
                commit_msg: r.commit_msg,
                snapshot: r.snapshot,
                diff: r.diff,
                created_by: r.created_by,
                created_time: r.created_time,
            })
            .collect();

        Ok((versions, total))
    }

    /// Get a single version by its UUID.
    #[instrument(skip(self))]
    pub async fn get_version(&self, id: Uuid) -> Result<Option<EntityVersion>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id,
                      tenant_id,
                      entity_id,
                      entity_type,
                      version_number,
                      commit_msg,
                      snapshot as "snapshot: serde_json::Value",
                      diff,
                      created_by,
                      created_time
               FROM entity_version
               WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntityVersion {
            id: r.id,
            tenant_id: r.tenant_id,
            entity_id: r.entity_id,
            entity_type: r.entity_type,
            version_number: r.version_number,
            commit_msg: r.commit_msg,
            snapshot: r.snapshot,
            diff: r.diff,
            created_by: r.created_by,
            created_time: r.created_time,
        }))
    }

    /// Get a version by entity_id + version_number.
    #[instrument(skip(self))]
    pub async fn get_by_number(
        &self,
        entity_id: Uuid,
        version_number: i64,
    ) -> Result<Option<EntityVersion>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id,
                      tenant_id,
                      entity_id,
                      entity_type,
                      version_number,
                      commit_msg,
                      snapshot as "snapshot: serde_json::Value",
                      diff,
                      created_by,
                      created_time
               FROM entity_version
               WHERE entity_id = $1 AND version_number = $2"#,
            entity_id,
            version_number
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntityVersion {
            id: r.id,
            tenant_id: r.tenant_id,
            entity_id: r.entity_id,
            entity_type: r.entity_type,
            version_number: r.version_number,
            commit_msg: r.commit_msg,
            snapshot: r.snapshot,
            diff: r.diff,
            created_by: r.created_by,
            created_time: r.created_time,
        }))
    }

    /// Delete old versions beyond max_versions, keeping the most recent ones.
    /// Returns the number of rows deleted.
    #[instrument(skip(self))]
    pub async fn cleanup_old_versions(
        &self,
        entity_id: Uuid,
        max_versions: i64,
    ) -> Result<i64, DaoError> {
        let result = sqlx::query!(
            r#"DELETE FROM entity_version
               WHERE entity_id = $1
                 AND version_number NOT IN (
                     SELECT version_number FROM entity_version
                     WHERE entity_id = $1
                     ORDER BY version_number DESC
                     LIMIT $2
                 )"#,
            entity_id,
            max_versions
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
