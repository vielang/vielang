use serde::{Deserialize, Serialize};

/// Request body for bulk import endpoints.
/// POST /api/device/bulk_import, POST /api/asset/bulk_import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkImportRequest {
    /// CSV content as string
    pub file: String,
    /// Column mapping configuration
    pub mapping: ColumnMapping,
}

/// Configuration for how CSV columns map to entity fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMapping {
    /// Array of column definitions in order
    pub columns: Vec<ColumnDef>,
    /// CSV delimiter character (default: ',')
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
    /// If true, update existing entities by name; if false, skip
    #[serde(default)]
    pub update: bool,
    /// If true, first row is header and should be skipped
    #[serde(default = "default_true")]
    pub header: bool,
}

fn default_delimiter() -> char {
    ','
}

fn default_true() -> bool {
    true
}

/// Definition of a single CSV column.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDef {
    /// Type of data in this column
    #[serde(rename = "type")]
    pub column_type: BulkImportColumnType,
    /// Key name for key-value types (SERVER_ATTRIBUTE, SHARED_ATTRIBUTE, TIMESERIES)
    #[serde(default)]
    pub key: Option<String>,
}

/// Types of columns that can be imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BulkImportColumnType {
    // Core entity fields
    Name,
    Type,
    Label,
    Description,
    IsGateway,

    // Device credentials
    AccessToken,
    MqttClientId,
    MqttUserName,
    MqttPassword,
    X509,

    // Key-value fields (require key parameter)
    ServerAttribute,
    SharedAttribute,
    ClientAttribute,
    Timeseries,

    // Edge-specific
    RoutingKey,
    Secret,

    // LwM2M (advanced)
    Lwm2mClientEndpoint,
    Lwm2mClientSecurityConfigMode,
    Lwm2mClientIdentity,
    Lwm2mClientKey,
    Lwm2mClientCert,

    // SNMP
    SnmpHost,
    SnmpPort,
    SnmpVersion,
    SnmpCommunityString,
}

impl BulkImportColumnType {
    /// Returns true if this column type requires a key parameter
    pub fn is_key_value(&self) -> bool {
        matches!(
            self,
            BulkImportColumnType::ServerAttribute
                | BulkImportColumnType::SharedAttribute
                | BulkImportColumnType::ClientAttribute
                | BulkImportColumnType::Timeseries
        )
    }
}

/// Result of a bulk import operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkImportResult {
    /// Number of entities created
    pub created: i64,
    /// Number of entities updated
    pub updated: i64,
    /// Number of errors encountered
    pub errors: i64,
    /// List of error messages with line numbers
    #[serde(default)]
    pub errors_list: Vec<String>,
}

impl BulkImportResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_created(&mut self) {
        self.created += 1;
    }

    pub fn add_updated(&mut self) {
        self.updated += 1;
    }

    pub fn add_error(&mut self, line: usize, message: &str) {
        self.errors += 1;
        self.errors_list.push(format!("Line {}: {}", line, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_bulk_import_request() {
        let json = r#"{
            "file": "NAME,TYPE\nDevice1,sensor",
            "mapping": {
                "columns": [
                    {"type": "NAME"},
                    {"type": "TYPE"}
                ],
                "delimiter": ",",
                "update": true,
                "header": true
            }
        }"#;

        let req: BulkImportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mapping.columns.len(), 2);
        assert_eq!(req.mapping.columns[0].column_type, BulkImportColumnType::Name);
        assert_eq!(req.mapping.columns[1].column_type, BulkImportColumnType::Type);
        assert!(req.mapping.update);
        assert!(req.mapping.header);
    }

    #[test]
    fn test_column_type_is_key_value() {
        assert!(BulkImportColumnType::ServerAttribute.is_key_value());
        assert!(BulkImportColumnType::SharedAttribute.is_key_value());
        assert!(BulkImportColumnType::Timeseries.is_key_value());
        assert!(!BulkImportColumnType::Name.is_key_value());
        assert!(!BulkImportColumnType::AccessToken.is_key_value());
    }

    #[test]
    fn test_bulk_import_result() {
        let mut result = BulkImportResult::new();
        result.add_created();
        result.add_created();
        result.add_updated();
        result.add_error(5, "Invalid name");

        assert_eq!(result.created, 2);
        assert_eq!(result.updated, 1);
        assert_eq!(result.errors, 1);
        assert_eq!(result.errors_list[0], "Line 5: Invalid name");
    }
}
