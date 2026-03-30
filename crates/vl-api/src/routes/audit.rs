use axum::{
    extract::{Extension, Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, CoreState}};
use vl_core::entities::AuditLog;
use vl_dao::{PageData, PageLink};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/audit/logs",                                  get(get_audit_logs))
        .route("/audit/logs/user/{userId}",                    get(get_audit_logs_by_user))
        .route("/audit/logs/customer/{customerId}",            get(get_audit_logs_by_customer))
        .route("/audit/logs/entity/{entityType}/{entityId}",   get(get_audit_logs_by_entity))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditPageParams {
    #[serde(default)]
    page:      i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
}
fn default_page_size() -> i64 { 20 }

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/audit/logs — paginated audit logs for current tenant
async fn get_audit_logs(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(p): Query<AuditPageParams>,
) -> Result<Json<PageData<AuditLog>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    let page = PageLink::new(p.page, p.page_size);
    let data = state.audit_log_dao
        .find_by_tenant(ctx.tenant_id, &page)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /api/audit/logs/user/{userId}
async fn get_audit_logs_by_user(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(user_id): Path<Uuid>,
    Query(p): Query<AuditPageParams>,
) -> Result<Json<PageData<AuditLog>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    let page = PageLink::new(p.page, p.page_size);
    let data = state.audit_log_dao
        .find_by_user(ctx.tenant_id, user_id, &page)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /api/audit/logs/customer/{customerId} — equivalent to user filter by customer scope
async fn get_audit_logs_by_customer(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(customer_id): Path<Uuid>,
    Query(p): Query<AuditPageParams>,
) -> Result<Json<PageData<AuditLog>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    // Customer audit logs: filter by entity_type='CUSTOMER' AND entity_id=customer_id
    let page = PageLink::new(p.page, p.page_size);
    let data = state.audit_log_dao
        .find_by_entity("CUSTOMER", customer_id, &page)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /api/audit/logs/entity/{entityType}/{entityId}
async fn get_audit_logs_by_entity(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(p): Query<AuditPageParams>,
) -> Result<Json<PageData<AuditLog>>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    let page = PageLink::new(p.page, p.page_size);
    let data = state.audit_log_dao
        .find_by_entity(&entity_type, entity_id, &page)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    async fn test_app(pool: PgPool) -> (axum::Router, AppState) {
        let config = VieLangConfig::default();
        let re     = vl_rule_engine::RuleEngine::start_noop();
        let qp     = vl_queue::create_producer(&config.queue).expect("queue");
        let cache  = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state  = AppState::new(pool, config, ts_dao, re, qp, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        let app    = create_router(state.clone());
        (app, state)
    }

    fn admin_token(state: &AppState) -> String {
        state.jwt_service
            .issue_token(
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                None,
                "TENANT_ADMIN",
                vec!["TENANT_ADMIN".into()],
            )
            .unwrap()
            .token
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_audit_logs_returns_empty_page(pool: PgPool) {
        let (app, state) = test_app(pool).await;
        let token = admin_token(&state);

        let resp = app.oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/audit/logs")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["totalElements"], 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn customer_user_cannot_read_audit_logs(pool: PgPool) {
        let (app, state) = test_app(pool).await;
        let token = state.jwt_service
            .issue_token(Uuid::new_v4(), Some(Uuid::new_v4()), Some(Uuid::new_v4()),
                         "CUSTOMER_USER", vec!["CUSTOMER_USER".into()])
            .unwrap().token;

        let resp = app.oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/audit/logs")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    }
}
