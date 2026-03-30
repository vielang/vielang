//! Graph query language — SQL-like queries over the twin graph.

use super::graph::{TwinGraph, GraphEdge, GraphNode, RelationshipCategory};
use uuid::Uuid;

/// A graph query — fluent API for querying the twin graph.
///
/// Example usage:
/// ```ignore
/// let results = GraphQuery::new()
///     .from_model("dtmi:vielang:WindTurbine;1")
///     .traverse("locatedIn", Direction::Outgoing)
///     .where_property("space_type", |v| v == &json!("Site"))
///     .execute(&graph);
/// ```
pub struct GraphQuery {
    /// Starting node filter.
    start: StartFilter,
    /// Traversal steps.
    steps: Vec<TraversalStep>,
    /// Maximum results.
    limit: Option<usize>,
}

enum StartFilter {
    All,
    ByModel(String),
    ById(Uuid),
    ByLabel(String),
    ByProperty(String, serde_json::Value),
}

struct TraversalStep {
    rel_type: Option<String>,
    category: Option<RelationshipCategory>,
    direction: Direction,
    min_depth: usize,
    max_depth: usize,
    node_filter: Option<Box<dyn Fn(&GraphNode) -> bool + Send + Sync>>,
}

/// Traversal direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

/// A single query result row.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub node_id: Uuid,
    pub label: String,
    pub model_id: String,
    pub depth: usize,
    pub path: Vec<Uuid>,
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

impl GraphQuery {
    pub fn new() -> Self {
        Self {
            start: StartFilter::All,
            steps: Vec::new(),
            limit: None,
        }
    }

    /// Start from nodes matching a specific model ID.
    pub fn from_model(mut self, model_id: &str) -> Self {
        self.start = StartFilter::ByModel(model_id.into());
        self
    }

    /// Start from a specific node by ID.
    pub fn from_node(mut self, node_id: Uuid) -> Self {
        self.start = StartFilter::ById(node_id);
        self
    }

    /// Start from nodes matching a label.
    pub fn from_label(mut self, label: &str) -> Self {
        self.start = StartFilter::ByLabel(label.into());
        self
    }

    /// Start from nodes with a specific property value.
    pub fn from_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.start = StartFilter::ByProperty(key.into(), value);
        self
    }

    /// Add a traversal step following a relationship type.
    pub fn traverse(mut self, rel_type: &str, direction: Direction) -> Self {
        self.steps.push(TraversalStep {
            rel_type: Some(rel_type.into()),
            category: None,
            direction,
            min_depth: 1,
            max_depth: 1,
            node_filter: None,
        });
        self
    }

    /// Add a traversal step following any relationship of a category.
    pub fn traverse_category(mut self, category: RelationshipCategory, direction: Direction) -> Self {
        self.steps.push(TraversalStep {
            rel_type: None,
            category: Some(category),
            direction,
            min_depth: 1,
            max_depth: 1,
            node_filter: None,
        });
        self
    }

    /// Add a recursive traversal (variable depth).
    pub fn traverse_recursive(mut self, rel_type: &str, direction: Direction, min_depth: usize, max_depth: usize) -> Self {
        self.steps.push(TraversalStep {
            rel_type: Some(rel_type.into()),
            category: None,
            direction,
            min_depth,
            max_depth,
            node_filter: None,
        });
        self
    }

    /// Limit the number of results.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Execute the query against a graph.
    pub fn execute(&self, graph: &TwinGraph) -> Vec<QueryResult> {
        // Step 1: Find starting nodes
        let start_nodes: Vec<Uuid> = match &self.start {
            StartFilter::All => graph.all_nodes().map(|n| n.node_id).collect(),
            StartFilter::ByModel(model) => graph.nodes_by_model(model).iter().map(|n| n.node_id).collect(),
            StartFilter::ById(id) => {
                if graph.get_node(*id).is_some() {
                    vec![*id]
                } else {
                    vec![]
                }
            }
            StartFilter::ByLabel(label) => {
                graph.all_nodes()
                    .filter(|n| n.label == *label)
                    .map(|n| n.node_id)
                    .collect()
            }
            StartFilter::ByProperty(key, value) => {
                graph.all_nodes()
                    .filter(|n| n.properties.get(key) == Some(value))
                    .map(|n| n.node_id)
                    .collect()
            }
        };

        if self.steps.is_empty() {
            // No traversal — just return starting nodes
            let mut results: Vec<QueryResult> = start_nodes.iter()
                .filter_map(|id| graph.get_node(*id))
                .map(|n| QueryResult {
                    node_id: n.node_id,
                    label: n.label.clone(),
                    model_id: n.model_id.clone(),
                    depth: 0,
                    path: vec![n.node_id],
                    properties: n.properties.clone(),
                })
                .collect();
            if let Some(limit) = self.limit {
                results.truncate(limit);
            }
            return results;
        }

        // Step 2: Execute traversal steps
        let mut current_nodes: Vec<(Uuid, usize, Vec<Uuid>)> = start_nodes.iter()
            .map(|id| (*id, 0, vec![*id]))
            .collect();

        for step in &self.steps {
            let mut next_nodes = Vec::new();

            for (node_id, _depth, path) in &current_nodes {
                let reached = self.execute_step(graph, *node_id, step);
                for (target_id, target_depth) in reached {
                    if target_depth >= step.min_depth {
                        let mut new_path = path.clone();
                        new_path.push(target_id);
                        next_nodes.push((target_id, target_depth, new_path));
                    }
                }
            }

            current_nodes = next_nodes;
        }

        // Step 3: Build results
        let mut results: Vec<QueryResult> = current_nodes.iter()
            .filter_map(|(id, depth, path)| {
                graph.get_node(*id).map(|n| QueryResult {
                    node_id: n.node_id,
                    label: n.label.clone(),
                    model_id: n.model_id.clone(),
                    depth: *depth,
                    path: path.clone(),
                    properties: n.properties.clone(),
                })
            })
            .collect();

        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        results
    }

    fn execute_step(&self, graph: &TwinGraph, start: Uuid, step: &TraversalStep) -> Vec<(Uuid, usize)> {
        let mut results = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0usize));
        visited.insert(start);

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= step.max_depth {
                continue;
            }

            let edges = match step.direction {
                Direction::Outgoing => graph.outgoing_edges(current, step.rel_type.as_deref()),
                Direction::Incoming => graph.incoming_edges(current, step.rel_type.as_deref()),
                Direction::Both => {
                    let mut all = graph.outgoing_edges(current, step.rel_type.as_deref());
                    all.extend(graph.incoming_edges(current, step.rel_type.as_deref()));
                    all
                }
            };

            // Filter by category if specified
            let edges: Vec<&GraphEdge> = if let Some(cat) = &step.category {
                edges.into_iter().filter(|e| &e.category == cat).collect()
            } else {
                edges
            };

            for edge in edges {
                let target = match step.direction {
                    Direction::Incoming => edge.source_id,
                    _ => edge.target_id,
                };

                if !visited.insert(target) {
                    continue;
                }

                let new_depth = depth + 1;

                // Apply node filter
                let passes = step.node_filter.as_ref().map_or(true, |f| {
                    graph.get_node(target).map_or(false, |n| f(n))
                });

                if passes {
                    results.push((target, new_depth));
                    queue.push_back((target, new_depth));
                }
            }
        }

        results
    }
}

impl Default for GraphQuery {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::graph::*;
    use std::collections::HashMap;

    fn build_test_graph() -> TwinGraph {
        let mut g = TwinGraph::default();
        let site = Uuid::new_v4();
        let area1 = Uuid::new_v4();
        let area2 = Uuid::new_v4();
        let turbine = Uuid::new_v4();
        let pump = Uuid::new_v4();

        for (id, label, model) in [
            (site, "Site Alpha", "dtmi:vielang:Space;1"),
            (area1, "Area 1", "dtmi:vielang:Space;1"),
            (area2, "Area 2", "dtmi:vielang:Space;1"),
            (turbine, "WT-001", "dtmi:vielang:WindTurbine;1"),
            (pump, "Pump-001", "dtmi:vielang:Pump;1"),
        ] {
            let mut props = HashMap::new();
            if model.contains("Space") {
                props.insert("space_type".into(), serde_json::json!(label.split_whitespace().next().unwrap_or("")));
            }
            g.add_node(GraphNode {
                node_id: id, label: label.into(), model_id: model.into(), properties: props,
            });
        }

        g.add_edge(GraphEdge::new(site, area1, "contains", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(site, area2, "contains", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(turbine, area1, "locatedIn", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(pump, area2, "locatedIn", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(pump, turbine, "monitors", RelationshipCategory::Logical));

        g
    }

    #[test]
    fn query_all_nodes() {
        let g = build_test_graph();
        let results = GraphQuery::new().execute(&g);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn query_by_model() {
        let g = build_test_graph();
        let results = GraphQuery::new()
            .from_model("dtmi:vielang:Space;1")
            .execute(&g);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn query_with_traversal() {
        let g = build_test_graph();
        // Find all nodes that turbines are locatedIn
        let results = GraphQuery::new()
            .from_model("dtmi:vielang:WindTurbine;1")
            .traverse("locatedIn", Direction::Outgoing)
            .execute(&g);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Area 1");
    }

    #[test]
    fn query_recursive_traversal() {
        let mut g = TwinGraph::default();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        for (id, label) in [(a, "A"), (b, "B"), (c, "C")] {
            g.add_node(GraphNode {
                node_id: id, label: label.into(), model_id: "test".into(), properties: HashMap::new(),
            });
        }
        g.add_edge(GraphEdge::new(a, b, "contains", RelationshipCategory::Spatial));
        g.add_edge(GraphEdge::new(b, c, "contains", RelationshipCategory::Spatial));

        let results = GraphQuery::new()
            .from_node(a)
            .traverse_recursive("contains", Direction::Outgoing, 1, 10)
            .execute(&g);
        assert_eq!(results.len(), 2); // b and c
    }

    #[test]
    fn query_with_limit() {
        let g = build_test_graph();
        let results = GraphQuery::new().limit(2).execute(&g);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_by_label() {
        let g = build_test_graph();
        let results = GraphQuery::new()
            .from_label("WT-001")
            .execute(&g);
        assert_eq!(results.len(), 1);
    }
}
