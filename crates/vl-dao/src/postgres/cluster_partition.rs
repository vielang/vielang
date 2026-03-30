use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;

use vl_core::entities::ClusterPartition;
use crate::DaoError;

pub struct ClusterPartitionDao {
    pool: PgPool,
}

impl ClusterPartitionDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    /// Upsert a single partition assignment.
    #[instrument(skip(self))]
    pub async fn assign(
        &self,
        partition_id: i32,
        node_id:      &str,
    ) -> Result<(), DaoError> {
        let now = Utc::now().timestamp_millis();
        sqlx::query!(
            r#"
            INSERT INTO cluster_partition (partition_id, node_id, assigned_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (partition_id) DO UPDATE
            SET node_id     = EXCLUDED.node_id,
                assigned_at = EXCLUDED.assigned_at
            "#,
            partition_id,
            node_id,
            now,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Return all partition assignments ordered by partition_id.
    #[instrument(skip(self))]
    pub async fn find_all(&self) -> Result<Vec<ClusterPartition>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT partition_id, node_id, assigned_at
            FROM cluster_partition
            ORDER BY partition_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ClusterPartition {
            partition_id: r.partition_id,
            node_id:      r.node_id,
            assigned_at:  r.assigned_at,
        }).collect())
    }

    /// Return partitions assigned to a specific node.
    #[instrument(skip(self))]
    pub async fn find_by_node(&self, node_id: &str) -> Result<Vec<ClusterPartition>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT partition_id, node_id, assigned_at
            FROM cluster_partition
            WHERE node_id = $1
            ORDER BY partition_id
            "#,
            node_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ClusterPartition {
            partition_id: r.partition_id,
            node_id:      r.node_id,
            assigned_at:  r.assigned_at,
        }).collect())
    }

    /// Reassign all partitions owned by `dead_node` evenly across `live_nodes`.
    /// Returns the number of partitions reassigned.
    #[instrument(skip(self))]
    pub async fn failover(
        &self,
        dead_node_id: &str,
        live_nodes:   &[String],
    ) -> Result<usize, DaoError> {
        if live_nodes.is_empty() {
            return Ok(0);
        }
        let orphans = self.find_by_node(dead_node_id).await?;
        let now = Utc::now().timestamp_millis();
        let mut count = 0usize;

        for (i, partition) in orphans.iter().enumerate() {
            let new_owner = &live_nodes[i % live_nodes.len()];
            sqlx::query!(
                r#"
                UPDATE cluster_partition
                SET node_id = $2, assigned_at = $3
                WHERE partition_id = $1
                "#,
                partition.partition_id,
                new_owner,
                now,
            )
            .execute(&self.pool)
            .await?;
            count += 1;
        }
        Ok(count)
    }

    /// Seed initial partition assignments for `num_partitions` across `live_nodes`.
    /// Only inserts rows that don't already exist.
    #[instrument(skip(self))]
    pub async fn seed(
        &self,
        num_partitions: i32,
        live_nodes:     &[String],
    ) -> Result<(), DaoError> {
        if live_nodes.is_empty() { return Ok(()); }
        let now = Utc::now().timestamp_millis();
        for p in 0..num_partitions {
            let node = &live_nodes[(p as usize) % live_nodes.len()];
            sqlx::query!(
                r#"
                INSERT INTO cluster_partition (partition_id, node_id, assigned_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (partition_id) DO NOTHING
                "#,
                p,
                node,
                now,
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
