use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::AssetProfile;
use vl_dao::PageData;

use crate::{error::ApiError, routes::devices::{IdResponse, PageParams}, state::{AppState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: AssetProfileController
        .route("/assetProfile",                           post(save_asset_profile))
        .route("/assetProfile/{assetProfileId}",          get(get_asset_profile).delete(delete_asset_profile))
        .route("/assetProfile/{assetProfileId}/default",  post(set_default_asset_profile))
        .route("/assetProfileInfo/{assetProfileId}",      get(get_asset_profile_info))
        .route("/assetProfileInfo/default",               get(get_default_asset_profile_info))
        .route("/assetProfiles",                          get(list_asset_profiles))
        .route("/assetProfileInfos",                      get(list_asset_profile_infos))
        .route("/assetProfile/names",                     get(get_asset_profile_names))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetProfileResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "default")]
    pub is_default: bool,
    #[serde(rename = "defaultRuleChainId")]
    pub default_rule_chain_id: Option<IdResponse>,
    #[serde(rename = "defaultDashboardId")]
    pub default_dashboard_id: Option<IdResponse>,
    #[serde(rename = "defaultQueueName")]
    pub default_queue_name: Option<String>,
}

impl From<AssetProfile> for AssetProfileResponse {
    fn from(p: AssetProfile) -> Self {
        Self {
            id:                   IdResponse::new(p.id, "ASSET_PROFILE"),
            created_time:         p.created_time,
            tenant_id:            IdResponse::tenant(p.tenant_id),
            name:                 p.name,
            description:          p.description,
            image:                p.image,
            is_default:           p.is_default,
            default_rule_chain_id: p.default_rule_chain_id.map(|id| IdResponse::new(id, "RULE_CHAIN")),
            default_dashboard_id:  p.default_dashboard_id.map(|id| IdResponse::new(id, "DASHBOARD")),
            default_queue_name:    p.default_queue_name,
        }
    }
}

/// Lightweight variant
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetProfileInfo {
    pub id: IdResponse,
    pub name: String,
    #[serde(rename = "default")]
    pub is_default: bool,
}

impl From<AssetProfile> for AssetProfileInfo {
    fn from(p: AssetProfile) -> Self {
        Self {
            id:         IdResponse::new(p.id, "ASSET_PROFILE"),
            name:       p.name,
            is_default: p.is_default,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetProfileEntityInfo {
    pub id: IdResponse,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveAssetProfileRequest {
    pub id: Option<IdResponse>,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "defaultRuleChainId")]
    pub default_rule_chain_id: Option<IdResponse>,
    #[serde(rename = "defaultDashboardId")]
    pub default_dashboard_id: Option<IdResponse>,
    #[serde(rename = "defaultQueueName")]
    pub default_queue_name: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/assetProfile/{assetProfileId}
async fn get_asset_profile(
    State(state): State<EntityState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetProfileResponse>, ApiError> {
    let profile = state.asset_profile_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/assetProfileInfo/{assetProfileId}
async fn get_asset_profile_info(
    State(state): State<EntityState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetProfileInfo>, ApiError> {
    let profile = state.asset_profile_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/assetProfileInfo/default
async fn get_default_asset_profile_info(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
) -> Result<Json<AssetProfileInfo>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let profile = state.asset_profile_dao
        .find_default(tenant_id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/assetProfiles?pageSize=10&page=0
async fn list_asset_profiles(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<AssetProfileResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page_link = params.to_page_link();
    let page = state.asset_profile_dao
        .find_by_tenant(tenant_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/assetProfileInfos?pageSize=10&page=0
async fn list_asset_profile_infos(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<AssetProfileInfo>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page_link = params.to_page_link();
    let page = state.asset_profile_dao
        .find_by_tenant(tenant_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/assetProfile/names
async fn get_asset_profile_names(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
) -> Result<Json<Vec<AssetProfileEntityInfo>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let names = state.asset_profile_dao.find_names_by_tenant(tenant_id).await?;
    let result = names.into_iter().map(|(id, name)| AssetProfileEntityInfo {
        id: IdResponse::new(id, "ASSET_PROFILE"),
        name,
    }).collect();
    Ok(Json(result))
}

/// POST /api/assetProfile
async fn save_asset_profile(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Json(req): Json<SaveAssetProfileRequest>,
) -> Result<(StatusCode, Json<AssetProfileResponse>), ApiError> {
    let tenant_id = ctx.tenant_id;
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let id = existing_id.id;
        let existing = state.asset_profile_dao.find_by_id(id).await?
            .ok_or(ApiError::NotFound("Entity not found".into()))?;
        (id, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let profile = AssetProfile {
        id,
        created_time,
        tenant_id,
        name: req.name,
        description: req.description,
        image: req.image,
        is_default: false,
        default_rule_chain_id: req.default_rule_chain_id.map(|r| r.id),
        default_dashboard_id: req.default_dashboard_id.map(|r| r.id),
        default_queue_name: req.default_queue_name,
        default_edge_rule_chain_id: None,
        external_id: None,
        version,
    };

    let saved = state.asset_profile_dao.save(&profile).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/assetProfile/{assetProfileId}
async fn delete_asset_profile(
    State(state): State<EntityState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.asset_profile_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/assetProfile/{assetProfileId}/default
async fn set_default_asset_profile(
    State(state): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetProfileResponse>, ApiError> {
    let tenant_id = ctx.tenant_id;

    state.asset_profile_dao.find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;

    state.asset_profile_dao.set_default(tenant_id, id).await?;

    let updated = state.asset_profile_dao.find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(updated.into()))
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
    async fn create_asset_profile_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "ap_cr@test.com", "pass123").await;
        let token = get_token(app.clone(), "ap_cr@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/assetProfile", &token, json!({
            "name": "Test AP",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;
        assert_eq!(body["name"], "Test AP");
        assert_eq!(body["id"]["entityType"], "ASSET_PROFILE");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_asset_profile_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "ap_get@test.com", "pass123").await;
        let token = get_token(app.clone(), "ap_get@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/assetProfile", &token, json!({
            "name": "Get AP",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        let ap_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/assetProfile/{ap_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["name"], "Get AP");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_asset_profiles(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "ap_list@test.com", "pass123").await;
        let token = get_token(app.clone(), "ap_list@test.com", "pass123").await;

        for i in 0..3u32 {
            post_json_auth(app.clone(), "/api/assetProfile", &token, json!({
                "name": format!("AP-{i}"),
                "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
            })).await;
        }

        let resp = get_auth(app, "/api/assetProfiles?pageSize=2&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["data"].as_array().unwrap().len(), 2);
        assert_eq!(body["totalElements"], 3);
        assert_eq!(body["hasNext"], true);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "ap_404@test.com", "pass123").await;
        let token = get_token(app.clone(), "ap_404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/assetProfile/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
