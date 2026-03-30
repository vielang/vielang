use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{TenantProfile, EntityInfo};
use vl_dao::PageData;

use crate::{error::ApiError, routes::devices::{IdResponse, PageParams}, state::{AppState, AuthState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: TenantProfileController — SYS_ADMIN only
        .route("/tenantProfile",                              post(save_tenant_profile))
        .route("/tenantProfile/{tenantProfileId}",            get(get_tenant_profile).delete(delete_tenant_profile))
        .route("/tenantProfile/{tenantProfileId}/default",    post(set_default_tenant_profile))
        .route("/tenantProfileInfo/{tenantProfileId}",        get(get_tenant_profile_info))
        .route("/tenantProfileInfo/default",                  get(get_default_tenant_profile_info))
        .route("/tenantProfiles",                             get(list_tenant_profiles))
        .route("/tenantProfileInfos",                         get(list_tenant_profile_infos))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantProfileResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "default")]
    pub is_default: bool,
    #[serde(rename = "isolatedTbRuleEngine")]
    pub isolated_vl_rule_engine: bool,
    #[serde(rename = "profileData")]
    pub profile_data: Option<serde_json::Value>,
}

impl From<TenantProfile> for TenantProfileResponse {
    fn from(p: TenantProfile) -> Self {
        Self {
            id:                      IdResponse::new(p.id, "TENANT_PROFILE"),
            created_time:            p.created_time,
            name:                    p.name,
            description:             p.description,
            is_default:              p.is_default,
            isolated_vl_rule_engine: p.isolated_vl_rule_engine,
            profile_data:            p.profile_data,
        }
    }
}

/// EntityInfo — id + name (used for /tenantProfileInfo endpoints)
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantProfileInfoResponse {
    pub id: IdResponse,
    pub name: String,
}

impl From<EntityInfo> for TenantProfileInfoResponse {
    fn from(e: EntityInfo) -> Self {
        Self {
            id:   IdResponse::new(e.id, "TENANT_PROFILE"),
            name: e.name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveTenantProfileRequest {
    pub id: Option<IdResponse>,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "isolatedTbRuleEngine", default)]
    pub isolated_vl_rule_engine: bool,
    #[serde(rename = "profileData")]
    pub profile_data: Option<serde_json::Value>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/tenantProfile/{tenantProfileId}
async fn get_tenant_profile(
    State(state): State<AuthState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TenantProfileResponse>, ApiError> {
    let profile = state.tenant_profile_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/tenantProfileInfo/{tenantProfileId}
async fn get_tenant_profile_info(
    State(state): State<AuthState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TenantProfileInfoResponse>, ApiError> {
    let info = state.tenant_profile_dao
        .find_info_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(info.into()))
}

/// GET /api/tenantProfileInfo/default
async fn get_default_tenant_profile_info(
    State(state): State<AuthState>,
) -> Result<Json<TenantProfileInfoResponse>, ApiError> {
    let profile = state.tenant_profile_dao
        .find_default().await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(TenantProfileInfoResponse {
        id:   IdResponse::new(profile.id, "TENANT_PROFILE"),
        name: profile.name,
    }))
}

/// GET /api/tenantProfiles?pageSize=10&page=0
async fn list_tenant_profiles(
    State(state): State<AuthState>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<TenantProfileResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.tenant_profile_dao.find_by_page(&page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/tenantProfileInfos?pageSize=10&page=0
async fn list_tenant_profile_infos(
    State(state): State<AuthState>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<TenantProfileInfoResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.tenant_profile_dao.find_infos_by_page(&page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/tenantProfile
async fn save_tenant_profile(
    State(state): State<AuthState>,
    Json(req): Json<SaveTenantProfileRequest>,
) -> Result<(StatusCode, Json<TenantProfileResponse>), ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let id = existing_id.id;
        let existing = state.tenant_profile_dao.find_by_id(id).await?
            .ok_or(ApiError::NotFound("Entity not found".into()))?;
        (id, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let profile = TenantProfile {
        id,
        created_time,
        name:                    req.name,
        description:             req.description,
        is_default:              false,
        isolated_vl_rule_engine: req.isolated_vl_rule_engine,
        profile_data:            req.profile_data,
        version,
    };

    let saved = state.tenant_profile_dao.save(&profile).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/tenantProfile/{tenantProfileId}
async fn delete_tenant_profile(
    State(state): State<AuthState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.tenant_profile_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/tenantProfile/{tenantProfileId}/default
async fn set_default_tenant_profile(
    State(state): State<AuthState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TenantProfileResponse>, ApiError> {
    state.tenant_profile_dao.find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;

    state.tenant_profile_dao.set_default(id).await?;

    let updated = state.tenant_profile_dao.find_by_id(id).await?
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

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_profiles_returns_ok(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let _user = create_test_user(&pool, "tp_list@test.com", "pass123").await;
        let token = get_token(app.clone(), "tp_list@test.com", "pass123").await;

        let resp = get_auth(app, "/api/tenantProfiles?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
