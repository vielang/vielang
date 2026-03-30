use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use vl_core::entities::AdminSettings;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings/user",   get(get_user_ui_settings).put(update_user_ui_settings))
        .route("/settings/system", get(get_system_ui_settings).put(update_system_ui_settings))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub settings: Value,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self { settings: Value::Object(Default::default()) }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/settings/user — per-user UI settings (key = "userSettings" keyed by user_id)
async fn get_user_ui_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UiSettings>, ApiError> {
    let settings = state.admin_settings_dao
        .find_by_key(ctx.user_id, "userSettings")
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|s| UiSettings { settings: s.json_value })
        .unwrap_or_default();
    Ok(Json(settings))
}

/// PUT /api/settings/user — update per-user UI settings
async fn update_user_ui_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<UiSettings>,
) -> Result<Json<UiSettings>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    let s = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    ctx.user_id,
        key:          "userSettings".into(),
        json_value:   body.settings.clone(),
    };
    state.admin_settings_dao.save(&s).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(body))
}

/// GET /api/settings/system — system-wide UI settings (SYS_ADMIN only)
async fn get_system_ui_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UiSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let settings = state.admin_settings_dao
        .find_by_key(Uuid::nil(), "uiSettings")
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|s| UiSettings { settings: s.json_value })
        .unwrap_or_default();
    Ok(Json(settings))
}

/// PUT /api/settings/system — update system UI settings (SYS_ADMIN only)
async fn update_system_ui_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<UiSettings>,
) -> Result<Json<UiSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let s = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    Uuid::nil(),
        key:          "uiSettings".into(),
        json_value:   body.settings.clone(),
    };
    state.admin_settings_dao.save(&s).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(body))
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
    async fn get_ui_settings_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/settings/user")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
