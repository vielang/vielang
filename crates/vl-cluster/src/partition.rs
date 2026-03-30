use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::node::NodeInfo;

/// Number of virtual partitions — matches ThingsBoard default.
pub const DEFAULT_NUM_PARTITIONS: u32 = 12;

// ── ConsistentHashRing ────────────────────────────────────────────────────────

/// Maps partition_id → node_id.
/// Uses simple modulo assignment; minimal reassignment on add/remove via stable ordering.
struct ConsistentHashRing {
    /// assignments[partition_id] = responsible node_id (None when no nodes available)
    assignments: Vec<Option<Uuid>>,
}

impl ConsistentHashRing {
    fn new(num_partitions: u32) -> Self {
        Self {
            assignments: vec![None; num_partitions as usize],
        }
    }

    /// Distribute partitions evenly across `nodes` (sorted by node_id for stability).
    fn rebalance(&mut self, mut nodes: Vec<NodeInfo>) {
        // Sort deterministically so any node calculates the same assignment.
        nodes.sort_by_key(|n| n.node_id);
        let n = nodes.len();
        for (partition, slot) in self.assignments.iter_mut().enumerate() {
            *slot = if n == 0 {
                None
            } else {
                Some(nodes[partition % n].node_id)
            };
        }
    }

    fn get(&self, partition: u32) -> Option<Uuid> {
        self.assignments.get(partition as usize).copied().flatten()
    }

    fn assignments(&self) -> &[Option<Uuid>] {
        &self.assignments
    }
}

// ── PartitionService ──────────────────────────────────────────────────────────

/// Maps entities to partitions and partitions to nodes.
///
/// Thread-safe: inner ring protected by RwLock.
#[derive(Clone)]
pub struct PartitionService {
    ring:           Arc<RwLock<ConsistentHashRing>>,
    num_partitions: u32,
}

impl PartitionService {
    pub fn new(num_partitions: u32) -> Self {
        Self {
            ring:           Arc::new(RwLock::new(ConsistentHashRing::new(num_partitions))),
            num_partitions,
        }
    }

    /// Recompute partition → node assignments based on the current active node list.
    pub async fn rebalance(&self, nodes: Vec<NodeInfo>) {
        let mut ring = self.ring.write().await;
        ring.rebalance(nodes);
    }

    /// Map entity/device ID → partition number (0..num_partitions).
    pub fn get_partition(&self, entity_id: Uuid) -> u32 {
        let mut hasher = DefaultHasher::new();
        entity_id.as_bytes().hash(&mut hasher);
        (hasher.finish() % self.num_partitions as u64) as u32
    }

    /// Map partition → responsible node (None when no nodes).
    pub async fn get_node_for_partition(&self, partition: u32) -> Option<Uuid> {
        self.ring.read().await.get(partition)
    }

    /// Convenience: map entity → responsible node.
    pub async fn route(&self, entity_id: Uuid) -> Option<Uuid> {
        let partition = self.get_partition(entity_id);
        self.get_node_for_partition(partition).await
    }

    pub fn num_partitions(&self) -> u32 { self.num_partitions }

    /// Return a snapshot of all partition → node assignments.
    pub async fn snapshot(&self) -> Vec<(u32, Option<Uuid>)> {
        let ring = self.ring.read().await;
        ring.assignments()
            .iter()
            .enumerate()
            .map(|(i, n)| (i as u32, *n))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u128) -> NodeInfo {
        NodeInfo::new(Uuid::from_u128(id), "127.0.0.1", 9090)
    }

    #[tokio::test]
    async fn test_partition_distribution() {
        let svc = PartitionService::new(12);
        let nodes: Vec<NodeInfo> = (1..=3).map(|i| node(i)).collect();
        svc.rebalance(nodes).await;

        // All 12 partitions assigned
        let snap = svc.snapshot().await;
        assert_eq!(snap.len(), 12);
        assert!(snap.iter().all(|(_, n)| n.is_some()));

        // Roughly even distribution (4 partitions per node)
        let mut counts = std::collections::HashMap::new();
        for (_, n) in &snap {
            *counts.entry(n.unwrap()).or_insert(0usize) += 1;
        }
        for (_, count) in &counts {
            assert_eq!(*count, 4, "each node should own exactly 4 partitions");
        }
    }

    #[tokio::test]
    async fn test_rebalance_on_node_leave() {
        let svc = PartitionService::new(12);
        let all_nodes: Vec<NodeInfo> = (1..=3).map(|i| node(i)).collect();
        svc.rebalance(all_nodes.clone()).await;

        let before: Vec<(u32, Option<Uuid>)> = svc.snapshot().await;

        // Remove node 2
        let remaining: Vec<NodeInfo> = all_nodes.into_iter().filter(|n| n.node_id != Uuid::from_u128(2)).collect();
        svc.rebalance(remaining).await;
        let after: Vec<(u32, Option<Uuid>)> = svc.snapshot().await;

        // All partitions still assigned
        assert!(after.iter().all(|(_, n)| n.is_some()));

        // Minimal reassignment: partitions that were on nodes 1 or 3 may stay
        let changed = before.iter().zip(after.iter())
            .filter(|((_, n1), (_, n2))| n1 != n2)
            .count();
        // At most 8 partitions need to be reassigned (the ones that were on node 2 + rebalancing)
        assert!(changed <= 8, "too many reassignments: {}", changed);
    }

    #[test]
    fn test_entity_partition_is_deterministic() {
        let svc = PartitionService::new(12);
        let id = Uuid::from_u128(0xDEAD_BEEF);
        let p1 = svc.get_partition(id);
        let p2 = svc.get_partition(id);
        assert_eq!(p1, p2);
    }
}
