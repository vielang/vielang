//! Property graph for digital twin relationships.
//!
//! Edges are typed and can carry properties. Supports standard industrial
//! relationship types: spatial (ISA-95), logical, and temporal.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// ── Edge types ───────────────────────────────────────────────────────────────

/// Standard relationship categories in industrial digital twins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipCategory {
    /// Spatial containment (ISA-95): Site → Area → Line → Cell
    Spatial,
    /// Logical relationships: controls, monitors, feeds, supplies
    Logical,
    /// Hierarchical: parent-child non-spatial (organizational)
    Hierarchical,
    /// Temporal: precedes, follows (process flow)
    Temporal,
    /// Data flow: telemetry source, aggregation target
    DataFlow,
    /// Custom user-defined category
    Custom(String),
}

/// A typed edge in the twin graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub edge_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    /// Relationship type name (e.g., "contains", "monitors", "locatedIn").
    pub rel_type: String,
    /// Category of this relationship.
    pub category: RelationshipCategory,
    /// Edge properties (e.g., weight, priority, metadata).
    pub properties: HashMap<String, serde_json::Value>,
    /// Whether this is a directional (true) or bidirectional (false) edge.
    pub directed: bool,
    /// Strength/confidence of the relationship (0.0 - 1.0).
    pub weight: f64,
    /// Creation timestamp (ms).
    pub created_at: i64,
}

impl GraphEdge {
    pub fn new(source_id: Uuid, target_id: Uuid, rel_type: &str, category: RelationshipCategory) -> Self {
        Self {
            edge_id: Uuid::new_v4(),
            source_id,
            target_id,
            rel_type: rel_type.into(),
            category,
            properties: HashMap::new(),
            directed: true,
            weight: 1.0,
            created_at: crate::components::device::current_time_ms(),
        }
    }

    pub fn with_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn bidirectional(mut self) -> Self {
        self.directed = false;
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

/// A node in the twin graph (lightweight reference).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub node_id: Uuid,
    pub label: String,
    /// Model identifier (DTMI or device_type).
    pub model_id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

// ── Graph resource ───────────────────────────────────────────────────────────

/// The twin relationship graph — a Bevy resource.
///
/// Maintains adjacency lists for both outgoing and incoming edges,
/// enabling efficient traversal in both directions.
#[derive(Resource, Default)]
pub struct TwinGraph {
    nodes: HashMap<Uuid, GraphNode>,
    /// source_id → edges
    outgoing: HashMap<Uuid, Vec<GraphEdge>>,
    /// target_id → edges
    incoming: HashMap<Uuid, Vec<GraphEdge>>,
    /// edge_id → (source_id, target_id) for O(1) lookup
    edge_index: HashMap<Uuid, (Uuid, Uuid)>,
}

impl TwinGraph {
    // ── Node operations ──────────────────────────────────────────────────────

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.node_id, node);
    }

    pub fn remove_node(&mut self, node_id: Uuid) -> Option<GraphNode> {
        // Remove all edges involving this node
        let out_edges: Vec<Uuid> = self.outgoing.get(&node_id)
            .map(|edges| edges.iter().map(|e| e.edge_id).collect())
            .unwrap_or_default();
        let in_edges: Vec<Uuid> = self.incoming.get(&node_id)
            .map(|edges| edges.iter().map(|e| e.edge_id).collect())
            .unwrap_or_default();

        for eid in out_edges.iter().chain(in_edges.iter()) {
            self.remove_edge(*eid);
        }

        self.outgoing.remove(&node_id);
        self.incoming.remove(&node_id);
        self.nodes.remove(&node_id)
    }

    pub fn get_node(&self, node_id: Uuid) -> Option<&GraphNode> {
        self.nodes.get(&node_id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    // ── Edge operations ──────────────────────────────────────────────────────

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let eid = edge.edge_id;
        let src = edge.source_id;
        let tgt = edge.target_id;

        self.edge_index.insert(eid, (src, tgt));

        // For bidirectional edges, also add reverse
        if !edge.directed {
            let reverse = GraphEdge {
                edge_id: Uuid::new_v4(),
                source_id: tgt,
                target_id: src,
                rel_type: edge.rel_type.clone(),
                category: edge.category.clone(),
                properties: edge.properties.clone(),
                directed: false,
                weight: edge.weight,
                created_at: edge.created_at,
            };
            self.incoming.entry(src).or_default().push(reverse.clone());
            self.outgoing.entry(tgt).or_default().push(reverse);
        }

        self.incoming.entry(tgt).or_default().push(edge.clone());
        self.outgoing.entry(src).or_default().push(edge);
    }

    pub fn remove_edge(&mut self, edge_id: Uuid) -> bool {
        if let Some((src, tgt)) = self.edge_index.remove(&edge_id) {
            if let Some(edges) = self.outgoing.get_mut(&src) {
                edges.retain(|e| e.edge_id != edge_id);
            }
            if let Some(edges) = self.incoming.get_mut(&tgt) {
                edges.retain(|e| e.edge_id != edge_id);
            }
            true
        } else {
            false
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edge_index.len()
    }

    /// Get outgoing edges from a node, optionally filtered by type.
    pub fn outgoing_edges(&self, node_id: Uuid, rel_type: Option<&str>) -> Vec<&GraphEdge> {
        self.outgoing.get(&node_id)
            .map(|edges| {
                edges.iter()
                    .filter(|e| rel_type.map_or(true, |t| e.rel_type == t))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get incoming edges to a node, optionally filtered by type.
    pub fn incoming_edges(&self, node_id: Uuid, rel_type: Option<&str>) -> Vec<&GraphEdge> {
        self.incoming.get(&node_id)
            .map(|edges| {
                edges.iter()
                    .filter(|e| rel_type.map_or(true, |t| e.rel_type == t))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get direct neighbors of a node.
    pub fn neighbors(&self, node_id: Uuid) -> Vec<Uuid> {
        let mut result: HashSet<Uuid> = HashSet::new();
        if let Some(edges) = self.outgoing.get(&node_id) {
            for e in edges {
                result.insert(e.target_id);
            }
        }
        if let Some(edges) = self.incoming.get(&node_id) {
            for e in edges {
                result.insert(e.source_id);
            }
        }
        result.into_iter().collect()
    }

    /// Get the degree (number of edges) for a node.
    pub fn degree(&self, node_id: Uuid) -> usize {
        let out = self.outgoing.get(&node_id).map(|e| e.len()).unwrap_or(0);
        let inc = self.incoming.get(&node_id).map(|e| e.len()).unwrap_or(0);
        out + inc
    }

    // ── Spatial hierarchy helpers ────────────────────────────────────────────

    /// Get all children of a node via "contains" relationship.
    pub fn children(&self, parent_id: Uuid) -> Vec<Uuid> {
        self.outgoing_edges(parent_id, Some("contains"))
            .iter()
            .map(|e| e.target_id)
            .collect()
    }

    /// Get the parent of a node via "contains" relationship.
    pub fn parent(&self, child_id: Uuid) -> Option<Uuid> {
        self.incoming_edges(child_id, Some("contains"))
            .first()
            .map(|e| e.source_id)
    }

    /// Get the full ancestry path from a node to the root.
    pub fn ancestors(&self, node_id: Uuid) -> Vec<Uuid> {
        let mut path = Vec::new();
        let mut current = node_id;
        let mut visited = HashSet::new();
        while let Some(parent_id) = self.parent(current) {
            if !visited.insert(parent_id) {
                break; // cycle protection
            }
            path.push(parent_id);
            current = parent_id;
        }
        path
    }

    /// Get all descendants of a node via "contains" relationship (BFS).
    pub fn descendants(&self, root_id: Uuid) -> Vec<Uuid> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(root_id);
        visited.insert(root_id);

        while let Some(current) = queue.pop_front() {
            for child in self.children(current) {
                if visited.insert(child) {
                    result.push(child);
                    queue.push_back(child);
                }
            }
        }
        result
    }

    // ── Shortest path (BFS) ─────────────────────────────────────────────────

    /// Find shortest path between two nodes (any relationship type).
    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent_map: HashMap<Uuid, Uuid> = HashMap::new();
        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.neighbors(current) {
                if !visited.insert(neighbor) {
                    continue;
                }
                parent_map.insert(neighbor, current);
                if neighbor == to {
                    // Reconstruct path
                    let mut path = vec![to];
                    let mut cur = to;
                    while let Some(&prev) = parent_map.get(&cur) {
                        path.push(prev);
                        cur = prev;
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(neighbor);
            }
        }

        None
    }

    // ── Subgraph extraction ──────────────────────────────────────────────────

    /// Extract edges that match a category.
    pub fn edges_by_category(&self, category: &RelationshipCategory) -> Vec<&GraphEdge> {
        self.outgoing.values()
            .flat_map(|edges| edges.iter())
            .filter(|e| &e.category == category)
            .collect()
    }

    /// Get all nodes of a specific model type.
    pub fn nodes_by_model(&self, model_id: &str) -> Vec<&GraphNode> {
        self.nodes.values()
            .filter(|n| n.model_id == model_id)
            .collect()
    }
}

// ── ISA-95 hierarchy builder ─────────────────────────────────────────────────

/// Helper to build ISA-95 spatial hierarchies.
pub struct Isa95Builder<'a> {
    graph: &'a mut TwinGraph,
}

impl<'a> Isa95Builder<'a> {
    pub fn new(graph: &'a mut TwinGraph) -> Self {
        Self { graph }
    }

    /// Add a spatial node and optionally connect it to a parent.
    pub fn add_space(
        &mut self,
        node_id: Uuid,
        label: &str,
        space_type: &str,
        parent_id: Option<Uuid>,
    ) -> Uuid {
        let mut props = HashMap::new();
        props.insert("space_type".into(), serde_json::json!(space_type));

        self.graph.add_node(GraphNode {
            node_id,
            label: label.into(),
            model_id: "dtmi:vielang:Space;1".into(),
            properties: props,
        });

        if let Some(pid) = parent_id {
            self.graph.add_edge(
                GraphEdge::new(pid, node_id, "contains", RelationshipCategory::Spatial)
            );
        }

        node_id
    }

    /// Place a device twin in a spatial node.
    pub fn locate_device(&mut self, device_id: Uuid, space_id: Uuid) {
        self.graph.add_edge(
            GraphEdge::new(device_id, space_id, "locatedIn", RelationshipCategory::Spatial)
        );
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(label: &str) -> GraphNode {
        GraphNode {
            node_id: Uuid::new_v4(),
            label: label.into(),
            model_id: "test".into(),
            properties: HashMap::new(),
        }
    }

    #[test]
    fn add_and_remove_nodes() {
        let mut g = TwinGraph::default();
        let n = make_node("A");
        let id = n.node_id;
        g.add_node(n);
        assert_eq!(g.node_count(), 1);
        assert!(g.get_node(id).is_some());
        g.remove_node(id);
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn add_and_query_edges() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let a_id = a.node_id;
        let b_id = b.node_id;
        g.add_node(a);
        g.add_node(b);

        g.add_edge(GraphEdge::new(a_id, b_id, "contains", RelationshipCategory::Spatial));
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.outgoing_edges(a_id, Some("contains")).len(), 1);
        assert_eq!(g.incoming_edges(b_id, Some("contains")).len(), 1);
        assert_eq!(g.outgoing_edges(a_id, Some("monitors")).len(), 0);
    }

    #[test]
    fn spatial_hierarchy() {
        let mut g = TwinGraph::default();
        let site = make_node("Site");
        let area = make_node("Area");
        let cell = make_node("Cell");
        let s_id = site.node_id;
        let a_id = area.node_id;
        let c_id = cell.node_id;
        g.add_node(site);
        g.add_node(area);
        g.add_node(cell);

        g.add_edge(GraphEdge::new(s_id, a_id, "contains", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(a_id, c_id, "contains", RelationshipCategory::Spatial));

        assert_eq!(g.children(s_id), vec![a_id]);
        assert_eq!(g.parent(a_id), Some(s_id));
        assert_eq!(g.ancestors(c_id), vec![a_id, s_id]);
        assert_eq!(g.descendants(s_id).len(), 2);
    }

    #[test]
    fn shortest_path() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let c = make_node("C");
        let a_id = a.node_id;
        let b_id = b.node_id;
        let c_id = c.node_id;
        g.add_node(a);
        g.add_node(b);
        g.add_node(c);

        g.add_edge(GraphEdge::new(a_id, b_id, "next", RelationshipCategory::Temporal));
        g.add_edge(GraphEdge::new(b_id, c_id, "next", RelationshipCategory::Temporal));

        let path = g.shortest_path(a_id, c_id).expect("path should exist");
        assert_eq!(path, vec![a_id, b_id, c_id]);
    }

    #[test]
    fn no_path_returns_none() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let a_id = a.node_id;
        let b_id = b.node_id;
        g.add_node(a);
        g.add_node(b);
        // No edges
        assert!(g.shortest_path(a_id, b_id).is_none());
    }

    #[test]
    fn isa95_builder() {
        let mut g = TwinGraph::default();
        let site_id = Uuid::new_v4();
        let area_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        {
            let mut builder = Isa95Builder::new(&mut g);
            builder.add_space(site_id, "Plant Alpha", "Site", None);
            builder.add_space(area_id, "Assembly Area", "Area", Some(site_id));
            builder.locate_device(device_id, area_id);
        }

        assert_eq!(g.node_count(), 2); // site + area (device not added as graph node)
        assert_eq!(g.children(site_id), vec![area_id]);
    }

    #[test]
    fn degree_calculation() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let c = make_node("C");
        let a_id = a.node_id;
        let b_id = b.node_id;
        let c_id = c.node_id;
        g.add_node(a);
        g.add_node(b);
        g.add_node(c);
        g.add_edge(GraphEdge::new(a_id, b_id, "x", RelationshipCategory::Logical));
        g.add_edge(GraphEdge::new(a_id, c_id, "y", RelationshipCategory::Logical));
        assert_eq!(g.degree(a_id), 2);
        assert_eq!(g.degree(b_id), 1);
    }

    #[test]
    fn remove_node_cascades_edges() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let a_id = a.node_id;
        let b_id = b.node_id;
        g.add_node(a);
        g.add_node(b);
        g.add_edge(GraphEdge::new(a_id, b_id, "x", RelationshipCategory::Logical));
        assert_eq!(g.edge_count(), 1);
        g.remove_node(a_id);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn edges_by_category() {
        let mut g = TwinGraph::default();
        let a = make_node("A");
        let b = make_node("B");
        let a_id = a.node_id;
        let b_id = b.node_id;
        g.add_node(a);
        g.add_node(b);
        g.add_edge(GraphEdge::new(a_id, b_id, "contains", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(a_id, b_id, "monitors", RelationshipCategory::Logical));

        assert_eq!(g.edges_by_category(&RelationshipCategory::Spatial).len(), 1);
        assert_eq!(g.edges_by_category(&RelationshipCategory::Logical).len(), 1);
        assert_eq!(g.edges_by_category(&RelationshipCategory::Temporal).len(), 0);
    }
}
