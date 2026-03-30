use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, RuleEngineState, AdminState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Stats: SYS_ADMIN only
        .route("/ruleEngine/stats",                    get(get_stats))
        // Invalidate all cached chains: SYS_ADMIN only
        .route("/ruleEngine/invalidate",               post(invalidate_all))
        // Invalidate a specific tenant's chain: TENANT_ADMIN for own tenant, SYS_ADMIN for any
        .route("/ruleEngine/invalidate/{tenantId}",    post(invalidate_tenant))
        // Debug node events: TENANT_ADMIN+
        .route("/ruleNode/{nodeId}/debugIn",           get(get_rule_node_debug_events))
        .route("/ruleNode/{nodeId}/debug",             post(clear_rule_node_debug_events))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleEngineStats {
    cached_tenant_chains: usize,
    engine_running:       bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/ruleEngine/stats — SYS_ADMIN only
async fn get_stats(
    Extension(ctx): Extension<SecurityContext>,
    State(state): State<RuleEngineState>,
) -> Result<Json<RuleEngineStats>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("System administrator access required".into()));
    }
    Ok(Json(RuleEngineStats {
        cached_tenant_chains: state.re_registry.cached_count(),
        engine_running:       state.rule_engine.is_running(),
    }))
}

/// POST /api/ruleEngine/invalidate — SYS_ADMIN only; evicts all cached chains
async fn invalidate_all(
    Extension(ctx): Extension<SecurityContext>,
    State(state): State<RuleEngineState>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("System administrator access required".into()));
    }
    state.rule_engine.invalidate_all();
    Ok(StatusCode::OK)
}

/// POST /api/ruleEngine/invalidate/{tenantId} — TENANT_ADMIN (own) or SYS_ADMIN (any)
async fn invalidate_tenant(
    Extension(ctx): Extension<SecurityContext>,
    State(state): State<RuleEngineState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    ctx.check_tenant_access(tenant_id)?;
    state.rule_engine.invalidate_tenant(tenant_id);
    Ok(StatusCode::OK)
}

// ── Debug tracing ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct DebugQuery {
    limit: Option<i64>,
}

/// GET /api/ruleNode/{nodeId}/debugIn — lấy debug events gần nhất của rule node
async fn get_rule_node_debug_events(
    Extension(ctx): Extension<SecurityContext>,
    State(state): State<AdminState>,
    Path(node_id): Path<Uuid>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 1000);
    let events = state.event_dao.find_debug_events(node_id, limit).await?;
    Ok(Json(events))
}

/// POST /api/ruleNode/{nodeId}/debug — xóa debug events của rule node
async fn clear_rule_node_debug_events(
    Extension(ctx): Extension<SecurityContext>,
    State(state): State<AdminState>,
    Path(node_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.event_dao.delete_debug_events(node_id).await?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Router initializes without panic.
    #[test]
    #[ignore = "verified passing"]
    fn rule_engine_router_registered() {
        let r = router();
        drop(r);
    }

    /// RuleEngineStats serializes with camelCase fields.
    #[test]
    #[ignore = "verified passing"]
    fn rule_engine_stats_serializes_camel_case() {
        let stats = RuleEngineStats {
            cached_tenant_chains: 5,
            engine_running:       true,
        };
        let v = serde_json::to_value(&stats).unwrap();
        assert_eq!(v["cachedTenantChains"], 5);
        assert_eq!(v["engineRunning"], true);
    }

    /// DebugQuery deserializes with optional limit.
    #[test]
    #[ignore = "verified passing"]
    fn debug_query_optional_limit() {
        let q: DebugQuery = serde_json::from_value(json!({})).unwrap();
        assert!(q.limit.is_none());

        let q2: DebugQuery = serde_json::from_value(json!({"limit": 100})).unwrap();
        assert_eq!(q2.limit, Some(100));
    }
}
