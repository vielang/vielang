//! Asset Administration Shell — the core AAS structure.
//!
//! An AAS wraps a physical or digital asset with standardized submodels
//! that describe its identity, capabilities, and operational state.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::submodel::Submodel;

/// Unique identifier for an AAS asset (IRI or IRDI format).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetId(pub String);

impl AssetId {
    /// Create from a URN.
    pub fn from_urn(namespace: &str, id: &str) -> Self {
        Self(format!("urn:{namespace}:asset:{id}"))
    }

    /// Create from a UUID.
    pub fn from_uuid(id: Uuid) -> Self {
        Self(format!("urn:vielang:asset:{id}"))
    }
}

/// The kind of asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetKind {
    /// Physical hardware instance.
    Instance,
    /// Asset type / class (template).
    Type,
    /// Not yet determined.
    NotApplicable,
}

impl Default for AssetKind {
    fn default() -> Self {
        Self::Instance
    }
}

/// Asset information embedded in the AAS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInformation {
    pub asset_kind: AssetKind,
    pub global_asset_id: AssetId,
    #[serde(default)]
    pub specific_asset_ids: Vec<SpecificAssetId>,
    #[serde(default)]
    pub default_thumbnail: Option<String>,
}

/// A domain-specific identifier for an asset (e.g., serial number, ERP ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificAssetId {
    pub name: String,
    pub value: String,
    /// Who issued this ID (e.g., "manufacturer", "operator").
    #[serde(default)]
    pub external_subject: Option<String>,
}

/// The Asset Administration Shell itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAdministrationShell {
    /// Unique AAS identifier.
    pub id: String,
    /// Human-readable short name.
    pub id_short: String,
    /// Description in multiple languages.
    #[serde(default)]
    pub description: HashMap<String, String>,
    /// Asset this AAS represents.
    pub asset_information: AssetInformation,
    /// References to submodels (by submodel ID).
    #[serde(default)]
    pub submodel_refs: Vec<String>,
    /// Administration metadata.
    #[serde(default)]
    pub administration: Option<AdministrativeInformation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeInformation {
    pub version: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub creator: Option<String>,
}

impl AssetAdministrationShell {
    pub fn new(id: &str, id_short: &str, asset_id: AssetId, asset_kind: AssetKind) -> Self {
        Self {
            id: id.into(),
            id_short: id_short.into(),
            description: HashMap::new(),
            asset_information: AssetInformation {
                asset_kind,
                global_asset_id: asset_id,
                specific_asset_ids: Vec::new(),
                default_thumbnail: None,
            },
            submodel_refs: Vec::new(),
            administration: Some(AdministrativeInformation {
                version: "1.0".into(),
                revision: Some("0".into()),
                creator: Some("VíeLang Digital Twin".into()),
            }),
        }
    }

    /// Add a reference to a submodel.
    pub fn add_submodel_ref(&mut self, submodel_id: &str) {
        if !self.submodel_refs.contains(&submodel_id.to_string()) {
            self.submodel_refs.push(submodel_id.into());
        }
    }

    /// Add a description in a language.
    pub fn with_description(mut self, lang: &str, text: &str) -> Self {
        self.description.insert(lang.into(), text.into());
        self
    }

    /// Add a specific asset identifier.
    pub fn add_specific_id(&mut self, name: &str, value: &str, issuer: Option<&str>) {
        self.asset_information.specific_asset_ids.push(SpecificAssetId {
            name: name.into(),
            value: value.into(),
            external_subject: issuer.map(|s| s.into()),
        });
    }
}

/// Central AAS registry — Bevy resource holding all shells and submodels.
#[derive(Resource, Default)]
pub struct AasRegistry {
    /// AAS ID → Shell
    pub shells: HashMap<String, AssetAdministrationShell>,
    /// Submodel ID → Submodel
    pub submodels: HashMap<String, Submodel>,
    /// Asset ID → AAS ID (reverse lookup)
    asset_to_aas: HashMap<String, String>,
}

impl AasRegistry {
    pub fn register_shell(&mut self, shell: AssetAdministrationShell) {
        self.asset_to_aas.insert(
            shell.asset_information.global_asset_id.0.clone(),
            shell.id.clone(),
        );
        self.shells.insert(shell.id.clone(), shell);
    }

    pub fn register_submodel(&mut self, submodel: Submodel) {
        self.submodels.insert(submodel.id.clone(), submodel);
    }

    pub fn get_shell(&self, aas_id: &str) -> Option<&AssetAdministrationShell> {
        self.shells.get(aas_id)
    }

    pub fn get_submodel(&self, submodel_id: &str) -> Option<&Submodel> {
        self.submodels.get(submodel_id)
    }

    /// Find AAS by asset ID.
    pub fn find_by_asset(&self, asset_id: &str) -> Option<&AssetAdministrationShell> {
        self.asset_to_aas.get(asset_id)
            .and_then(|aas_id| self.shells.get(aas_id))
    }

    /// Get all submodels referenced by an AAS.
    pub fn submodels_for_shell(&self, aas_id: &str) -> Vec<&Submodel> {
        self.shells.get(aas_id)
            .map(|shell| {
                shell.submodel_refs.iter()
                    .filter_map(|ref_id| self.submodels.get(ref_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Total number of shells.
    pub fn shell_count(&self) -> usize {
        self.shells.len()
    }

    /// Total number of submodels.
    pub fn submodel_count(&self) -> usize {
        self.submodels.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_id_from_uuid() {
        let id = AssetId::from_uuid(Uuid::nil());
        assert!(id.0.starts_with("urn:vielang:asset:"));
    }

    #[test]
    fn shell_creation() {
        let shell = AssetAdministrationShell::new(
            "aas:pump-001",
            "Pump001",
            AssetId::from_urn("factory", "pump-001"),
            AssetKind::Instance,
        ).with_description("en", "Main coolant pump");

        assert_eq!(shell.id, "aas:pump-001");
        assert_eq!(shell.description.get("en").unwrap(), "Main coolant pump");
    }

    #[test]
    fn registry_crud() {
        let mut reg = AasRegistry::default();

        let asset_id = AssetId::from_urn("test", "device-001");
        let mut shell = AssetAdministrationShell::new(
            "aas:001", "Device001", asset_id.clone(), AssetKind::Instance,
        );
        shell.add_submodel_ref("sm:nameplate:001");
        reg.register_shell(shell);

        assert_eq!(reg.shell_count(), 1);
        assert!(reg.get_shell("aas:001").is_some());
        assert!(reg.find_by_asset(&asset_id.0).is_some());
    }

    #[test]
    fn specific_asset_ids() {
        let mut shell = AssetAdministrationShell::new(
            "aas:001", "Device001",
            AssetId::from_urn("test", "001"),
            AssetKind::Instance,
        );
        shell.add_specific_id("serialNumber", "SN-12345", Some("manufacturer"));
        shell.add_specific_id("erpId", "ERP-789", Some("operator"));

        assert_eq!(shell.asset_information.specific_asset_ids.len(), 2);
    }
}
