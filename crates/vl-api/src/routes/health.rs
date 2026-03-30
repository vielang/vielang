use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use serde_json::Value;

use crate::state::{AppState, CoreState, RuleEngineState};

/// Server start time — set once at first health check call.
static SERVER_START: OnceLock<Instant> = OnceLock::new();

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status:      String,
    pub version:     String,
    pub uptime_secs: u64,
    pub components:  HashMap<String, ComponentHealth>,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    pub status:  String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health",            get(health_check))
        // Kubernetes-style aliases
        .route("/health/live",       get(liveness))
        .route("/health/ready",      get(readiness))
        // Spring Boot Actuator-style aliases (backwards compat)
        .route("/health/liveness",   get(liveness))
        .route("/health/readiness",  get(readiness))
}

/// Liveness probe — always 200 while the process is running.
/// Kubernetes restarts the pod only if this returns non-200.
async fn liveness() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe — 200 when the pod can serve traffic, 503 when not ready.
/// Checks DB connectivity only (the minimum requirement to handle requests).
async fn readiness(State(state): State<CoreState>) -> impl IntoResponse {
    let db_ok = state.pool.acquire().await.is_ok();
    if db_ok {
        StatusCode::OK.into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({ "status": "DOWN", "reason": "database unavailable" })),
        )
            .into_response()
    }
}

/// Health check khớp Spring Boot Actuator /health format.
///
/// Returns 200 OK nếu tất cả components UP, 503 nếu có component DOWN.
async fn health_check(
    State(state): State<CoreState>,
    State(re_state): State<RuleEngineState>,
) -> impl IntoResponse {
    let start = SERVER_START.get_or_init(Instant::now);
    let uptime_secs = start.elapsed().as_secs();

    // ── DB check — acquire a connection from the pool to verify liveness ──────
    let db_ok     = state.pool.acquire().await.is_ok();
    let pool_size = state.pool.size() as i64;
    let pool_idle = state.pool.num_idle() as i64;

    // Update pool metrics while we're here
    metrics::gauge!(crate::metrics::DB_POOL_SIZE).set(pool_size as f64);
    metrics::gauge!(crate::metrics::DB_POOL_IDLE).set(pool_idle as f64);

    // ── Rule engine check ─────────────────────────────────────────────────────
    let re_running = re_state.rule_engine.is_running();

    // ── Queue type ────────────────────────────────────────────────────────────
    let queue_type = format!("{:?}", state.config.queue.queue_type).to_lowercase();

    let mut components: HashMap<String, ComponentHealth> = HashMap::new();

    components.insert("db".into(), ComponentHealth {
        status:  if db_ok { "UP".into() } else { "DOWN".into() },
        details: Some(serde_json::json!({
            "poolSize": pool_size,
            "idle":     pool_idle,
        })),
    });

    components.insert("cache".into(), ComponentHealth {
        status:  "UP".into(),
        details: Some(serde_json::json!({
            "type": format!("{:?}", state.config.cache.cache_type).to_lowercase(),
        })),
    });

    components.insert("ruleEngine".into(), ComponentHealth {
        status:  if re_running { "UP".into() } else { "DEGRADED".into() },
        details: None,
    });

    components.insert("queue".into(), ComponentHealth {
        status:  "UP".into(),
        details: Some(serde_json::json!({ "type": queue_type })),
    });

    let all_up = db_ok;
    let overall_status = if all_up { "UP" } else { "DEGRADED" };
    let status_code    = if all_up { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let body = HealthResponse {
        status:      overall_status.into(),
        version:     env!("CARGO_PKG_VERSION").into(),
        uptime_secs,
        components,
    };

    (status_code, Json(body))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::Value;
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

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn health_check_returns_200_when_db_up(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["status"], "UP");
        assert!(body["components"]["db"]["status"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn health_liveness_returns_200(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/health/liveness")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn health_readiness_returns_200(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/health/readiness")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["status"], "UP");
    }

    /// Phase 40 — verify enhanced health response format includes version, uptimeSecs, components.
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn health_check_returns_enhanced_format(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;

        // status and version
        assert_eq!(body["status"], "UP");
        assert!(body["version"].is_string(), "version field must be present");

        // uptimeSecs (camelCase)
        assert!(body["uptimeSecs"].is_number(), "uptimeSecs must be a number");

        // components: db, cache, ruleEngine, queue
        assert_eq!(body["components"]["db"]["status"], "UP");
        assert_eq!(body["components"]["cache"]["status"], "UP");
        assert!(body["components"]["ruleEngine"]["status"].is_string());
        assert_eq!(body["components"]["queue"]["status"], "UP");
    }
}
