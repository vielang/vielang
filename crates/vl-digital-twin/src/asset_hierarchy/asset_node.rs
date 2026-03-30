//! Phase 33 — Asset hierarchy: Site → Building → Floor → Zone → Device.
//!
//! `AssetTree` resource holds the non-device nodes.  `DeviceEntity.parent_asset_id`
//! links each device to its parent node.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Data model ─────────────────────────────────────────────────────────────────

/// A single non-device hierarchy node (Site / Building / Floor / Zone).
#[derive(Debug, Clone)]
pub struct AssetNodeData {
    pub asset_id:   Uuid,
    pub name:       String,
    pub asset_type: String,
    pub parent_id:  Option<Uuid>,
}

/// TOML-serializable form of `AssetNodeData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetNodeEntry {
    pub id:         String,
    pub name:       String,
    pub asset_type: String,
    #[serde(default)]
    pub parent_id:  Option<String>,
}

impl AssetNodeEntry {
    pub fn from_data(d: &AssetNodeData) -> Self {
        Self {
            id:         d.asset_id.to_string(),
            name:       d.name.clone(),
            asset_type: d.asset_type.clone(),
            parent_id:  d.parent_id.map(|u| u.to_string()),
        }
    }

    pub fn to_data(&self) -> Option<AssetNodeData> {
        Some(AssetNodeData {
            asset_id:   self.id.parse().ok()?,
            name:       self.name.clone(),
            asset_type: self.asset_type.clone(),
            parent_id:  self.parent_id.as_ref().and_then(|s| s.parse().ok()),
        })
    }
}

// ── Resource ──────────────────────────────────────────────────────────────────

/// Central asset tree state.
///
/// Holds all non-device hierarchy nodes and a mapping from `device_id` to the
/// parent asset node's `asset_id`.  Populated at startup from demo data or the
/// ThingsBoard REST API.
#[derive(Resource, Default, Debug)]
pub struct AssetTree {
    pub nodes:         Vec<AssetNodeData>,
    /// device_id → parent asset_id
    pub device_parent: HashMap<Uuid, Uuid>,
    pub loaded:        bool,
}

impl AssetTree {
    /// Direct children of `parent_id` (`None` = root nodes).
    pub fn children_of(&self, parent_id: Option<Uuid>) -> Vec<&AssetNodeData> {
        self.nodes.iter()
            .filter(|n| n.parent_id == parent_id)
            .collect()
    }

    /// All asset_ids in the subtree rooted at `root_id` (inclusive).
    pub fn subtree_ids(&self, root_id: Uuid) -> Vec<Uuid> {
        let mut result = vec![root_id];
        let mut queue  = vec![root_id];
        while let Some(cur) = queue.pop() {
            for child in self.children_of(Some(cur)) {
                result.push(child.asset_id);
                queue.push(child.asset_id);
            }
        }
        result
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> (AssetTree, Uuid, Uuid, Uuid) {
        let site_id     = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let floor_id    = Uuid::new_v4();
        let mut t = AssetTree::default();
        t.nodes = vec![
            AssetNodeData { asset_id: site_id,     name: "Site".into(),     asset_type: "Site".into(),     parent_id: None },
            AssetNodeData { asset_id: building_id, name: "Building".into(), asset_type: "Building".into(), parent_id: Some(site_id) },
            AssetNodeData { asset_id: floor_id,    name: "Floor 1".into(),  asset_type: "Floor".into(),    parent_id: Some(building_id) },
        ];
        (t, site_id, building_id, floor_id)
    }

    #[test]
    fn children_of_root() {
        let (t, _, _, _) = make_tree();
        let roots = t.children_of(None);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].asset_type, "Site");
    }

    #[test]
    fn children_of_site() {
        let (t, site_id, _, _) = make_tree();
        let children = t.children_of(Some(site_id));
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].asset_type, "Building");
    }

    #[test]
    fn subtree_includes_all_descendants() {
        let (t, site_id, _, _) = make_tree();
        let ids = t.subtree_ids(site_id);
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn subtree_leaf_is_single() {
        let (t, _, _, floor_id) = make_tree();
        let ids = t.subtree_ids(floor_id);
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn asset_node_entry_roundtrip() {
        let data = AssetNodeData {
            asset_id:   Uuid::nil(),
            name:       "Test Zone".into(),
            asset_type: "Zone".into(),
            parent_id:  None,
        };
        let entry   = AssetNodeEntry::from_data(&data);
        let decoded = entry.to_data().expect("decode");
        assert_eq!(decoded.name, "Test Zone");
        assert_eq!(decoded.asset_type, "Zone");
        assert!(decoded.parent_id.is_none());
    }

    #[test]
    fn asset_node_entry_with_parent() {
        let parent_id = Uuid::new_v4();
        let data = AssetNodeData {
            asset_id:   Uuid::new_v4(),
            name:       "Floor 2".into(),
            asset_type: "Floor".into(),
            parent_id:  Some(parent_id),
        };
        let entry   = AssetNodeEntry::from_data(&data);
        let decoded = entry.to_data().expect("decode");
        assert_eq!(decoded.parent_id, Some(parent_id));
    }

    // ── Default state ─────────────────────────────────────────────────────────

    #[test]
    fn asset_tree_default_is_empty() {
        let tree = AssetTree::default();
        assert!(tree.nodes.is_empty(),         "nodes should start empty");
        assert!(tree.device_parent.is_empty(), "device_parent should start empty");
        assert!(!tree.loaded,                  "loaded flag should be false");
    }

    // ── Device parent mapping ─────────────────────────────────────────────────

    #[test]
    fn device_parent_map_insert_and_lookup() {
        let mut tree      = AssetTree::default();
        let site_id       = Uuid::new_v4();
        let device_id     = Uuid::new_v4();
        tree.nodes.push(AssetNodeData {
            asset_id: site_id, name: "Site".into(),
            asset_type: "Site".into(), parent_id: None,
        });
        tree.device_parent.insert(device_id, site_id);
        assert_eq!(tree.device_parent.get(&device_id), Some(&site_id));
    }

    #[test]
    fn device_parent_unknown_device_returns_none() {
        let tree = AssetTree::default();
        assert!(tree.device_parent.get(&Uuid::new_v4()).is_none());
    }

    // ── Multiple siblings ─────────────────────────────────────────────────────

    #[test]
    fn children_of_returns_multiple_siblings() {
        let site_id = Uuid::new_v4();
        let b1_id   = Uuid::new_v4();
        let b2_id   = Uuid::new_v4();
        let b3_id   = Uuid::new_v4();
        let tree = AssetTree {
            nodes: vec![
                AssetNodeData { asset_id: site_id, name: "Site".into(), asset_type: "Site".into(), parent_id: None },
                AssetNodeData { asset_id: b1_id,   name: "B1".into(),   asset_type: "Building".into(), parent_id: Some(site_id) },
                AssetNodeData { asset_id: b2_id,   name: "B2".into(),   asset_type: "Building".into(), parent_id: Some(site_id) },
                AssetNodeData { asset_id: b3_id,   name: "B3".into(),   asset_type: "Building".into(), parent_id: Some(site_id) },
            ],
            ..Default::default()
        };
        let children = tree.children_of(Some(site_id));
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn children_of_leaf_returns_empty() {
        let (tree, _, _, floor_id) = make_tree();
        let children = tree.children_of(Some(floor_id));
        assert!(children.is_empty(), "leaf node should have no children");
    }

    // ── Deep hierarchy subtree ────────────────────────────────────────────────

    #[test]
    fn subtree_wide_and_deep() {
        // Site → Building1 → Floor1 + Floor2
        //      → Building2
        let site_id  = Uuid::new_v4();
        let b1_id    = Uuid::new_v4();
        let b2_id    = Uuid::new_v4();
        let f1_id    = Uuid::new_v4();
        let f2_id    = Uuid::new_v4();
        let tree = AssetTree {
            nodes: vec![
                AssetNodeData { asset_id: site_id, name: "Site".into(), asset_type: "Site".into(),     parent_id: None },
                AssetNodeData { asset_id: b1_id,   name: "B1".into(),   asset_type: "Building".into(), parent_id: Some(site_id) },
                AssetNodeData { asset_id: b2_id,   name: "B2".into(),   asset_type: "Building".into(), parent_id: Some(site_id) },
                AssetNodeData { asset_id: f1_id,   name: "F1".into(),   asset_type: "Floor".into(),    parent_id: Some(b1_id) },
                AssetNodeData { asset_id: f2_id,   name: "F2".into(),   asset_type: "Floor".into(),    parent_id: Some(b1_id) },
            ],
            ..Default::default()
        };
        // Site subtree = 5 nodes (site + 2 buildings + 2 floors)
        assert_eq!(tree.subtree_ids(site_id).len(), 5);
        // Building1 subtree = 3 nodes (b1 + f1 + f2)
        assert_eq!(tree.subtree_ids(b1_id).len(), 3);
        // Building2 subtree = 1 node (b2 only)
        assert_eq!(tree.subtree_ids(b2_id).len(), 1);
    }

    // ── AssetNodeEntry TOML roundtrip ─────────────────────────────────────────

    #[test]
    fn asset_node_entry_toml_roundtrip() {
        let entries = vec![
            AssetNodeEntry { id: Uuid::nil().to_string(), name: "HCM Site".into(),     asset_type: "Site".into(),     parent_id: None },
            AssetNodeEntry { id: Uuid::new_v4().to_string(), name: "Nhà máy A".into(), asset_type: "Building".into(), parent_id: Some(Uuid::nil().to_string()) },
        ];
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrap { assets: Vec<AssetNodeEntry> }
        let wrap = Wrap { assets: entries };
        let toml_str  = toml::to_string_pretty(&wrap).expect("serialize");
        let recovered: Wrap = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(recovered.assets.len(), 2);
        assert_eq!(recovered.assets[0].name, "HCM Site");
        assert_eq!(recovered.assets[0].parent_id, None);
        assert_eq!(recovered.assets[1].asset_type, "Building");
    }

    #[test]
    fn asset_node_entry_invalid_uuid_returns_none() {
        let entry = AssetNodeEntry {
            id:         "not-a-uuid".into(),
            name:       "Bad".into(),
            asset_type: "Site".into(),
            parent_id:  None,
        };
        assert!(entry.to_data().is_none(), "invalid UUID should return None");
    }
}
