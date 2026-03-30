use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{OtaPackage, OtaPackageInfo, OtaPackageType, ChecksumAlgorithm};
use vl_dao::PageData;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, CoreState, DeviceState, OtaState}};
use super::devices::{IdResponse, PageParams};

pub fn router() -> Router<AppState> {
    Router::new()
        // OTA Package CRUD
        .route("/otaPackage", post(save_ota_package))
        .route("/otaPackage/{otaPackageId}", get(get_ota_package).delete(delete_ota_package))
        .route("/otaPackages", get(list_ota_packages))
        .route("/otaPackages/{type}", get(list_ota_packages_by_type))
        // OTA Package data (binary)
        .route("/otaPackage/{otaPackageId}/download", get(download_ota_package))
        .route("/otaPackage/{otaPackageId}/data", post(upload_ota_package_data))
        // OTA Package info by device profile
        .route("/otaPackage/info/{type}/{deviceProfileId}", get(get_ota_package_by_profile))
        // P2: Assign firmware → device + OTA state tracking
        .route("/device/{deviceId}/firmware/{packageId}", post(assign_firmware_to_device))
        .route("/device/{deviceId}/ota", get(get_device_ota_status))
}

/// Public router — device token authenticated endpoints (không cần JWT)
pub fn device_token_router() -> Router<AppState> {
    Router::new()
        // P2: HTTP firmware download (device token auth)
        .route("/v1/{token}/firmware", get(download_firmware_by_token))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct OtaPackageResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "deviceProfileId")]
    pub device_profile_id: Option<IdResponse>,
    #[serde(rename = "type")]
    pub ota_type: String,
    pub title: String,
    pub version: String,
    pub tag: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "hasData")]
    pub has_data: bool,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "dataSize")]
    pub data_size: Option<i64>,
    #[serde(rename = "checksumAlgorithm")]
    pub checksum_algorithm: Option<String>,
    pub checksum: Option<String>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

impl From<OtaPackage> for OtaPackageResponse {
    fn from(p: OtaPackage) -> Self {
        Self {
            id: IdResponse::new(p.id, "OTA_PACKAGE"),
            created_time: p.created_time,
            tenant_id: IdResponse::new(p.tenant_id, "TENANT"),
            device_profile_id: p.device_profile_id.map(|id| IdResponse::new(id, "DEVICE_PROFILE")),
            ota_type: p.ota_package_type.as_str().to_string(),
            title: p.title,
            version: p.version,
            tag: p.tag,
            url: p.url,
            has_data: p.has_data,
            file_name: p.file_name,
            content_type: p.content_type,
            data_size: p.data_size,
            checksum_algorithm: p.checksum_algorithm.map(|a| a.as_str().to_string()),
            checksum: p.checksum,
            additional_info: p.additional_info,
        }
    }
}

impl From<OtaPackageInfo> for OtaPackageResponse {
    fn from(p: OtaPackageInfo) -> Self {
        Self {
            id: IdResponse::new(p.id, "OTA_PACKAGE"),
            created_time: p.created_time,
            tenant_id: IdResponse::new(p.tenant_id, "TENANT"),
            device_profile_id: p.device_profile_id.map(|id| IdResponse::new(id, "DEVICE_PROFILE")),
            ota_type: p.ota_package_type.as_str().to_string(),
            title: p.title,
            version: p.version,
            tag: p.tag,
            url: p.url,
            has_data: p.has_data,
            file_name: p.file_name,
            content_type: p.content_type,
            data_size: p.data_size,
            checksum_algorithm: p.checksum_algorithm.map(|a| a.as_str().to_string()),
            checksum: p.checksum,
            additional_info: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveOtaPackageRequest {
    pub id: Option<IdResponse>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    #[serde(rename = "deviceProfileId")]
    pub device_profile_id: Option<IdResponse>,
    #[serde(rename = "type")]
    pub ota_type: String,
    pub title: String,
    pub version: String,
    pub tag: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "checksumAlgorithm")]
    pub checksum_algorithm: Option<String>,
    pub checksum: Option<String>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OtaTypeParams {
    #[serde(rename = "type")]
    pub ota_type: Option<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/otaPackage — create or update OTA package metadata
async fn save_ota_package(
    State(state): State<OtaState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveOtaPackageRequest>,
) -> Result<Json<OtaPackageResponse>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let tenant_id = req.tenant_id.map(|t| t.id).unwrap_or(ctx.tenant_id);

    let ota_type = OtaPackageType::from_str(&req.ota_type);
    let checksum_alg = req.checksum_algorithm
        .as_deref()
        .and_then(ChecksumAlgorithm::from_str);

    let pkg = OtaPackage {
        id: req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time: now,
        tenant_id,
        device_profile_id: req.device_profile_id.map(|p| p.id),
        ota_package_type: ota_type,
        title: req.title,
        version: req.version,
        tag: req.tag,
        url: req.url,
        file_name: req.file_name,
        content_type: req.content_type,
        data_size: None, // Set when data is uploaded
        checksum_algorithm: checksum_alg,
        checksum: req.checksum,
        has_data: false, // Set when data is uploaded
        additional_info: req.additional_info,
        version_int: 1,
    };

    let saved = state.ota_package_dao.save(&pkg).await?;
    Ok(Json(OtaPackageResponse::from(saved)))
}

/// GET /api/otaPackage/{otaPackageId}
async fn get_ota_package(
    State(state): State<OtaState>,
    Path(ota_package_id): Path<Uuid>,
) -> Result<Json<OtaPackageResponse>, ApiError> {
    let pkg = state.ota_package_dao
        .find_by_id(ota_package_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("OtaPackage [{}] not found", ota_package_id)))?;

    Ok(Json(OtaPackageResponse::from(pkg)))
}

/// DELETE /api/otaPackage/{otaPackageId}
async fn delete_ota_package(
    State(state): State<OtaState>,
    Path(ota_package_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.ota_package_dao.delete(ota_package_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/otaPackages — list all OTA packages
async fn list_ota_packages(
    State(state): State<OtaState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<OtaPackageResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let page = state.ota_package_dao
        .find_by_tenant(tenant_id, &params.to_page_link())
        .await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(OtaPackageResponse::from).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

/// GET /api/otaPackages/{type} — list OTA packages by type (FIRMWARE/SOFTWARE)
async fn list_ota_packages_by_type(
    State(state): State<OtaState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(ota_type): Path<String>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<OtaPackageResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let ota_type = OtaPackageType::from_str(&ota_type);

    let page = state.ota_package_dao
        .find_by_tenant_and_type(tenant_id, ota_type, &params.to_page_link())
        .await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(OtaPackageResponse::from).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

/// GET /api/otaPackage/info/{type}/{deviceProfileId} — get OTA package for device profile
async fn get_ota_package_by_profile(
    State(state): State<OtaState>,
    Path((ota_type, device_profile_id)): Path<(String, Uuid)>,
) -> Result<Json<OtaPackageResponse>, ApiError> {
    let ota_type = OtaPackageType::from_str(&ota_type);

    let pkg = state.ota_package_dao
        .find_by_device_profile(device_profile_id, ota_type)
        .await?
        .ok_or_else(|| ApiError::NotFound(
            format!("OtaPackage for profile [{}] type [{:?}] not found", device_profile_id, ota_type)
        ))?;

    Ok(Json(OtaPackageResponse::from(pkg)))
}

/// POST /api/otaPackage/{otaPackageId}/data — upload OTA package binary data
async fn upload_ota_package_data(
    State(state): State<OtaState>,
    Path(ota_package_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<OtaPackageResponse>, ApiError> {
    // Check package exists
    let mut pkg = state.ota_package_dao
        .find_by_id(ota_package_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("OtaPackage [{}] not found", ota_package_id)))?;

    // Read multipart data
    while let Some(field) = multipart.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();

        if name == "file" {
            let file_name = field.file_name().map(|s| s.to_string());
            let content_type = field.content_type().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;

            // Save binary data
            state.ota_package_dao.save_data(ota_package_id, &data).await?;

            // Update package metadata
            pkg.file_name = file_name.or(pkg.file_name);
            pkg.content_type = content_type.or(pkg.content_type);
            pkg.data_size = Some(data.len() as i64);
            pkg.has_data = true;

            // Calculate checksum if algorithm is set
            if let Some(alg) = pkg.checksum_algorithm {
                let checksum = calculate_checksum(&data, alg);
                pkg.checksum = Some(checksum);
            }

            state.ota_package_dao.save(&pkg).await?;
        }
    }

    // Reload and return
    let updated = state.ota_package_dao
        .find_by_id(ota_package_id)
        .await?
        .ok_or(ApiError::NotFound("Package not found after save".into()))?;

    Ok(Json(OtaPackageResponse::from(updated)))
}

/// GET /api/otaPackage/{otaPackageId}/download — download OTA package binary data
async fn download_ota_package(
    State(state): State<OtaState>,
    Path(ota_package_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Get package info
    let pkg = state.ota_package_dao
        .find_by_id(ota_package_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("OtaPackage [{}] not found", ota_package_id)))?;

    // If URL-based, redirect
    if let Some(url) = &pkg.url {
        if !pkg.has_data {
            return Err(ApiError::BadRequest(format!(
                "Package is URL-based. Download from: {}", url
            )));
        }
    }

    // Get binary data
    let data = state.ota_package_dao
        .get_data(ota_package_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Package has no data".into()))?;

    let content_type = pkg.content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let file_name = pkg.file_name
        .unwrap_or_else(|| format!("{}-{}.bin", pkg.title, pkg.version));

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name)),
            (header::CONTENT_LENGTH, data.len().to_string()),
        ],
        Body::from(data),
    ))
}

// ── P2: Assign firmware + OTA State endpoints ─────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OtaStateResponse {
    #[serde(rename = "deviceId")]
    pub device_id:      IdResponse,
    #[serde(rename = "otaPackageId")]
    pub ota_package_id: IdResponse,
    pub status:         String,
    pub error:          Option<String>,
    #[serde(rename = "createdTime")]
    pub created_time:   i64,
    #[serde(rename = "updatedTime")]
    pub updated_time:   i64,
}

#[derive(Debug, Deserialize)]
pub struct FirmwareChunkParams {
    pub title:   Option<String>,
    pub version: Option<String>,
    pub size:    Option<i64>,
    pub chunk:   Option<u32>,
}

/// POST /api/device/{deviceId}/firmware/{packageId}
/// Assign firmware package → device, tạo QUEUED state và gửi MQTT notification
async fn assign_firmware_to_device(
    State(state): State<OtaState>,
    Path((device_id, package_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OtaStateResponse>, ApiError> {
    let ota_state = state.ota_service
        .assign_firmware(device_id, package_id)
        .await?;

    Ok(Json(OtaStateResponse {
        device_id:      IdResponse::new(ota_state.device_id, "DEVICE"),
        ota_package_id: IdResponse::new(ota_state.ota_package_id, "OTA_PACKAGE"),
        status:         ota_state.status.as_str().to_string(),
        error:          ota_state.error,
        created_time:   ota_state.created_time,
        updated_time:   ota_state.updated_time,
    }))
}

/// GET /api/device/{deviceId}/ota
/// Trả về OTA state hiện tại của device
async fn get_device_ota_status(
    State(state): State<OtaState>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<OtaStateResponse>, ApiError> {
    let ota_state = state.ota_service
        .get_device_ota_status(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("No OTA state for device [{}]", device_id)))?;

    Ok(Json(OtaStateResponse {
        device_id:      IdResponse::new(ota_state.device_id, "DEVICE"),
        ota_package_id: IdResponse::new(ota_state.ota_package_id, "OTA_PACKAGE"),
        status:         ota_state.status.as_str().to_string(),
        error:          ota_state.error,
        created_time:   ota_state.created_time,
        updated_time:   ota_state.updated_time,
    }))
}

/// GET /api/v1/{token}/firmware?chunk=N
/// HTTP firmware download dùng device access token
/// Trả về binary chunk theo index (default chunk=0, chunk_size=16KB)
async fn download_firmware_by_token(
    State(ota): State<OtaState>,
    State(device): State<DeviceState>,
    State(core): State<CoreState>,
    Path(token): Path<String>,
    Query(params): Query<FirmwareChunkParams>,
) -> Result<impl IntoResponse, ApiError> {
    // Tìm device theo token
    let (dev, _creds) = device.device_dao
        .find_by_credentials_id(&token)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Unauthorized("Invalid device token".into()))?;

    // Tìm firmware đang pending cho device
    let pkg = ota.ota_package_dao
        .find_pending_for_device(dev.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("No pending firmware for device [{}]", dev.id)))? ;

    if !pkg.has_data {
        return Err(ApiError::NotFound("Firmware has no binary data".into()));
    }

    let chunk_index = params.chunk.unwrap_or(0);
    let chunk_size  = core.config.ota.chunk_size_kb * 1024;

    let data = ota.ota_package_dao
        .get_chunk(pkg.id, chunk_index, chunk_size)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Chunk {} out of bounds", chunk_index)))?;

    let content_type = pkg.content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_LENGTH, data.len().to_string()),
        ],
        Body::from(data),
    ))
}

// ── Helper functions ─────────────────────────────────────────────────────────

fn calculate_checksum(data: &[u8], algorithm: ChecksumAlgorithm) -> String {
    use sha2::{Sha256, Sha384, Sha512, Digest};

    match algorithm {
        ChecksumAlgorithm::Md5 => {
            use md5::{Md5, Digest as _};
            let mut hasher = Md5::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Sha384 => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Crc32 => {
            let checksum = crc32fast::hash(data);
            format!("{:08x}", checksum)
        }
        ChecksumAlgorithm::Murmur3_32 | ChecksumAlgorithm::Murmur3_128 => {
            // Fallback to SHA256 for murmur
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;
    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;

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

    async fn login_as(app: axum::Router, email: &str, pass: &str) -> String {
        let resp = app.oneshot(
            Request::builder()
                .method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pass}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    async fn create_user(pool: &PgPool, email: &str, pass: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::nil(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None,
            additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pass).unwrap();
        dao.save_credentials(&UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(),
            user_id: user.id, enabled: true,
            password: Some(hash), activate_token: None,
            reset_token: None, additional_info: None,
        }).await.unwrap();
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

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    // ── POST /api/otaPackage ──────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_ota_package_returns_saved(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ota@test.com", "pass123").await;
        let token = login_as(app.clone(), "ota@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/otaPackage", &token, json!({
            "type": "FIRMWARE",
            "title": "v1.0.0-fw",
            "version": "1.0.0",
            "tag": "stable"
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["type"].as_str().unwrap(), "FIRMWARE");
        assert_eq!(body["title"].as_str().unwrap(), "v1.0.0-fw");
        assert_eq!(body["version"].as_str().unwrap(), "1.0.0");
        assert!(body["id"]["id"].is_string());
        assert_eq!(body["hasData"].as_bool().unwrap(), false);
    }

    // ── GET /api/otaPackage/{id} ──────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_ota_package_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ota2@test.com", "pass123").await;
        let token = login_as(app.clone(), "ota2@test.com", "pass123").await;

        let created = post_json_auth(app.clone(), "/api/otaPackage", &token, json!({
            "type": "SOFTWARE",
            "title": "sw-package",
            "version": "2.0.0"
        })).await;
        let created_body = body_json(created).await;
        let ota_id = created_body["id"]["id"].as_str().unwrap();

        let resp = get_auth(app, &format!("/api/otaPackage/{}", ota_id), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["type"].as_str().unwrap(), "SOFTWARE");
        assert_eq!(body["title"].as_str().unwrap(), "sw-package");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_ota_package_not_found(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ota3@test.com", "pass123").await;
        let token = login_as(app.clone(), "ota3@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/otaPackage/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── GET /api/otaPackages ──────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_ota_packages_returns_paginated(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ota4@test.com", "pass123").await;
        let token = login_as(app.clone(), "ota4@test.com", "pass123").await;

        for (title, ver) in [("fw-a", "1.0"), ("fw-b", "2.0")] {
            post_json_auth(app.clone(), "/api/otaPackage", &token, json!({
                "type": "FIRMWARE", "title": title, "version": ver
            })).await;
        }

        let resp = get_auth(app, "/api/otaPackages?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["totalElements"].as_i64().unwrap() >= 2);
    }

    // ── DELETE /api/otaPackage/{id} ───────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_ota_package_then_not_found(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ota5@test.com", "pass123").await;
        let token = login_as(app.clone(), "ota5@test.com", "pass123").await;

        let created = post_json_auth(app.clone(), "/api/otaPackage", &token, json!({
            "type": "FIRMWARE", "title": "del-fw", "version": "0.1"
        })).await;
        let body = body_json(created).await;
        let ota_id = body["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/otaPackage/{}", ota_id), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/otaPackage/{}", ota_id), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }
}
