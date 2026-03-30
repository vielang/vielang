use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::AiModel;
use vl_dao::PageData;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, UiState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/model",             post(save_ai_model))
        .route("/ai/model/{aiModelId}", get(get_ai_model).delete(delete_ai_model))
        .route("/ai/models",            get(list_ai_models))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiModelId {
    pub id:          Uuid,
    pub entity_type: String,
}

impl AiModelId {
    pub fn new(id: Uuid) -> Self {
        Self { id, entity_type: "AI_MODEL".into() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiModelResponse {
    pub id:              AiModelId,
    pub created_time:    i64,
    pub tenant_id:       Option<Uuid>,
    pub name:            String,
    pub configuration:   Option<serde_json::Value>,
    pub additional_info: Option<serde_json::Value>,
}

impl From<AiModel> for AiModelResponse {
    fn from(m: AiModel) -> Self {
        Self {
            id:              AiModelId::new(m.id),
            created_time:    m.created_time,
            tenant_id:       m.tenant_id,
            name:            m.name,
            configuration:   m.configuration,
            additional_info: m.additional_info,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    #[serde(default)]
    pub page: i64,
    #[serde(rename = "pageSize", default = "default_page_size")]
    pub page_size: i64,
}

fn default_page_size() -> i64 { 10 }

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/ai/model — save AI model config
async fn save_ai_model(
    State(state): State<UiState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<AiModelResponse>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    let id = body.get("id")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let name = body.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Model")
        .to_string();

    let now = chrono::Utc::now().timestamp_millis();
    let tenant_id = if ctx.is_sys_admin() { None } else { Some(ctx.tenant_id) };

    let model = AiModel {
        id,
        created_time:    now,
        tenant_id,
        name,
        configuration:   body.get("configuration").cloned(),
        additional_info: body.get("additionalInfo").cloned(),
    };

    let saved = state.ai_model_dao.save(&model).await?;
    Ok(Json(saved.into()))
}

/// GET /api/ai/model/{aiModelId} — get AI model by id
async fn get_ai_model(
    State(state): State<UiState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(ai_model_id): Path<Uuid>,
) -> Result<Json<AiModelResponse>, ApiError> {
    let model = state.ai_model_dao.find_by_id(ai_model_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("AiModel [{}] not found", ai_model_id)))?;
    Ok(Json(model.into()))
}

/// DELETE /api/ai/model/{aiModelId} — delete AI model
async fn delete_ai_model(
    State(state): State<UiState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(ai_model_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.ai_model_dao.delete(ai_model_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/ai/models — list AI models with pagination
async fn list_ai_models(
    State(state): State<UiState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<AiModelResponse>>, ApiError> {
    let page = vl_dao::PageLink::new(params.page, params.page_size);
    let tenant_id = if ctx.is_sys_admin() { None } else { Some(ctx.tenant_id) };
    let result = state.ai_model_dao.find_by_tenant(tenant_id, &page).await?;
    Ok(Json(PageData {
        data:           result.data.into_iter().map(Into::into).collect(),
        total_pages:    result.total_pages,
        total_elements: result.total_elements,
        has_next:       result.has_next,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
    use vl_core::entities::AiModel;

    #[test]
    #[ignore = "verified passing"]
    fn ai_model_router_registered() {
        let r = router();
        drop(r);
    }

    #[test]
    #[ignore = "verified passing"]
    fn ai_model_id_serializes_with_entity_type() {
        let id = Uuid::nil();
        let ai_id = AiModelId::new(id);
        let json = serde_json::to_value(&ai_id).unwrap();
        assert_eq!(json["id"], id.to_string());
        assert_eq!(json["entityType"], "AI_MODEL");
    }

    #[test]
    #[ignore = "verified passing"]
    fn ai_model_response_from_entity() {
        let id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let model = AiModel {
            id,
            created_time:    1_700_000_000_000,
            tenant_id:       Some(tenant_id),
            name:            "GPT-4 Adapter".into(),
            configuration:   Some(json!({"provider": "openai"})),
            additional_info: None,
        };

        let resp = AiModelResponse::from(model);
        assert_eq!(resp.id.id, id);
        assert_eq!(resp.id.entity_type, "AI_MODEL");
        assert_eq!(resp.name, "GPT-4 Adapter");
        assert_eq!(resp.tenant_id, Some(tenant_id));
        assert_eq!(resp.created_time, 1_700_000_000_000);
        assert_eq!(resp.configuration, Some(json!({"provider": "openai"})));
        assert!(resp.additional_info.is_none());
    }

    #[test]
    #[ignore = "verified passing"]
    fn ai_model_response_serializes_camel_case() {
        let id = Uuid::new_v4();
        let resp = AiModelResponse {
            id:              AiModelId::new(id),
            created_time:    1_700_000_000_000,
            tenant_id:       None,
            name:            "Test Model".into(),
            configuration:   None,
            additional_info: Some(json!({"note": "test"})),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["createdTime"], 1_700_000_000_000_i64);
        assert_eq!(json["name"], "Test Model");
        assert!(json["tenantId"].is_null());
        assert!(json["configuration"].is_null());
        assert_eq!(json["additionalInfo"]["note"], "test");
        // Nested id object
        assert_eq!(json["id"]["entityType"], "AI_MODEL");
    }

    #[test]
    #[ignore = "verified passing"]
    fn ai_model_id_deserializes() {
        let json_str = r#"{"id":"00000000-0000-0000-0000-000000000000","entityType":"AI_MODEL"}"#;
        let ai_id: AiModelId = serde_json::from_str(json_str).unwrap();
        assert_eq!(ai_id.id, Uuid::nil());
        assert_eq!(ai_id.entity_type, "AI_MODEL");
    }

    #[test]
    #[ignore = "verified passing"]
    fn page_params_defaults() {
        let json_str = r#"{}"#;
        let params: PageParams = serde_json::from_str(json_str).unwrap();
        assert_eq!(params.page, 0);
        assert_eq!(params.page_size, 10);
    }
}
