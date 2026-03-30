use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{DeviceProfile, DeviceProfileType, DeviceTransportType, DeviceProvisionType};
use vl_dao::PageData;

use crate::{error::ApiError, routes::devices::{IdResponse, PageParams}, state::{AppState, DeviceState, TelemetryState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: DeviceProfileController
        .route("/deviceProfile",                            post(save_device_profile))
        .route("/deviceProfile/{deviceProfileId}",          get(get_device_profile).delete(delete_device_profile))
        .route("/deviceProfile/{deviceProfileId}/default",  post(set_default_device_profile))
        .route("/deviceProfileInfo/{deviceProfileId}",      get(get_device_profile_info))
        .route("/deviceProfileInfo/default",                get(get_default_device_profile_info))
        .route("/deviceProfiles",                           get(list_device_profiles))
        .route("/deviceProfileInfos",                       get(list_device_profile_infos))
        .route("/deviceProfile/names",                      get(get_device_profile_names))
        .route("/deviceProfile/devices/keys/timeseries",    get(get_timeseries_keys))
        .route("/deviceProfile/devices/keys/attributes",    get(get_attribute_keys))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceProfileResponse {
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
    #[serde(rename = "type")]
    pub profile_type: String,
    #[serde(rename = "transportType")]
    pub transport_type: String,
    #[serde(rename = "provisionType")]
    pub provision_type: String,
    #[serde(rename = "profileData")]
    pub profile_data: Option<serde_json::Value>,
    #[serde(rename = "defaultRuleChainId")]
    pub default_rule_chain_id: Option<IdResponse>,
    #[serde(rename = "defaultDashboardId")]
    pub default_dashboard_id: Option<IdResponse>,
    #[serde(rename = "defaultQueueName")]
    pub default_queue_name: Option<String>,
    #[serde(rename = "firmwareId")]
    pub firmware_id: Option<IdResponse>,
    #[serde(rename = "softwareId")]
    pub software_id: Option<IdResponse>,
}

impl From<DeviceProfile> for DeviceProfileResponse {
    fn from(p: DeviceProfile) -> Self {
        Self {
            id:                   IdResponse::new(p.id, "DEVICE_PROFILE"),
            created_time:         p.created_time,
            tenant_id:            IdResponse::tenant(p.tenant_id),
            name:                 p.name,
            description:          p.description,
            image:                p.image,
            is_default:           p.is_default,
            profile_type:         format!("{:?}", p.device_profile_type).to_uppercase(),
            transport_type:       format!("{:?}", p.transport_type).to_uppercase(),
            provision_type:       provision_type_str(&p.provision_type),
            profile_data:         p.profile_data,
            default_rule_chain_id: p.default_rule_chain_id.map(|id| IdResponse::new(id, "RULE_CHAIN")),
            default_dashboard_id:  p.default_dashboard_id.map(|id| IdResponse::new(id, "DASHBOARD")),
            default_queue_name:    p.default_queue_name,
            firmware_id:           p.firmware_id.map(|id| IdResponse::new(id, "OTA_PACKAGE")),
            software_id:           p.software_id.map(|id| IdResponse::new(id, "OTA_PACKAGE")),
        }
    }
}

/// Lightweight variant — id + name + transport_type
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceProfileInfo {
    pub id: IdResponse,
    pub name: String,
    #[serde(rename = "default")]
    pub is_default: bool,
    #[serde(rename = "transportType")]
    pub transport_type: String,
}

impl From<DeviceProfile> for DeviceProfileInfo {
    fn from(p: DeviceProfile) -> Self {
        Self {
            id:             IdResponse::new(p.id, "DEVICE_PROFILE"),
            name:           p.name,
            is_default:     p.is_default,
            transport_type: format!("{:?}", p.transport_type).to_uppercase(),
        }
    }
}

/// EntityInfo — id + name only
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileEntityInfo {
    pub id: IdResponse,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveDeviceProfileRequest {
    pub id: Option<IdResponse>,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "type")]
    pub profile_type: Option<String>,
    #[serde(rename = "transportType")]
    pub transport_type: Option<String>,
    #[serde(rename = "provisionType")]
    pub provision_type: Option<String>,
    #[serde(rename = "profileData")]
    pub profile_data: Option<serde_json::Value>,
    #[serde(rename = "defaultRuleChainId")]
    pub default_rule_chain_id: Option<IdResponse>,
    #[serde(rename = "defaultDashboardId")]
    pub default_dashboard_id: Option<IdResponse>,
    #[serde(rename = "defaultQueueName")]
    pub default_queue_name: Option<String>,
    #[serde(rename = "firmwareId")]
    pub firmware_id: Option<IdResponse>,
    #[serde(rename = "softwareId")]
    pub software_id: Option<IdResponse>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceProfileInfosParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
    #[serde(rename = "transportType")]
    pub transport_type: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/deviceProfile/{deviceProfileId}
async fn get_device_profile(
    State(state): State<DeviceState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceProfileResponse>, ApiError> {
    let profile = state.device_profile_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/deviceProfileInfo/{deviceProfileId}
async fn get_device_profile_info(
    State(state): State<DeviceState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceProfileInfo>, ApiError> {
    let profile = state.device_profile_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/deviceProfileInfo/default
async fn get_default_device_profile_info(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
) -> Result<Json<DeviceProfileInfo>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let profile = state.device_profile_dao
        .find_default(tenant_id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(profile.into()))
}

/// GET /api/deviceProfiles?pageSize=10&page=0&textSearch=
async fn list_device_profiles(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<DeviceProfileResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page_link = params.to_page_link();
    let page = state.device_profile_dao
        .find_by_tenant(tenant_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/deviceProfileInfos?pageSize=10&page=0&transportType=MQTT
async fn list_device_profile_infos(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<DeviceProfileInfosParams>,
) -> Result<Json<PageData<DeviceProfileInfo>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page_link = {
        let mut pl = vl_dao::PageLink::new(
            params.page.unwrap_or(0),
            params.page_size.unwrap_or(10),
        );
        pl.text_search = params.text_search;
        pl
    };

    let page = state.device_profile_dao
        .find_by_tenant(tenant_id, &page_link).await?;

    // Filter by transport_type if provided
    let data: Vec<DeviceProfileInfo> = page.data
        .into_iter()
        .filter(|p| {
            if let Some(tt) = &params.transport_type {
                format!("{:?}", p.transport_type).eq_ignore_ascii_case(tt)
            } else {
                true
            }
        })
        .map(Into::into)
        .collect();

    Ok(Json(PageData {
        has_next:       page.has_next,
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        data,
    }))
}

/// GET /api/deviceProfile/names
async fn get_device_profile_names(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
) -> Result<Json<Vec<ProfileEntityInfo>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let names = state.device_profile_dao.find_names_by_tenant(tenant_id).await?;
    let result = names.into_iter().map(|(id, name)| ProfileEntityInfo {
        id: IdResponse::new(id, "DEVICE_PROFILE"),
        name,
    }).collect();
    Ok(Json(result))
}

/// GET /api/deviceProfile/devices/keys/timeseries
async fn get_timeseries_keys(
    State(state): State<TelemetryState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let profile_id = params.get("deviceProfileId").and_then(|s| Uuid::parse_str(s).ok());

    let keys = state.kv_dao
        .find_timeseries_keys_by_tenant(tenant_id, profile_id).await?;
    Ok(Json(keys))
}

/// GET /api/deviceProfile/devices/keys/attributes
async fn get_attribute_keys(
    State(state): State<TelemetryState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<String>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let profile_id = params.get("deviceProfileId").and_then(|s| Uuid::parse_str(s).ok());

    let keys = state.kv_dao
        .find_attribute_keys_by_tenant(tenant_id, profile_id).await?;
    Ok(Json(keys))
}

/// POST /api/deviceProfile
async fn save_device_profile(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Json(req): Json<SaveDeviceProfileRequest>,
) -> Result<(StatusCode, Json<DeviceProfileResponse>), ApiError> {
    let tenant_id = ctx.tenant_id;
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let id = existing_id.id;
        let existing = state.device_profile_dao.find_by_id(id).await?
            .ok_or(ApiError::NotFound("Entity not found".into()))?;
        (id, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let profile = DeviceProfile {
        id,
        created_time,
        tenant_id,
        name: req.name,
        description: req.description,
        image: req.image,
        is_default: false,
        device_profile_type: DeviceProfileType::Default,
        transport_type: parse_transport_type_str(
            req.transport_type.as_deref().unwrap_or("DEFAULT")
        ),
        provision_type: parse_provision_type_str(
            req.provision_type.as_deref().unwrap_or("DISABLED")
        ),
        profile_data: req.profile_data,
        default_rule_chain_id: req.default_rule_chain_id.map(|r| r.id),
        default_dashboard_id: req.default_dashboard_id.map(|r| r.id),
        default_queue_name: req.default_queue_name,
        default_edge_rule_chain_id: None,
        provision_device_key: None,
        firmware_id: req.firmware_id.map(|r| r.id),
        software_id: req.software_id.map(|r| r.id),
        external_id: None,
        version,
    };

    let saved = state.device_profile_dao.save(&profile).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/deviceProfile/{deviceProfileId}
async fn delete_device_profile(
    State(state): State<DeviceState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.device_profile_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/deviceProfile/{deviceProfileId}/default
async fn set_default_device_profile(
    State(state): State<DeviceState>,
    axum::extract::Extension(ctx): axum::extract::Extension<crate::middleware::SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceProfileResponse>, ApiError> {
    let tenant_id = ctx.tenant_id;

    // Kiểm tra tồn tại
    state.device_profile_dao.find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;

    state.device_profile_dao.set_default(tenant_id, id).await?;

    let updated = state.device_profile_dao.find_by_id(id).await?
        .ok_or(ApiError::NotFound("Entity not found".into()))?;
    Ok(Json(updated.into()))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn provision_type_str(t: &DeviceProvisionType) -> String {
    match t {
        DeviceProvisionType::Disabled                 => "DISABLED".into(),
        DeviceProvisionType::AllowCreateNewDevices    => "ALLOW_CREATE_NEW_DEVICES".into(),
        DeviceProvisionType::CheckPreProvisionedDevices => "CHECK_PRE_PROVISIONED_DEVICES".into(),
        DeviceProvisionType::X509CertificateChain     => "X509_CERTIFICATE_CHAIN".into(),
    }
}

fn parse_transport_type_str(s: &str) -> DeviceTransportType {
    match s.to_uppercase().as_str() {
        "MQTT"  => DeviceTransportType::Mqtt,
        "COAP"  => DeviceTransportType::Coap,
        "LWM2M" => DeviceTransportType::Lwm2m,
        "SNMP"  => DeviceTransportType::Snmp,
        _       => DeviceTransportType::Default,
    }
}

fn parse_provision_type_str(s: &str) -> DeviceProvisionType {
    match s.to_uppercase().as_str() {
        "ALLOW_CREATE_NEW_DEVICES"      => DeviceProvisionType::AllowCreateNewDevices,
        "CHECK_PRE_PROVISIONED_DEVICES" => DeviceProvisionType::CheckPreProvisionedDevices,
        "X509_CERTIFICATE_CHAIN"        => DeviceProvisionType::X509CertificateChain,
        _                               => DeviceProvisionType::Disabled,
    }
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
    async fn create_device_profile_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dp_cr@test.com", "pass123").await;
        let token = get_token(app.clone(), "dp_cr@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/deviceProfile", &token, json!({
            "name": "Test Profile",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
            "type": "DEFAULT",
            "transportType": "DEFAULT",
            "provisionType": "DISABLED",
        })).await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;
        assert_eq!(body["name"], "Test Profile");
        assert_eq!(body["id"]["entityType"], "DEVICE_PROFILE");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_device_profile_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dp_get@test.com", "pass123").await;
        let token = get_token(app.clone(), "dp_get@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/deviceProfile", &token, json!({
            "name": "Get Profile",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
            "type": "DEFAULT",
            "transportType": "DEFAULT",
            "provisionType": "DISABLED",
        })).await;
        let profile_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/deviceProfile/{profile_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["name"], "Get Profile");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_device_profiles(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dp_list@test.com", "pass123").await;
        let token = get_token(app.clone(), "dp_list@test.com", "pass123").await;

        for i in 0..3u32 {
            post_json_auth(app.clone(), "/api/deviceProfile", &token, json!({
                "name": format!("Profile-{i}"),
                "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
                "type": "DEFAULT",
                "transportType": "DEFAULT",
                "provisionType": "DISABLED",
            })).await;
        }

        let resp = get_auth(app, "/api/deviceProfiles?pageSize=2&page=0", &token).await;
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
        let user = create_test_user(&pool, "dp_404@test.com", "pass123").await;
        let token = get_token(app.clone(), "dp_404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/deviceProfile/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
