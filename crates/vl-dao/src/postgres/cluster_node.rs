use sqlx::PgPool;
use tracing::instrument;

use vl_core::entities::{ClusterNode, ClusterLeaderInfo};
use crate::DaoError;

pub struct ClusterNodeDao {
    pool: PgPool,
}

fn map_node(r: ClusterNodeRow) -> ClusterNode {
    ClusterNode {
        node_id:        r.node_id,
        host:           r.host,
        port:           r.port,
        status:         r.status,
        service_type:   r.service_type,
        last_heartbeat: r.last_heartbeat,
        joined_at:      r.joined_at,
        metadata:       r.metadata,
        is_leader:      r.is_leader,
        grpc_port:      r.grpc_port,
        leader_epoch:   r.leader_epoch,
    }
}

struct ClusterNodeRow {
    node_id:        String,
    host:           String,
    port:           i32,
    status:         String,
    service_type:   String,
    last_heartbeat: i64,
    joined_at:      i64,
    metadata:       serde_json::Value,
    is_leader:      bool,
    grpc_port:      i32,
    leader_epoch:   i64,
}

impl ClusterNodeDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert or update a cluster node record.
    #[instrument(skip(self, metadata))]
    pub async fn upsert_node(
        &self,
        node_id:      &str,
        host:         &str,
        port:         i32,
        grpc_port:    i32,
        service_type: &str,
        metadata:     serde_json::Value,
    ) -> Result<ClusterNode, DaoError> {
        let now_ms = chrono::Utc::now().timestamp_millis();

        let row = sqlx::query!(
            r#"
            INSERT INTO cluster_node
                (node_id, host, port, grpc_port, service_type, metadata, last_heartbeat, joined_at, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, 'ACTIVE')
            ON CONFLICT (node_id) DO UPDATE
            SET host           = EXCLUDED.host,
                port           = EXCLUDED.port,
                grpc_port      = EXCLUDED.grpc_port,
                service_type   = EXCLUDED.service_type,
                metadata       = EXCLUDED.metadata,
                last_heartbeat = EXCLUDED.last_heartbeat,
                status         = 'ACTIVE'
            RETURNING node_id, host, port, grpc_port, status, service_type,
                      last_heartbeat, joined_at, metadata,
                      is_leader, leader_epoch
            "#,
            node_id,
            host,
            port,
            grpc_port,
            service_type,
            metadata,
            now_ms,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(map_node(ClusterNodeRow {
            node_id:        row.node_id,
            host:           row.host,
            port:           row.port,
            grpc_port:      row.grpc_port,
            status:         row.status,
            service_type:   row.service_type,
            last_heartbeat: row.last_heartbeat,
            joined_at:      row.joined_at,
            metadata:       row.metadata,
            is_leader:      row.is_leader,
            leader_epoch:   row.leader_epoch,
        }))
    }

    /// Refresh last_heartbeat for a node to the current time in ms.
    #[instrument(skip(self))]
    pub async fn heartbeat(&self, node_id: &str) -> Result<(), DaoError> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            "UPDATE cluster_node SET last_heartbeat = $2 WHERE node_id = $1",
            node_id,
            now_ms,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Return nodes with status = 'ACTIVE' and heartbeat within the last 30 seconds.
    #[instrument(skip(self))]
    pub async fn find_active_nodes(&self) -> Result<Vec<ClusterNode>, DaoError> {
        let threshold = chrono::Utc::now().timestamp_millis() - 30_000;
        let rows = sqlx::query!(
            r#"
            SELECT node_id, host, port, grpc_port, status, service_type,
                   last_heartbeat, joined_at, metadata, is_leader, leader_epoch
            FROM cluster_node
            WHERE status = 'ACTIVE' AND last_heartbeat > $1
            ORDER BY joined_at ASC
            "#,
            threshold,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| map_node(ClusterNodeRow {
            node_id: r.node_id, host: r.host, port: r.port, grpc_port: r.grpc_port,
            status: r.status, service_type: r.service_type,
            last_heartbeat: r.last_heartbeat, joined_at: r.joined_at, metadata: r.metadata,
            is_leader: r.is_leader, leader_epoch: r.leader_epoch,
        })).collect())
    }

    /// Return all nodes regardless of status.
    #[instrument(skip(self))]
    pub async fn find_all_nodes(&self) -> Result<Vec<ClusterNode>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT node_id, host, port, grpc_port, status, service_type,
                   last_heartbeat, joined_at, metadata, is_leader, leader_epoch
            FROM cluster_node
            ORDER BY joined_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| map_node(ClusterNodeRow {
            node_id: r.node_id, host: r.host, port: r.port, grpc_port: r.grpc_port,
            status: r.status, service_type: r.service_type,
            last_heartbeat: r.last_heartbeat, joined_at: r.joined_at, metadata: r.metadata,
            is_leader: r.is_leader, leader_epoch: r.leader_epoch,
        })).collect())
    }

    /// Find a specific node by node_id.
    #[instrument(skip(self))]
    pub async fn find_node(&self, node_id: &str) -> Result<Option<ClusterNode>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT node_id, host, port, grpc_port, status, service_type,
                   last_heartbeat, joined_at, metadata, is_leader, leader_epoch
            FROM cluster_node
            WHERE node_id = $1
            "#,
            node_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| map_node(ClusterNodeRow {
            node_id: r.node_id, host: r.host, port: r.port, grpc_port: r.grpc_port,
            status: r.status, service_type: r.service_type,
            last_heartbeat: r.last_heartbeat, joined_at: r.joined_at, metadata: r.metadata,
            is_leader: r.is_leader, leader_epoch: r.leader_epoch,
        })))
    }

    /// Mark a node as SUSPECT.
    #[instrument(skip(self))]
    pub async fn mark_suspect(&self, node_id: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE cluster_node SET status = 'SUSPECT' WHERE node_id = $1",
            node_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark a node as DOWN and clear its leader flag if it was leader.
    #[instrument(skip(self))]
    pub async fn mark_down(&self, node_id: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE cluster_node SET status = 'DOWN', is_leader = false WHERE node_id = $1",
            node_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete DOWN nodes whose last_heartbeat is older than before_ts.
    /// Returns the number of rows deleted.
    #[instrument(skip(self))]
    pub async fn cleanup_dead_nodes(&self, before_ts: i64) -> Result<i64, DaoError> {
        let result = sqlx::query!(
            "DELETE FROM cluster_node WHERE status = 'DOWN' AND last_heartbeat < $1",
            before_ts,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // ── Leader Election ───────────────────────────────────────────────────────

    /// Attempt to claim leadership (optimistic CAS).
    ///
    /// Steps:
    /// 1. If no leader exists → immediately claim (UPDATE set is_leader=true).
    /// 2. If the current leader's heartbeat is stale > election_timeout_ms → take over.
    /// 3. Return (winner_node_id, epoch).
    ///
    /// The UNIQUE partial index on `is_leader WHERE is_leader = TRUE` ensures at most
    /// one winner even with concurrent calls.
    #[instrument(skip(self))]
    pub async fn try_claim_leader(
        &self,
        candidate_id:        &str,
        election_timeout_ms: i64,
    ) -> Result<(String, i64), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let stale_threshold = now - election_timeout_ms;

        // Attempt takeover: set this node as leader if no active leader exists.
        let rows_updated = sqlx::query!(
            r#"
            UPDATE cluster_node
            SET is_leader    = true,
                leader_epoch = leader_epoch + 1,
                last_heartbeat = $2
            WHERE node_id = $1
              AND (
                  -- No current leader
                  NOT EXISTS (
                      SELECT 1 FROM cluster_node
                      WHERE is_leader = true AND last_heartbeat > $3
                  )
              )
            "#,
            candidate_id,
            now,
            stale_threshold,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_updated > 0 {
            // Clear is_leader on all other nodes (in case of split-brain recovery)
            sqlx::query!(
                "UPDATE cluster_node SET is_leader = false WHERE node_id != $1 AND is_leader = true",
                candidate_id,
            )
            .execute(&self.pool)
            .await?;
        }

        // Return whoever is leader now
        let leader = sqlx::query!(
            r#"
            SELECT node_id, leader_epoch
            FROM cluster_node
            WHERE is_leader = true
            ORDER BY leader_epoch DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = leader {
            Ok((r.node_id, r.leader_epoch))
        } else {
            // Fallback: caller is "leader" by default in single-node mode
            Ok((candidate_id.to_string(), 0))
        }
    }

    /// Renew the leader lease (bump last_heartbeat). Returns false if we are no longer leader.
    #[instrument(skip(self))]
    pub async fn renew_leader_lease(&self, node_id: &str, epoch: i64) -> Result<bool, DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        let rows = sqlx::query!(
            r#"
            UPDATE cluster_node
            SET last_heartbeat = $3
            WHERE node_id = $1 AND is_leader = true AND leader_epoch = $2
            "#,
            node_id,
            epoch,
            now,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows > 0)
    }

    /// Release leadership voluntarily.
    #[instrument(skip(self))]
    pub async fn step_down_leader(&self, node_id: &str) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE cluster_node SET is_leader = false WHERE node_id = $1",
            node_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Return the current leader node, if any.
    #[instrument(skip(self))]
    pub async fn find_leader(&self) -> Result<Option<ClusterLeaderInfo>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT node_id, host, grpc_port, leader_epoch
            FROM cluster_node
            WHERE is_leader = true
            ORDER BY leader_epoch DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ClusterLeaderInfo {
            leader_node_id: Some(r.node_id),
            host:           Some(r.host),
            grpc_port:      Some(r.grpc_port),
            leader_epoch:   r.leader_epoch,
            is_local:       false, // caller sets this
        }))
    }
}
