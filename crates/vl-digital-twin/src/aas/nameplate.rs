//! IDTA Nameplate Submodel — asset identity information.
//!
//! Based on IDTA 02006-2-0 "Digital Nameplate for Industrial Equipment".
//! Contains manufacturer info, serial number, product designation, markings.

use super::submodel::*;
use std::collections::HashMap;

/// Build a standard Nameplate submodel for an industrial asset.
pub fn build_nameplate(
    submodel_id: &str,
    manufacturer_name: &str,
    manufacturer_product_designation: &str,
    serial_number: &str,
) -> Submodel {
    let mut sm = Submodel::new(submodel_id, "Nameplate", SubmodelSemanticId::Nameplate);

    // ── Manufacturer info ────────────────────────────────────────────────────
    sm.add_element(SubmodelElement::MultiLanguageProperty(SmMultiLang {
        id_short: "ManufacturerName".into(),
        values: {
            let mut m = HashMap::new();
            m.insert("en".into(), manufacturer_name.into());
            m
        },
    }));

    sm.add_element(SubmodelElement::MultiLanguageProperty(SmMultiLang {
        id_short: "ManufacturerProductDesignation".into(),
        values: {
            let mut m = HashMap::new();
            m.insert("en".into(), manufacturer_product_designation.into());
            m
        },
    }));

    sm.add_element(SubmodelElement::Property(SmProperty {
        id_short: "SerialNumber".into(),
        value_type: ValueType::String,
        value: Some(serial_number.into()),
        unit: None,
        semantic_id: Some("0173-1#02-AAM556#002".into()),
        description: HashMap::new(),
    }));

    // ── Physical address (placeholder collection) ────────────────────────────
    sm.add_element(SubmodelElement::Collection(SmCollection {
        id_short: "ContactInformation".into(),
        elements: vec![
            SubmodelElement::Property(SmProperty {
                id_short: "Street".into(),
                value_type: ValueType::String,
                value: None,
                unit: None,
                semantic_id: None,
                description: HashMap::new(),
            }),
            SubmodelElement::Property(SmProperty {
                id_short: "CityTown".into(),
                value_type: ValueType::String,
                value: None,
                unit: None,
                semantic_id: None,
                description: HashMap::new(),
            }),
            SubmodelElement::Property(SmProperty {
                id_short: "NationalCode".into(),
                value_type: ValueType::String,
                value: None,
                unit: None,
                semantic_id: None,
                description: HashMap::new(),
            }),
        ],
        semantic_id: Some("0173-1#01-ADR650#006".into()),
        description: HashMap::new(),
    }));

    // ── Year of construction ─────────────────────────────────────────────────
    sm.add_element(SubmodelElement::Property(SmProperty {
        id_short: "YearOfConstruction".into(),
        value_type: ValueType::String,
        value: None,
        unit: None,
        semantic_id: Some("0173-1#02-AAP906#001".into()),
        description: HashMap::new(),
    }));

    // ── Markings collection ──────────────────────────────────────────────────
    sm.add_element(SubmodelElement::Collection(SmCollection {
        id_short: "Markings".into(),
        elements: Vec::new(),
        semantic_id: None,
        description: HashMap::new(),
    }));

    sm
}

/// Builder for adding fields to an existing nameplate.
pub struct NameplateBuilder {
    submodel: Submodel,
}

impl NameplateBuilder {
    pub fn new(submodel_id: &str, manufacturer: &str, product: &str, serial: &str) -> Self {
        Self {
            submodel: build_nameplate(submodel_id, manufacturer, product, serial),
        }
    }

    pub fn year_of_construction(mut self, year: &str) -> Self {
        if let Some(SubmodelElement::Property(p)) = self.submodel.elements.iter_mut()
            .find(|e| e.id_short() == "YearOfConstruction")
        {
            p.value = Some(year.into());
        }
        self
    }

    pub fn batch_id(mut self, batch: &str) -> Self {
        self.submodel.add_element(SubmodelElement::Property(SmProperty {
            id_short: "BatchId".into(),
            value_type: ValueType::String,
            value: Some(batch.into()),
            unit: None,
            semantic_id: Some("0173-1#02-AAQ196#001".into()),
            description: HashMap::new(),
        }));
        self
    }

    pub fn hardware_version(mut self, version: &str) -> Self {
        self.submodel.add_element(SubmodelElement::Property(SmProperty {
            id_short: "HardwareVersion".into(),
            value_type: ValueType::String,
            value: Some(version.into()),
            unit: None,
            semantic_id: None,
            description: HashMap::new(),
        }));
        self
    }

    pub fn firmware_version(mut self, version: &str) -> Self {
        self.submodel.add_element(SubmodelElement::Property(SmProperty {
            id_short: "FirmwareVersion".into(),
            value_type: ValueType::String,
            value: Some(version.into()),
            unit: None,
            semantic_id: None,
            description: HashMap::new(),
        }));
        self
    }

    pub fn country_of_origin(mut self, country: &str) -> Self {
        self.submodel.add_element(SubmodelElement::Property(SmProperty {
            id_short: "CountryOfOrigin".into(),
            value_type: ValueType::String,
            value: Some(country.into()),
            unit: None,
            semantic_id: Some("0173-1#02-AAO259#004".into()),
            description: HashMap::new(),
        }));
        self
    }

    pub fn company_logo(mut self, path: &str) -> Self {
        self.submodel.add_element(SubmodelElement::File(SmFile {
            id_short: "CompanyLogo".into(),
            content_type: "image/png".into(),
            value: path.into(),
            description: HashMap::new(),
        }));
        self
    }

    pub fn marking(mut self, marking_name: &str, marking_file: &str) -> Self {
        if let Some(SubmodelElement::Collection(c)) = self.submodel.elements.iter_mut()
            .find(|e| e.id_short() == "Markings")
        {
            c.elements.push(SubmodelElement::Collection(SmCollection {
                id_short: marking_name.into(),
                elements: vec![
                    SubmodelElement::File(SmFile {
                        id_short: "MarkingFile".into(),
                        content_type: "image/png".into(),
                        value: marking_file.into(),
                        description: HashMap::new(),
                    }),
                ],
                semantic_id: None,
                description: HashMap::new(),
            }));
        }
        self
    }

    pub fn build(self) -> Submodel {
        self.submodel
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_basic_nameplate() {
        let np = build_nameplate(
            "sm:np:001",
            "Siemens AG",
            "SIMOTICS GP 1LE1",
            "SN-2024-001234",
        );

        assert_eq!(np.id, "sm:np:001");
        assert_eq!(np.semantic_id, SubmodelSemanticId::Nameplate);
        assert_eq!(np.get_string("SerialNumber"), Some("SN-2024-001234"));
    }

    #[test]
    fn nameplate_builder_chain() {
        let np = NameplateBuilder::new(
            "sm:np:002", "ABB", "ACS880 Drive", "SN-002",
        )
            .year_of_construction("2024")
            .batch_id("BATCH-42")
            .hardware_version("3.1")
            .firmware_version("7.2.1")
            .country_of_origin("DE")
            .build();

        assert_eq!(np.get_string("SerialNumber"), Some("SN-002"));
        assert!(np.find_element("BatchId").is_some());
        assert!(np.find_element("HardwareVersion").is_some());
        assert!(np.find_element("FirmwareVersion").is_some());
        assert!(np.find_element("CountryOfOrigin").is_some());
    }

    #[test]
    fn nameplate_has_contact_collection() {
        let np = build_nameplate("sm:np:003", "Bosch", "Pump X", "SN-003");
        let contact = np.find_element("ContactInformation");
        assert!(contact.is_some());
        if let Some(SubmodelElement::Collection(c)) = contact {
            assert_eq!(c.elements.len(), 3);
        } else {
            panic!("ContactInformation should be a Collection");
        }
    }

    #[test]
    fn nameplate_marking() {
        let np = NameplateBuilder::new("sm:np:004", "Vendor", "Product", "SN-004")
            .marking("CE", "/marks/ce.png")
            .marking("ATEX", "/marks/atex.png")
            .build();

        let markings = np.find_element("Markings");
        if let Some(SubmodelElement::Collection(c)) = markings {
            assert_eq!(c.elements.len(), 2);
        } else {
            panic!("Markings should be a Collection");
        }
    }
}
