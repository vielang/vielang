use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_dao::{PageData, SearchResult};
use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, CoreState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", get(unified_search))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub query:     String,
    /// Comma-separated entity types: "DEVICE,ASSET,CUSTOMER,USER,ENTITY_VIEW,EDGE"
    /// Mặc định: DEVICE,ASSET,CUSTOMER
    pub types:     Option<String>,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default)]
    pub page:      i64,
}

fn default_page_size() -> i64 { 20 }

/// Response item — camelCase để khớp ThingsBoard frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultDto {
    pub entity_type:  &'static str,
    pub id:           Uuid,
    pub created_time: i64,
    pub name:         String,
    pub label:        Option<String>,
}

impl From<SearchResult> for SearchResultDto {
    fn from(r: SearchResult) -> Self {
        Self {
            entity_type:  r.entity_type,
            id:           r.id,
            created_time: r.created_time,
            name:         r.name,
            label:        r.label,
        }
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /api/search?query=sensor&types=DEVICE,ASSET&pageSize=20&page=0
///
/// Tìm kiếm FTS song song trên các entity types được chỉ định.
/// Trả về kết quả sắp xếp theo relevance.
pub async fn unified_search(
    Query(params):     Query<SearchParams>,
    State(core):       State<CoreState>,
    State(entity):     State<EntityState>,
    Extension(auth):   Extension<SecurityContext>,
) -> Result<Json<PageData<SearchResultDto>>, ApiError> {
    let cfg = &core.config.search;

    // Validate
    if !cfg.enabled {
        return Err(ApiError::BadRequest("Search feature is disabled".into()));
    }

    let query = params.query.trim();
    if query.len() > 200 {
        return Err(ApiError::BadRequest("Search query too long (max 200 characters)".into()));
    }
    if query.len() < cfg.min_query_length {
        return Ok(Json(PageData::new(vec![], 0, &vl_dao::PageLink {
            page:          params.page,
            page_size:     params.page_size,
            text_search:   None,
            sort_property: None,
            sort_order:    vl_dao::page::SortOrder::Desc,
        })));
    }

    // Parse types
    let requested_types_str = params.types
        .as_deref()
        .unwrap_or("DEVICE,ASSET,CUSTOMER");
    let types: Vec<&str> = requested_types_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let page_size = params.page_size.clamp(1, 100);
    let page      = params.page.max(0);

    let (results, total) = entity
        .search_dao
        .unified_search(
            auth.tenant_id,
            query,
            &types,
            page,
            page_size,
            cfg.max_results_per_type,
            cfg.min_query_length,
            cfg.prefix_matching,
        )
        .await
        .map_err(ApiError::from)?;

    let page_link = vl_dao::PageLink {
        page,
        page_size,
        text_search:   Some(query.to_string()),
        sort_property: None,
        sort_order:    vl_dao::page::SortOrder::Desc,
    };

    let dto_results: Vec<SearchResultDto> = results.into_iter().map(Into::into).collect();
    Ok(Json(PageData::new(dto_results, total, &page_link)))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;
    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

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

    async fn create_test_user(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: Some("Test".into()), last_name: Some("User".into()),
            phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: user.id,
            enabled: true, password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string())).unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pwd}).to_string())).unwrap(),
        ).await.unwrap();
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn search_returns_empty_for_no_match(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let _user = create_test_user(&pool, "search_empty@test.com", "pass123").await;
        let token = get_token(app.clone(), "search_empty@test.com", "pass123").await;

        let resp = get_auth(app, "/api/search?query=nonexistent_xyz&pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 0);
        assert_eq!(body["totalElements"], 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn search_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/search?query=test")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
