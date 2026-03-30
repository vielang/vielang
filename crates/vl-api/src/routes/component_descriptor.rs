use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use vl_core::entities::ComponentDescriptor;

use crate::{error::ApiError, state::{AppState, RuleEngineState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // GET /api/component/{componentDescriptorClazz:.+} — by class name
        // Note: Java uses /component/{clazz} path, not /component/{type}
        .route("/component/{clazz}", get(get_by_clazz))
        // GET /api/components/{componentType} — list by type
        .route("/components/{componentType}", get(get_by_type))
        // GET /api/components?componentTypes=... — list by multiple types
        .route("/components", get(get_by_types))
}

/// GET /api/component/{componentDescriptorClazz}
/// Returns a single ComponentDescriptor matching the given class name.
async fn get_by_clazz(
    State(state): State<RuleEngineState>,
    Path(clazz): Path<String>,
) -> Result<Json<ComponentDescriptor>, ApiError> {
    let descriptor = state
        .component_descriptor_dao
        .find_by_clazz(&clazz)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("ComponentDescriptor [{}] not found", clazz)))?;
    Ok(Json(descriptor))
}

/// GET /api/components/{componentType}
/// Returns all ComponentDescriptors matching the given type.
/// Optional ?ruleChainType=CORE|EDGE query param (currently not filtered in DB, returned as-is).
async fn get_by_type(
    State(state): State<RuleEngineState>,
    Path(component_type): Path<String>,
    Query(_params): Query<RuleChainTypeParam>,
) -> Result<Json<Vec<ComponentDescriptor>>, ApiError> {
    if component_type.trim().is_empty() {
        return Err(ApiError::BadRequest("componentType is required".into()));
    }
    let descriptors = state
        .component_descriptor_dao
        .find_by_type(&component_type)
        .await?;
    Ok(Json(descriptors))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComponentTypesParam {
    /// Comma-separated list of component types, or repeated query param
    component_types: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuleChainTypeParam {
    rule_chain_type: Option<String>,
}

/// GET /api/components?componentTypes=FILTER,ACTION
/// Returns ComponentDescriptors for all listed types.
async fn get_by_types(
    State(state): State<RuleEngineState>,
    Query(params): Query<ComponentTypesParam>,
) -> Result<Json<Vec<ComponentDescriptor>>, ApiError> {
    let types_str = params
        .component_types
        .unwrap_or_default();

    if types_str.trim().is_empty() {
        // No filter — return all
        let descriptors = state.component_descriptor_dao.find_all().await?;
        return Ok(Json(descriptors));
    }

    let types: Vec<String> = types_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if types.is_empty() {
        return Ok(Json(vec![]));
    }

    let descriptors = state
        .component_descriptor_dao
        .find_by_types(&types)
        .await?;
    Ok(Json(descriptors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_creates_without_panic() {
        let _ = router();
    }

    #[test]
    fn component_descriptor_serializes_type_field_correctly() {
        let desc = ComponentDescriptor {
            id: uuid::Uuid::nil(),
            created_time: 0,
            type_: Some("FILTER".into()),
            scope: Some("SYSTEM".into()),
            clustering_mode: Some("USER_PREFERENCE".into()),
            name: Some("Message Type Filter".into()),
            clazz: Some("org.thingsboard.rule.engine.filter.TbMsgTypeFilterNode".into()),
            configuration_descriptor: Some(serde_json::json!({})),
            configuration_version: Some(0),
            actions: None,
            has_queue_name: Some(false),
        };
        let json = serde_json::to_value(&desc).unwrap();
        // type_ field should serialize as "type" (not "type_")
        assert_eq!(json["type"], "FILTER");
        assert_eq!(json["scope"], "SYSTEM");
        assert_eq!(json["clusteringMode"], "USER_PREFERENCE");
        assert_eq!(json["name"], "Message Type Filter");
        assert_eq!(json["hasQueueName"], false);
        // Ensure snake_case fields are NOT present
        assert!(json.get("created_time").is_none());
        assert!(json.get("clustering_mode").is_none());
        assert!(json.get("has_queue_name").is_none());
    }

    #[test]
    fn component_types_param_deserializes_from_camel_case() {
        let json = serde_json::json!({
            "componentTypes": "FILTER,ACTION"
        });
        let params: ComponentTypesParam = serde_json::from_value(json).unwrap();
        assert_eq!(params.component_types.as_deref(), Some("FILTER,ACTION"));
    }

    #[test]
    fn rule_chain_type_param_deserializes_from_camel_case() {
        let json = serde_json::json!({
            "ruleChainType": "CORE"
        });
        let params: RuleChainTypeParam = serde_json::from_value(json).unwrap();
        assert_eq!(params.rule_chain_type.as_deref(), Some("CORE"));
    }

    #[test]
    fn component_descriptor_with_nulls_serializes() {
        let desc = ComponentDescriptor {
            id: uuid::Uuid::nil(),
            created_time: 0,
            type_: None,
            scope: None,
            clustering_mode: None,
            name: None,
            clazz: None,
            configuration_descriptor: None,
            configuration_version: None,
            actions: None,
            has_queue_name: None,
        };
        let json = serde_json::to_value(&desc).unwrap();
        assert!(json["type"].is_null());
        assert!(json["name"].is_null());
        assert!(json.get("createdTime").is_some());
    }
}
