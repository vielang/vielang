/// P15: Raft-style leader election for VieLang cluster.
///
/// Implementation uses PostgreSQL optimistic locking rather than the openraft
/// crate, giving us full Raft semantics (single leader, epoch fencing) without
/// requiring an extra external process.  The design is intentionally compatible
/// with a future openraft swap: all coordination is behind the `LeaderElection`
/// trait abstraction.
///
/// ThingsBoard Java reference:
///   thingsboard/.../service/cluster/discovery/ZkDiscoveryService.java
///   (leader election via ZK ephemeral nodes / distributed locks)
///
/// Consensus properties guaranteed by this implementation:
/// - **Safety**: at most one node can hold `is_leader = true` at a time
///   (enforced by the UNIQUE partial index `idx_cluster_node_leader`).
/// - **Liveness**: if the current leader stops refreshing its epoch, another
///   node will win the next election after `election_timeout_ms`.
/// - **Fencing**: each election bumps `leader_epoch`; stale leaders with a
///   lower epoch are rejected.

pub mod leader;

pub use leader::{LeaderElection, LeaderState};
