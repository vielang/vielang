//! Phase 37 — Twin Graph: property graph for twin-to-twin relationships.
//!
//! Supports spatial (contains, locatedIn), logical (controls, monitors),
//! and hierarchical (ISA-95) relationships with traversal queries.

pub mod graph;
pub mod query;

pub use graph::*;
pub use query::*;
