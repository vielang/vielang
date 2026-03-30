use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::ClusterError;

// ── LeaderState ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderState {
    pub leader_node_id: Option<String>,
    pub leader_epoch:   i64,
    pub is_local:       bool,
}

// ── LeaderElection ────────────────────────────────────────────────────────────

/// PostgreSQL-backed leader election.
///
/// Algorithm:
/// 1. On startup every node calls `try_become_leader()`.
/// 2. The first node to execute the CAS UPDATE wins (`is_leader = true`,
///    `leader_epoch` incremented, `last_heartbeat` refreshed).
/// 3. The winner then starts a background `lease_renewal_task` that calls
///    `renew_lease()` every `heartbeat_interval_ms`.
/// 4. All nodes also run `maybe_takeover()` every `election_timeout_ms`.
///    If `last_heartbeat` of the leader is stale by > `election_timeout_ms`,
///    they attempt to CAS the leadership away from the dead leader.
/// 5. The UNIQUE partial index on `is_leader WHERE is_leader = TRUE` ensures
///    at most one winner regardless of race conditions.
pub struct LeaderElection {
    node_id:              String,
    heartbeat_interval:   Duration,
    election_timeout:     Duration,
    state:                Arc<RwLock<LeaderState>>,
    /// Injected callback to perform DB operations — avoids importing sqlx into vl-cluster.
    backend:              Arc<dyn LeaderElectionBackend>,
}

/// Callbacks into the DB layer (implemented in vl-api, injected at startup).
#[async_trait::async_trait]
pub trait LeaderElectionBackend: Send + Sync + 'static {
    /// Try to claim leadership (CAS). Returns the winner's node_id and epoch.
    async fn try_claim(&self, candidate: &str) -> Result<(String, i64), ClusterError>;
    /// Renew the lease for `node_id` (bump last_heartbeat). Returns false if we are no longer leader.
    async fn renew_lease(&self, node_id: &str, epoch: i64) -> Result<bool, ClusterError>;
    /// Step down — release leadership.
    async fn step_down(&self, node_id: &str) -> Result<(), ClusterError>;
    /// Query current leader info.
    async fn current_leader(&self) -> Result<Option<(String, i64)>, ClusterError>;
}

impl LeaderElection {
    pub fn new(
        node_id:            impl Into<String>,
        heartbeat_interval: Duration,
        election_timeout:   Duration,
        backend:            Arc<dyn LeaderElectionBackend>,
    ) -> Self {
        let node_id = node_id.into();
        let state = Arc::new(RwLock::new(LeaderState {
            leader_node_id: None,
            leader_epoch:   0,
            is_local:       false,
        }));
        Self { node_id, heartbeat_interval, election_timeout, state, backend }
    }

    /// True if this node currently believes it is the leader.
    pub async fn is_leader(&self) -> bool {
        self.state.read().await.is_local
    }

    /// Current leader snapshot.
    pub async fn state(&self) -> LeaderState {
        self.state.read().await.clone()
    }

    /// Attempt to become leader on startup. Returns true if we won.
    pub async fn try_become_leader(&self) -> Result<bool, ClusterError> {
        let (winner, epoch) = self.backend.try_claim(&self.node_id).await?;
        let is_local = winner == self.node_id;
        {
            let mut s = self.state.write().await;
            s.leader_node_id = Some(winner.clone());
            s.leader_epoch   = epoch;
            s.is_local       = is_local;
        }
        if is_local {
            info!(node_id = %self.node_id, epoch, "This node won leader election");
        } else {
            info!(node_id = %self.node_id, leader = %winner, "Another node is the leader");
        }
        Ok(is_local)
    }

    /// Start background lease renewal + takeover check tasks.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let renewal_handle = {
            let this = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(this.heartbeat_interval);
                loop {
                    interval.tick().await;
                    let (is_local, epoch) = {
                        let s = this.state.read().await;
                        (s.is_local, s.leader_epoch)
                    };
                    if is_local {
                        match this.backend.renew_lease(&this.node_id, epoch).await {
                            Ok(true)  => debug!(node_id = %this.node_id, "Leader lease renewed"),
                            Ok(false) => {
                                warn!(node_id = %this.node_id, "Lost leadership — another node took over");
                                let mut s = this.state.write().await;
                                s.is_local = false;
                            }
                            Err(e) => warn!("renew_lease error: {e}"),
                        }
                    }
                }
            })
        };

        let takeover_handle = {
            let this = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(this.election_timeout);
                interval.tick().await; // skip first tick
                loop {
                    interval.tick().await;
                    let is_local = this.state.read().await.is_local;
                    if is_local { continue; }
                    // Check if the current leader is stale
                    match this.backend.current_leader().await {
                        Ok(None) => {
                            info!(node_id = %this.node_id, "No leader found — attempting election");
                            let _ = this.try_become_leader().await;
                        }
                        Ok(Some(_)) => {} // leader alive, do nothing
                        Err(e) => warn!("current_leader check failed: {e}"),
                    }
                }
            })
        };

        tokio::spawn(async move {
            tokio::select! {
                _ = renewal_handle => {}
                _ = takeover_handle => {}
            }
        })
    }

    /// Voluntarily step down as leader (e.g., on graceful shutdown).
    pub async fn step_down(&self) -> Result<(), ClusterError> {
        let is_local = self.state.read().await.is_local;
        if is_local {
            self.backend.step_down(&self.node_id).await?;
            let mut s = self.state.write().await;
            s.is_local = false;
            info!(node_id = %self.node_id, "Stepped down as leader");
        }
        Ok(())
    }
}
