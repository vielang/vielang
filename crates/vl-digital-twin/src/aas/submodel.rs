//! AAS Submodel framework — standardized data containers.
//!
//! Submodels represent different aspects of an asset:
//! - Nameplate: identity, manufacturer, serial number
//! - TechnicalData: specifications, ratings, dimensions
//! - OperationalData: runtime metrics, maintenance history
//! - Documentation: manuals, certificates, schematics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic identifier for standard submodel templates (IRDI/IRI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubmodelSemanticId {
    /// IDTA Nameplate submodel (0173-1#01-AHD205#001)
    Nameplate,
    /// IDTA Technical Data (0173-1#01-AHD206#001)
    TechnicalData,
    /// Operational Data (runtime metrics)
    OperationalData,
    /// Documentation submodel
    Documentation,
    /// Contact Information
    ContactInformation,
    /// Handover Documentation
    HandoverDocumentation,
    /// Custom semantic ID
    Custom(String),
}

/// A Submodel — a structured collection of SubmodelElements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submodel {
    pub id: String,
    pub id_short: String,
    pub semantic_id: SubmodelSemanticId,
    #[serde(default)]
    pub description: HashMap<String, String>,
    /// Submodel elements (ordered).
    pub elements: Vec<SubmodelElement>,
    /// Version/revision.
    #[serde(default)]
    pub version: Option<String>,
}

impl Submodel {
    pub fn new(id: &str, id_short: &str, semantic_id: SubmodelSemanticId) -> Self {
        Self {
            id: id.into(),
            id_short: id_short.into(),
            semantic_id,
            description: HashMap::new(),
            elements: Vec::new(),
            version: Some("1.0".into()),
        }
    }

    pub fn add_element(&mut self, element: SubmodelElement) {
        self.elements.push(element);
    }

    /// Find element by id_short path (e.g., "ManufacturerName").
    pub fn find_element(&self, id_short: &str) -> Option<&SubmodelElement> {
        self.elements.iter().find(|e| e.id_short() == id_short)
    }

    /// Find element value as string.
    pub fn get_string(&self, id_short: &str) -> Option<&str> {
        self.find_element(id_short).and_then(|e| {
            if let SubmodelElement::Property(p) = e {
                p.value.as_deref()
            } else {
                None
            }
        })
    }

    /// Get a nested element from a SubmodelElementCollection.
    pub fn get_nested(&self, collection_id: &str, element_id: &str) -> Option<&SubmodelElement> {
        self.find_element(collection_id).and_then(|e| {
            if let SubmodelElement::Collection(c) = e {
                c.elements.iter().find(|e2| e2.id_short() == element_id)
            } else {
                None
            }
        })
    }
}

/// A single element within a Submodel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmodelElement {
    /// A typed value with optional unit.
    Property(SmProperty),
    /// A collection of nested elements.
    Collection(SmCollection),
    /// A reference to a file/document.
    File(SmFile),
    /// A multi-language string.
    MultiLanguageProperty(SmMultiLang),
    /// A range (min–max pair).
    Range(SmRange),
    /// A BLOB (binary data, base64 encoded).
    Blob(SmBlob),
    /// A reference to another element.
    ReferenceElement(SmReference),
}

impl SubmodelElement {
    pub fn id_short(&self) -> &str {
        match self {
            Self::Property(p) => &p.id_short,
            Self::Collection(c) => &c.id_short,
            Self::File(f) => &f.id_short,
            Self::MultiLanguageProperty(m) => &m.id_short,
            Self::Range(r) => &r.id_short,
            Self::Blob(b) => &b.id_short,
            Self::ReferenceElement(r) => &r.id_short,
        }
    }
}

/// Property — a single typed value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmProperty {
    pub id_short: String,
    pub value_type: ValueType,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub semantic_id: Option<String>,
    #[serde(default)]
    pub description: HashMap<String, String>,
}

/// SubmodelElementCollection — a group of elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmCollection {
    pub id_short: String,
    pub elements: Vec<SubmodelElement>,
    #[serde(default)]
    pub semantic_id: Option<String>,
    #[serde(default)]
    pub description: HashMap<String, String>,
}

/// File reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmFile {
    pub id_short: String,
    pub content_type: String,
    pub value: String,
    #[serde(default)]
    pub description: HashMap<String, String>,
}

/// Multi-language property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmMultiLang {
    pub id_short: String,
    pub values: HashMap<String, String>,
}

/// Range — min/max pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmRange {
    pub id_short: String,
    pub value_type: ValueType,
    pub min: Option<String>,
    pub max: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
}

/// BLOB element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmBlob {
    pub id_short: String,
    pub content_type: String,
    /// Base64-encoded content.
    pub value: String,
}

/// Reference to another AAS element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmReference {
    pub id_short: String,
    pub target_type: String,
    pub target_id: String,
}

/// AAS value types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValueType {
    #[serde(rename = "xs:string")]
    String,
    #[serde(rename = "xs:boolean")]
    Boolean,
    #[serde(rename = "xs:integer")]
    Integer,
    #[serde(rename = "xs:long")]
    Long,
    #[serde(rename = "xs:float")]
    Float,
    #[serde(rename = "xs:double")]
    Double,
    #[serde(rename = "xs:decimal")]
    Decimal,
    #[serde(rename = "xs:dateTime")]
    DateTime,
    #[serde(rename = "xs:date")]
    Date,
    #[serde(rename = "xs:anyURI")]
    AnyUri,
}

// ── Builder helpers ──────────────────────────────────────────────────────────

/// Builder for TechnicalData submodel.
pub fn technical_data_submodel(id: &str) -> Submodel {
    Submodel::new(id, "TechnicalData", SubmodelSemanticId::TechnicalData)
}

/// Builder for OperationalData submodel.
pub fn operational_data_submodel(id: &str) -> Submodel {
    Submodel::new(id, "OperationalData", SubmodelSemanticId::OperationalData)
}

/// Builder for Documentation submodel.
pub fn documentation_submodel(id: &str) -> Submodel {
    let mut sm = Submodel::new(id, "Documentation", SubmodelSemanticId::Documentation);
    sm.add_element(SubmodelElement::Collection(SmCollection {
        id_short: "OperatingInstructions".into(),
        elements: Vec::new(),
        semantic_id: None,
        description: HashMap::new(),
    }));
    sm.add_element(SubmodelElement::Collection(SmCollection {
        id_short: "Certificates".into(),
        elements: Vec::new(),
        semantic_id: None,
        description: HashMap::new(),
    }));
    sm
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submodel_creation() {
        let sm = Submodel::new("sm:001", "Nameplate", SubmodelSemanticId::Nameplate);
        assert_eq!(sm.id, "sm:001");
        assert_eq!(sm.semantic_id, SubmodelSemanticId::Nameplate);
    }

    #[test]
    fn property_element() {
        let mut sm = Submodel::new("sm:002", "TechData", SubmodelSemanticId::TechnicalData);
        sm.add_element(SubmodelElement::Property(SmProperty {
            id_short: "MaxPower".into(),
            value_type: ValueType::Double,
            value: Some("5000.0".into()),
            unit: Some("kW".into()),
            semantic_id: None,
            description: HashMap::new(),
        }));

        let elem = sm.find_element("MaxPower").expect("should find");
        assert_eq!(elem.id_short(), "MaxPower");
        assert_eq!(sm.get_string("MaxPower"), Some("5000.0"));
    }

    #[test]
    fn collection_nesting() {
        let mut sm = Submodel::new("sm:003", "Contact", SubmodelSemanticId::ContactInformation);
        sm.add_element(SubmodelElement::Collection(SmCollection {
            id_short: "Address".into(),
            elements: vec![
                SubmodelElement::Property(SmProperty {
                    id_short: "Street".into(),
                    value_type: ValueType::String,
                    value: Some("123 Industrial Blvd".into()),
                    unit: None,
                    semantic_id: None,
                    description: HashMap::new(),
                }),
                SubmodelElement::Property(SmProperty {
                    id_short: "City".into(),
                    value_type: ValueType::String,
                    value: Some("Berlin".into()),
                    unit: None,
                    semantic_id: None,
                    description: HashMap::new(),
                }),
            ],
            semantic_id: None,
            description: HashMap::new(),
        }));

        let nested = sm.get_nested("Address", "City");
        assert!(nested.is_some());
        assert_eq!(nested.unwrap().id_short(), "City");
    }

    #[test]
    fn file_element() {
        let file = SubmodelElement::File(SmFile {
            id_short: "Manual".into(),
            content_type: "application/pdf".into(),
            value: "/docs/manual.pdf".into(),
            description: HashMap::new(),
        });
        assert_eq!(file.id_short(), "Manual");
    }

    #[test]
    fn range_element() {
        let range = SubmodelElement::Range(SmRange {
            id_short: "OperatingTemp".into(),
            value_type: ValueType::Double,
            min: Some("-20".into()),
            max: Some("85".into()),
            unit: Some("°C".into()),
        });
        assert_eq!(range.id_short(), "OperatingTemp");
    }

    #[test]
    fn json_roundtrip() {
        let mut sm = Submodel::new("sm:test", "Test", SubmodelSemanticId::TechnicalData);
        sm.add_element(SubmodelElement::Property(SmProperty {
            id_short: "Voltage".into(),
            value_type: ValueType::Double,
            value: Some("230".into()),
            unit: Some("V".into()),
            semantic_id: None,
            description: HashMap::new(),
        }));

        let json = serde_json::to_string(&sm).expect("serialize");
        let recovered: Submodel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(recovered.id, "sm:test");
        assert_eq!(recovered.elements.len(), 1);
    }
}
