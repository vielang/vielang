use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// ── Entity Key ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct EntityKey {
    #[serde(rename = "type")]
    pub key_type: EntityKeyType,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityKeyType {
    Attribute,
    ClientAttribute,
    SharedAttribute,
    ServerAttribute,
    TimeSeries,
    EntityField,
    AlarmField,
}

// ── Key Filters ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyFilter {
    pub key: EntityKey,
    pub value_type: EntityKeyValueType,
    pub predicate: KeyFilterPredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityKeyValueType {
    String,
    Numeric,
    Boolean,
    DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyFilterPredicate {
    String(StringFilterPredicate),
    Numeric(NumericFilterPredicate),
    Boolean(BooleanFilterPredicate),
    Complex(ComplexFilterPredicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringFilterPredicate {
    pub operation: StringOperation,
    pub value: FilterPredicateValue<String>,
    #[serde(default)]
    pub ignore_case: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StringOperation {
    Equal,
    NotEqual,
    StartsWith,
    EndsWith,
    Contains,
    NotContains,
    In,
    NotIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericFilterPredicate {
    pub operation: NumericOperation,
    pub value: FilterPredicateValue<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NumericOperation {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanFilterPredicate {
    pub operation: BooleanOperation,
    pub value: FilterPredicateValue<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BooleanOperation {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexFilterPredicate {
    pub operation: ComplexOperation,
    pub predicates: Vec<KeyFilterPredicate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplexOperation {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterPredicateValue<T> {
    pub default_value: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_value: Option<T>,
}

impl<T: Clone> FilterPredicateValue<T> {
    /// Trả về user_value nếu có, còn không thì default_value
    pub fn effective(&self) -> &T {
        self.user_value.as_ref().unwrap_or(&self.default_value)
    }
}

// ── Entity ID ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryEntityId {
    pub id: Uuid,
    pub entity_type: String,
}

// ── Relation Filter ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_type_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub entity_types: Vec<String>,
}

// ── Entity Filters ────────────────────────────────────────────────────────────

/// Polymorphic entity filter — discriminator field là "type"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EntityFilter {
    #[serde(rename = "singleEntity", rename_all = "camelCase")]
    SingleEntity {
        single_entity: QueryEntityId,
    },
    #[serde(rename = "entityList", rename_all = "camelCase")]
    EntityList {
        entity_type: String,
        entity_ids: Vec<Uuid>,
    },
    #[serde(rename = "entityName", rename_all = "camelCase")]
    EntityName {
        entity_type: String,
        entity_name_filter: String,
    },
    #[serde(rename = "entityType", rename_all = "camelCase")]
    EntityType {
        entity_type: String,
    },
    #[serde(rename = "assetType", rename_all = "camelCase")]
    AssetType {
        asset_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        asset_name_filter: Option<String>,
    },
    #[serde(rename = "deviceType", rename_all = "camelCase")]
    DeviceType {
        device_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        device_name_filter: Option<String>,
    },
    #[serde(rename = "entityViewType", rename_all = "camelCase")]
    EntityViewType {
        entity_view_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        entity_view_name_filter: Option<String>,
    },
    #[serde(rename = "edgeType", rename_all = "camelCase")]
    EdgeType {
        edge_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        edge_name_filter: Option<String>,
    },
    #[serde(rename = "relationsQuery", rename_all = "camelCase")]
    RelationsQuery {
        root_entity: QueryEntityId,
        direction: String,
        #[serde(default)]
        filters: Vec<RelationFilter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_level: Option<i32>,
        #[serde(default)]
        fetch_last_level_only: bool,
    },
    #[serde(rename = "assetSearchQuery", rename_all = "camelCase")]
    AssetSearchQuery {
        root_entity: QueryEntityId,
        direction: String,
        #[serde(default)]
        filters: Vec<RelationFilter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_level: Option<i32>,
        #[serde(default)]
        fetch_last_level_only: bool,
        asset_types: Vec<String>,
    },
    #[serde(rename = "deviceSearchQuery", rename_all = "camelCase")]
    DeviceSearchQuery {
        root_entity: QueryEntityId,
        direction: String,
        #[serde(default)]
        filters: Vec<RelationFilter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_level: Option<i32>,
        #[serde(default)]
        fetch_last_level_only: bool,
        device_types: Vec<String>,
    },
    /// Search entity views reachable from root_entity via relations.
    #[serde(rename = "entityViewSearchQuery", rename_all = "camelCase")]
    EntityViewSearchQuery {
        root_entity: QueryEntityId,
        direction: String,
        #[serde(default)]
        filters: Vec<RelationFilter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_level: Option<i32>,
        #[serde(default)]
        fetch_last_level_only: bool,
        entity_view_types: Vec<String>,
    },
    /// Filter by API usage state entity type (tenant-level usage tracking).
    #[serde(rename = "apiUsageState", rename_all = "camelCase")]
    ApiUsageState {
        entity_type: String,
    },
    /// Filter to sub-customers of a given root customer.
    #[serde(rename = "subCustomers", rename_all = "camelCase")]
    SubCustomers {
        root_customer_id: Uuid,
    },
}

// ── Page Links ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityDataPageLink {
    pub page_size: i64,
    pub page: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<EntityDataSortOrder>,
    #[serde(default)]
    pub dynamic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityDataSortOrder {
    pub key: EntityKey,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmDataPageLink {
    pub page_size: i64,
    pub page: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<EntityDataSortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_ts: Option<i64>,
    #[serde(default)]
    pub alarm_status_list: Vec<String>,
    #[serde(default)]
    pub alarm_severity_list: Vec<String>,
    #[serde(default)]
    pub alarm_type_list: Vec<String>,
    #[serde(default)]
    pub search_propagated_alarms: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_user_id: Option<Uuid>,
}

// ── Queries ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCountQuery {
    pub entity_filter: EntityFilter,
    #[serde(default)]
    pub key_filters: Vec<KeyFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityDataQuery {
    pub entity_filter: EntityFilter,
    #[serde(default)]
    pub key_filters: Vec<KeyFilter>,
    pub page_link: EntityDataPageLink,
    #[serde(default)]
    pub entity_fields: Vec<EntityKey>,
    #[serde(default)]
    pub latest_values: Vec<EntityKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmCountQuery {
    pub entity_filter: EntityFilter,
    #[serde(default)]
    pub key_filters: Vec<KeyFilter>,
    #[serde(default)]
    pub alarm_status_list: Vec<String>,
    #[serde(default)]
    pub alarm_severity_list: Vec<String>,
    #[serde(default)]
    pub alarm_type_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmDataQuery {
    pub entity_filter: EntityFilter,
    #[serde(default)]
    pub key_filters: Vec<KeyFilter>,
    pub page_link: AlarmDataPageLink,
    #[serde(default)]
    pub alarm_fields: Vec<EntityKey>,
    #[serde(default)]
    pub entity_fields: Vec<EntityKey>,
    #[serde(default)]
    pub latest_values: Vec<EntityKey>,
}

// ── Response Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsValue {
    pub ts: i64,
    pub value: String,
}

/// Entity data trả về từ entitiesQuery/find
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityData {
    pub entity_id: QueryEntityId,
    /// Map: EntityKeyType string → { key → TsValue }
    pub latest: HashMap<String, HashMap<String, TsValue>>,
    pub timeseries: HashMap<String, Vec<TsValue>>,
}

/// Alarm data trả về từ alarmsQuery/find
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmData {
    pub entity_id: QueryEntityId,
    pub alarm_id: QueryEntityId,
    pub created_time: i64,
    pub ack_ts: i64,
    pub clear_ts: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originator_label: Option<String>,
    pub severity: String,
    pub status: String,
    #[serde(rename = "type")]
    pub alarm_type: String,
    pub acknowledged: bool,
    pub cleared: bool,
    pub latest: HashMap<String, HashMap<String, TsValue>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_count_query_deserialize() {
        let json = r#"{"entityFilter": { "type": "entityType", "entityType": "DEVICE" }}"#;
        let result: Result<EntityCountQuery, _> = serde_json::from_str(json);
        match &result {
            Ok(q) => println!("OK: {:?}", q),
            Err(e) => println!("ERR: {}", e),
        }
        result.unwrap();
    }

    #[test]
    fn test_entity_filter_device_type_deserialize() {
        let json = r#"{"type": "deviceType", "deviceType": "sensor"}"#;
        let result: Result<EntityFilter, _> = serde_json::from_str(json);
        match &result {
            Ok(q) => println!("OK: {:?}", q),
            Err(e) => println!("ERR: {}", e),
        }
        result.unwrap();
    }

    #[test]
    fn test_entity_view_search_query_deserialize() {
        let json = r#"{
            "type": "entityViewSearchQuery",
            "rootEntity": {"id": "00000000-0000-0000-0000-000000000001", "entityType": "DEVICE"},
            "direction": "FROM",
            "maxLevel": 2,
            "fetchLastLevelOnly": false,
            "entityViewTypes": ["MyView"]
        }"#;
        let filter: EntityFilter = serde_json::from_str(json).unwrap();
        assert!(matches!(filter, EntityFilter::EntityViewSearchQuery { .. }));
    }

    #[test]
    fn test_api_usage_state_filter_deserialize() {
        let json = r#"{"type": "apiUsageState", "entityType": "TENANT"}"#;
        let filter: EntityFilter = serde_json::from_str(json).unwrap();
        assert!(matches!(filter, EntityFilter::ApiUsageState { .. }));
    }

    #[test]
    fn test_sub_customers_filter_deserialize() {
        let json = r#"{
            "type": "subCustomers",
            "rootCustomerId": "00000000-0000-0000-0000-000000000002"
        }"#;
        let filter: EntityFilter = serde_json::from_str(json).unwrap();
        assert!(matches!(filter, EntityFilter::SubCustomers { .. }));
    }

    #[test]
    fn test_device_search_query_deserialize() {
        let json = r#"{
            "type": "deviceSearchQuery",
            "rootEntity": {"id": "00000000-0000-0000-0000-000000000003", "entityType": "ASSET"},
            "direction": "FROM",
            "filters": [{"relationType": "Contains"}],
            "maxLevel": 3,
            "fetchLastLevelOnly": true,
            "deviceTypes": ["sensor", "actuator"]
        }"#;
        let filter: EntityFilter = serde_json::from_str(json).unwrap();
        if let EntityFilter::DeviceSearchQuery { device_types, fetch_last_level_only, max_level, .. } = filter {
            assert_eq!(device_types.len(), 2);
            assert!(fetch_last_level_only);
            assert_eq!(max_level, Some(3));
        } else {
            panic!("expected DeviceSearchQuery");
        }
    }
}
