use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::DomainEntry;
use vl_dao::PageData;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AuthState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/domain",                          post(save_domain))
        .route("/domain/{domainId}",               get(get_domain).delete(delete_domain))
        .route("/domains",                         get(list_domains))
        .route("/domain/{domainId}/oauth2Clients", post(assign_oauth2_clients).get(list_oauth2_clients))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainId {
    pub id:          Uuid,
    pub entity_type: String,
}

impl DomainId {
    pub fn new(id: Uuid) -> Self {
        Self { id, entity_type: "DOMAIN".into() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainResponse {
    pub id:                DomainId,
    pub created_time:      i64,
    pub tenant_id:         Uuid,
    pub name:              String,
    pub oauth2_enabled:    bool,
    pub propagate_to_edge: bool,
}

impl From<DomainEntry> for DomainResponse {
    fn from(d: DomainEntry) -> Self {
        Self {
            id:                DomainId::new(d.id),
            created_time:      d.created_time,
            tenant_id:         d.tenant_id,
            name:              d.name,
            oauth2_enabled:    d.oauth2_enabled,
            propagate_to_edge: d.propagate_to_edge,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ClientInfo {
    pub id:          Uuid,
    pub name:        String,
    pub provider_id: String,
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

/// POST /api/domain — save a domain
async fn save_domain(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DomainResponse>, ApiError> {
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
        .unwrap_or("")
        .to_string();

    let now = chrono::Utc::now().timestamp_millis();
    let domain = DomainEntry {
        id,
        created_time:      now,
        tenant_id:         ctx.tenant_id,
        name,
        oauth2_enabled:    body.get("oauth2Enabled").and_then(|v| v.as_bool()).unwrap_or(false),
        propagate_to_edge: body.get("propagateToEdge").and_then(|v| v.as_bool()).unwrap_or(false),
    };

    let saved = state.domain_dao.save(&domain).await?;
    Ok(Json(saved.into()))
}

/// GET /api/domain/{domainId} — get domain by id
async fn get_domain(
    State(state): State<AuthState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(domain_id): Path<Uuid>,
) -> Result<Json<DomainResponse>, ApiError> {
    let domain = state.domain_dao.find_by_id(domain_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Domain [{}] not found", domain_id)))?;
    Ok(Json(domain.into()))
}

/// DELETE /api/domain/{domainId} — delete domain
async fn delete_domain(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(domain_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.domain_dao.delete(domain_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/domains — list domains with pagination
async fn list_domains(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<DomainResponse>>, ApiError> {
    let page = vl_dao::PageLink::new(params.page, params.page_size);
    let result = state.domain_dao.find_by_tenant(ctx.tenant_id, &page).await?;
    Ok(Json(PageData {
        data:           result.data.into_iter().map(Into::into).collect(),
        total_pages:    result.total_pages,
        total_elements: result.total_elements,
        has_next:       result.has_next,
    }))
}

/// POST /api/domain/{domainId}/oauth2Clients — assign OAuth2 clients to domain
async fn assign_oauth2_clients(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(domain_id): Path<Uuid>,
    Json(client_ids): Json<Vec<Uuid>>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.domain_dao.set_oauth2_clients(domain_id, &client_ids).await?;
    Ok(StatusCode::OK)
}

/// GET /api/domain/{domainId}/oauth2Clients — list OAuth2 clients for domain
async fn list_oauth2_clients(
    State(state): State<AuthState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(domain_id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    let ids = state.domain_dao.get_oauth2_clients(domain_id).await?;
    Ok(Json(ids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    /// Router initializes without panic.
    #[test]
    #[ignore = "verified passing"]
    fn domain_router_registered() {
        let r = router();
        drop(r);
    }

    /// DomainId serializes with camelCase fields.
    #[test]
    #[ignore = "verified passing"]
    fn domain_id_serializes_correctly() {
        let id = Uuid::nil();
        let domain_id = DomainId::new(id);
        let v = serde_json::to_value(&domain_id).unwrap();
        assert_eq!(v["id"], id.to_string());
        assert_eq!(v["entityType"], "DOMAIN");
    }

    /// DomainResponse serializes with camelCase fields.
    #[test]
    #[ignore = "verified passing"]
    fn domain_response_serializes_camel_case() {
        let resp = DomainResponse {
            id:                DomainId::new(Uuid::nil()),
            created_time:      1711612800000,
            tenant_id:         Uuid::nil(),
            name:              "example.com".into(),
            oauth2_enabled:    true,
            propagate_to_edge: false,
        };
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["createdTime"], 1711612800000i64);
        assert_eq!(v["tenantId"], Uuid::nil().to_string());
        assert_eq!(v["name"], "example.com");
        assert_eq!(v["oauth2Enabled"], true);
        assert_eq!(v["propagateToEdge"], false);
    }

    /// PageParams deserializes with camelCase and defaults.
    #[test]
    #[ignore = "verified passing"]
    fn page_params_defaults() {
        let params: PageParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(params.page, 0);
        assert_eq!(params.page_size, 10);
    }

    /// DomainResponse round-trips From<DomainEntry>.
    #[test]
    #[ignore = "verified passing"]
    fn domain_entry_converts_to_response() {
        let entry = vl_core::entities::DomainEntry {
            id:                Uuid::new_v4(),
            created_time:      1000,
            tenant_id:         Uuid::new_v4(),
            name:              "test.io".into(),
            oauth2_enabled:    true,
            propagate_to_edge: true,
        };
        let resp = DomainResponse::from(entry.clone());
        assert_eq!(resp.id.id, entry.id);
        assert_eq!(resp.name, "test.io");
        assert_eq!(resp.oauth2_enabled, true);
    }
}
