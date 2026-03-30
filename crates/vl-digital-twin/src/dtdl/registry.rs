//! DTDL model registry — stores and resolves interface definitions.

use bevy::prelude::*;
use std::collections::HashMap;

use super::model::DtdlInterface;
use super::instance::{TwinInstance, RelationshipInstance};
use uuid::Uuid;

/// Central registry for DTDL interface definitions (models).
#[derive(Resource, Default)]
pub struct DtdlModelRegistry {
    /// DTMI → Interface definition
    models: HashMap<String, DtdlInterface>,
}

impl DtdlModelRegistry {
    /// Register a model. Returns the previous model if the DTMI was already registered.
    pub fn register(&mut self, interface: DtdlInterface) -> Option<DtdlInterface> {
        self.models.insert(interface.id.clone(), interface)
    }

    /// Look up a model by its DTMI.
    pub fn get(&self, dtmi: &str) -> Option<&DtdlInterface> {
        self.models.get(dtmi)
    }

    /// Remove a model by DTMI.
    pub fn remove(&mut self, dtmi: &str) -> Option<DtdlInterface> {
        self.models.remove(dtmi)
    }

    /// All registered DTMIs.
    pub fn model_ids(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered models.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// Resolve the full inheritance chain for an interface (extends).
    /// Returns all ancestor interfaces in order (closest parent first).
    pub fn resolve_extends(&self, dtmi: &str) -> Vec<&DtdlInterface> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.resolve_extends_inner(dtmi, &mut chain, &mut visited);
        chain
    }

    fn resolve_extends_inner<'a>(
        &'a self,
        dtmi: &str,
        chain: &mut Vec<&'a DtdlInterface>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(dtmi) {
            return; // circular reference protection
        }
        visited.insert(dtmi.to_string());

        if let Some(iface) = self.models.get(dtmi) {
            for parent_dtmi in &iface.extends {
                if let Some(parent) = self.models.get(parent_dtmi.as_str()) {
                    chain.push(parent);
                    self.resolve_extends_inner(parent_dtmi, chain, visited);
                }
            }
        }
    }

    /// Populate with VíeLang built-in industrial models.
    pub fn with_builtins(mut self) -> Self {
        self.register(DtdlInterface::temperature_sensor());
        self.register(DtdlInterface::wind_turbine());
        self.register(DtdlInterface::space());
        self.register(DtdlInterface::pump());
        self
    }
}

/// Central registry for live twin instances.
#[derive(Resource, Default)]
pub struct TwinInstanceRegistry {
    /// twin_id → TwinInstance
    instances: HashMap<Uuid, TwinInstance>,
    /// Relationship instances indexed by source twin.
    relationships_by_source: HashMap<Uuid, Vec<RelationshipInstance>>,
    /// Reverse index: target_id → relationship instances.
    relationships_by_target: HashMap<Uuid, Vec<RelationshipInstance>>,
}

impl TwinInstanceRegistry {
    /// Register or update a twin instance.
    pub fn upsert(&mut self, instance: TwinInstance) {
        self.instances.insert(instance.twin_id, instance);
    }

    /// Get a twin by ID.
    pub fn get(&self, twin_id: Uuid) -> Option<&TwinInstance> {
        self.instances.get(&twin_id)
    }

    /// Get a mutable twin by ID.
    pub fn get_mut(&mut self, twin_id: Uuid) -> Option<&mut TwinInstance> {
        self.instances.get_mut(&twin_id)
    }

    /// Remove a twin and all its relationships.
    pub fn remove(&mut self, twin_id: Uuid) -> Option<TwinInstance> {
        self.relationships_by_source.remove(&twin_id);
        self.relationships_by_target.remove(&twin_id);
        // Also clean reverse references
        for rels in self.relationships_by_source.values_mut() {
            rels.retain(|r| r.target_id != twin_id);
        }
        for rels in self.relationships_by_target.values_mut() {
            rels.retain(|r| r.source_id != twin_id);
        }
        self.instances.remove(&twin_id)
    }

    /// All twin IDs.
    pub fn twin_ids(&self) -> Vec<Uuid> {
        self.instances.keys().copied().collect()
    }

    /// All twins conforming to a specific model.
    pub fn twins_by_model(&self, model_id: &str) -> Vec<&TwinInstance> {
        self.instances
            .values()
            .filter(|t| t.model_id == model_id)
            .collect()
    }

    /// All twins for a given tenant.
    pub fn twins_by_tenant(&self, tenant_id: Uuid) -> Vec<&TwinInstance> {
        self.instances
            .values()
            .filter(|t| t.tenant_id == tenant_id)
            .collect()
    }

    /// Number of registered twins.
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    // ── Relationships ────────────────────────────────────────────────────────

    /// Add a relationship between two twins.
    pub fn add_relationship(&mut self, rel: RelationshipInstance) {
        self.relationships_by_target
            .entry(rel.target_id)
            .or_default()
            .push(rel.clone());
        self.relationships_by_source
            .entry(rel.source_id)
            .or_default()
            .push(rel);
    }

    /// Get all outgoing relationships from a twin.
    pub fn outgoing_relationships(&self, source_id: Uuid) -> &[RelationshipInstance] {
        self.relationships_by_source
            .get(&source_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all incoming relationships to a twin.
    pub fn incoming_relationships(&self, target_id: Uuid) -> &[RelationshipInstance] {
        self.relationships_by_target
            .get(&target_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get relationships of a specific type from a twin.
    pub fn relationships_by_name(&self, source_id: Uuid, name: &str) -> Vec<&RelationshipInstance> {
        self.outgoing_relationships(source_id)
            .iter()
            .filter(|r| r.name == name)
            .collect()
    }

    /// Remove a specific relationship by ID.
    pub fn remove_relationship(&mut self, relationship_id: Uuid) -> bool {
        let mut found = false;
        for rels in self.relationships_by_source.values_mut() {
            let before = rels.len();
            rels.retain(|r| r.relationship_id != relationship_id);
            if rels.len() < before {
                found = true;
            }
        }
        for rels in self.relationships_by_target.values_mut() {
            rels.retain(|r| r.relationship_id != relationship_id);
        }
        found
    }

    /// Traverse the graph: find all twins reachable from `start_id` via the given
    /// relationship name (BFS). Max depth prevents infinite loops.
    pub fn traverse(
        &self,
        start_id: Uuid,
        relationship_name: &str,
        max_depth: usize,
    ) -> Vec<(Uuid, usize)> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_id, 0usize));
        visited.insert(start_id);

        while let Some((current, depth)) = queue.pop_front() {
            if depth > 0 {
                result.push((current, depth));
            }
            if depth >= max_depth {
                continue;
            }
            for rel in self.outgoing_relationships(current) {
                if rel.name == relationship_name && !visited.contains(&rel.target_id) {
                    visited.insert(rel.target_id);
                    queue.push_back((rel.target_id, depth + 1));
                }
            }
        }

        result
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_registry_builtins() {
        let reg = DtdlModelRegistry::default().with_builtins();
        assert_eq!(reg.len(), 4);
        assert!(reg.get("dtmi:vielang:TemperatureSensor;1").is_some());
        assert!(reg.get("dtmi:vielang:WindTurbine;1").is_some());
        assert!(reg.get("dtmi:vielang:Space;1").is_some());
        assert!(reg.get("dtmi:vielang:Pump;1").is_some());
    }

    #[test]
    fn model_registry_register_and_remove() {
        let mut reg = DtdlModelRegistry::default();
        let iface = DtdlInterface::temperature_sensor();
        assert!(reg.register(iface).is_none());
        assert_eq!(reg.len(), 1);
        assert!(reg.remove("dtmi:vielang:TemperatureSensor;1").is_some());
        assert!(reg.is_empty());
    }

    #[test]
    fn twin_instance_registry_crud() {
        let mut reg = TwinInstanceRegistry::default();
        let tid = Uuid::new_v4();
        let inst = TwinInstance::new(tid, "dtmi:test:A;1", "Twin A", Uuid::nil());
        reg.upsert(inst);
        assert_eq!(reg.len(), 1);
        assert!(reg.get(tid).is_some());
        assert_eq!(reg.twins_by_model("dtmi:test:A;1").len(), 1);
        reg.remove(tid);
        assert!(reg.is_empty());
    }

    #[test]
    fn relationship_management() {
        let mut reg = TwinInstanceRegistry::default();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        reg.upsert(TwinInstance::new(a, "dtmi:test:A;1", "A", Uuid::nil()));
        reg.upsert(TwinInstance::new(b, "dtmi:test:B;1", "B", Uuid::nil()));

        let rel = RelationshipInstance::new(a, b, "contains");
        let rel_id = rel.relationship_id;
        reg.add_relationship(rel);

        assert_eq!(reg.outgoing_relationships(a).len(), 1);
        assert_eq!(reg.incoming_relationships(b).len(), 1);
        assert_eq!(reg.relationships_by_name(a, "contains").len(), 1);
        assert_eq!(reg.relationships_by_name(a, "other").len(), 0);

        assert!(reg.remove_relationship(rel_id));
        assert_eq!(reg.outgoing_relationships(a).len(), 0);
    }

    #[test]
    fn graph_traversal() {
        let mut reg = TwinInstanceRegistry::default();
        let site = Uuid::new_v4();
        let building = Uuid::new_v4();
        let floor = Uuid::new_v4();
        let room = Uuid::new_v4();

        for (id, name) in [(site, "Site"), (building, "Building"), (floor, "Floor"), (room, "Room")] {
            reg.upsert(TwinInstance::new(id, "dtmi:vielang:Space;1", name, Uuid::nil()));
        }

        reg.add_relationship(RelationshipInstance::new(site, building, "contains"));
        reg.add_relationship(RelationshipInstance::new(building, floor, "contains"));
        reg.add_relationship(RelationshipInstance::new(floor, room, "contains"));

        let reachable = reg.traverse(site, "contains", 10);
        assert_eq!(reachable.len(), 3); // building, floor, room
        assert!(reachable.iter().any(|(id, depth)| *id == building && *depth == 1));
        assert!(reachable.iter().any(|(id, depth)| *id == room && *depth == 3));
    }

    #[test]
    fn traverse_respects_max_depth() {
        let mut reg = TwinInstanceRegistry::default();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        for (id, name) in [(a, "A"), (b, "B"), (c, "C")] {
            reg.upsert(TwinInstance::new(id, "dtmi:test:X;1", name, Uuid::nil()));
        }
        reg.add_relationship(RelationshipInstance::new(a, b, "next"));
        reg.add_relationship(RelationshipInstance::new(b, c, "next"));

        let depth1 = reg.traverse(a, "next", 1);
        assert_eq!(depth1.len(), 1); // only b

        let depth2 = reg.traverse(a, "next", 2);
        assert_eq!(depth2.len(), 2); // b and c
    }
}
