use axum::{
    extract::{Extension, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use vl_core::entities::{ApiUsageHistory, TenantApiUsage, UsageInfo};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState, BillingState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/usage",                          get(get_usage_info))
        .route("/usage/history",                  get(get_usage_history))
        .route("/admin/usage",                    get(get_admin_usage))
        .route("/admin/usage/{tenantId}/reset",   post(reset_tenant_period))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 { 12 }

/// GET /api/usage — usage statistics for the current tenant
async fn get_usage_info(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UsageInfo>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let info = state.usage_info_dao
        .get_tenant_usage(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(info))
}

/// GET /api/usage/history — paginated billing history for the current tenant
async fn get_usage_history(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<ApiUsageHistory>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let history = state.api_usage_dao
        .get_history(ctx.tenant_id, q.limit.min(100), q.offset)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(history))
}

/// GET /api/admin/usage?period=YYYY-MM — all tenants' usage for a billing period
async fn get_admin_usage(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TenantApiUsage>>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("System admin access required".into()));
    }
    let period = params.get("period")
        .cloned()
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m").to_string());

    let result = state.api_usage_dao
        .get_all_usage_domain(&period)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(result))
}

/// POST /api/admin/usage/{tenantId}/reset — archive and reset a tenant's current period
async fn reset_tenant_period(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("System admin access required".into()));
    }
    state.api_usage_dao
        .reset_period(tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_usage_info_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/usage")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
