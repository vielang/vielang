use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use uuid::Uuid;

use crate::node::NodeInfo;

/// Rendezvous (highest random weight) hashing.
///
/// Phân chia entities sang nodes một cách ổn định — khi thêm/bỏ node
/// chỉ ~1/N entities cần rebalance (khác consistent ring hashing).
///
/// Khớp ThingsBoard Java: TbServiceInfoProvider.getServicesByType() + partitioning logic.
pub struct RendezvousHasher;

impl RendezvousHasher {
    /// Trả về node chịu trách nhiệm cho entity_id.
    /// Trả về None nếu danh sách nodes rỗng.
    pub fn get_node<'a>(entity_id: Uuid, nodes: &'a [NodeInfo]) -> Option<&'a NodeInfo> {
        nodes.iter().max_by_key(|node| score(entity_id, node.node_id))
    }

    /// Kiểm tra node có phải responsible cho entity không.
    pub fn is_responsible(entity_id: Uuid, node: &NodeInfo, all_nodes: &[NodeInfo]) -> bool {
        Self::get_node(entity_id, all_nodes)
            .map(|n| n.node_id == node.node_id)
            .unwrap_or(false)
    }
}

/// Score = hash(entity_id XOR node_id) — deterministic, independent per pair.
fn score(entity_id: Uuid, node_id: Uuid) -> u64 {
    let mut hasher = DefaultHasher::new();
    // XOR bytes của hai UUIDs để tạo unique combined key
    let e = entity_id.as_bytes();
    let n = node_id.as_bytes();
    let combined: Vec<u8> = e.iter().zip(n.iter()).map(|(a, b)| a ^ b).collect();
    combined.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u128, host: &str) -> NodeInfo {
        NodeInfo::new(Uuid::from_u128(id), host, 9090)
    }

    #[test]
    fn empty_nodes_returns_none() {
        let entity = Uuid::new_v4();
        assert!(RendezvousHasher::get_node(entity, &[]).is_none());
    }

    #[test]
    fn single_node_always_responsible() {
        let entity  = Uuid::new_v4();
        let node    = make_node(1, "node-1");
        let nodes   = [node.clone()];
        let result  = RendezvousHasher::get_node(entity, &nodes);
        assert_eq!(result.unwrap().node_id, node.node_id);
    }

    #[test]
    fn deterministic_same_entity_same_node() {
        let entity = Uuid::from_u128(0xDEAD_BEEF);
        let nodes: Vec<NodeInfo> = (1..=5).map(|i| make_node(i, &format!("node-{}", i))).collect();
        let first  = RendezvousHasher::get_node(entity, &nodes).unwrap().node_id;
        let second = RendezvousHasher::get_node(entity, &nodes).unwrap().node_id;
        assert_eq!(first, second);
    }

    #[test]
    fn different_entities_can_land_on_different_nodes() {
        let nodes: Vec<NodeInfo> = (1..=3).map(|i| make_node(i, &format!("node-{}", i))).collect();
        let assignments: Vec<Uuid> = (0..20_u128)
            .map(|i| RendezvousHasher::get_node(Uuid::from_u128(i * 1000), &nodes).unwrap().node_id)
            .collect();
        let unique: std::collections::HashSet<_> = assignments.iter().collect();
        // Với 20 entities và 3 nodes, expectation là >= 2 distinct nodes được chọn
        assert!(unique.len() >= 2, "expected distribution across nodes, got {:?}", unique);
    }

    #[test]
    fn is_responsible_matches_get_node() {
        let entity = Uuid::from_u128(0xCAFE);
        let nodes: Vec<NodeInfo> = (1..=4).map(|i| make_node(i, &format!("n{}", i))).collect();
        let winner = RendezvousHasher::get_node(entity, &nodes).unwrap().clone();
        assert!(RendezvousHasher::is_responsible(entity, &winner, &nodes));
        // Các node khác không responsible
        for node in nodes.iter().filter(|n| n.node_id != winner.node_id) {
            assert!(!RendezvousHasher::is_responsible(entity, node, &nodes));
        }
    }
}
