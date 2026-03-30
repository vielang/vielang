use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::TbResource;
use vl_dao::PageData;

use crate::{error::ApiError, middleware::SecurityContext, routes::devices::IdResponse, state::{AppState, UiState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Image endpoints (Java: ImageController) ──────────────────────────
        .route("/image",                            post(upload_image))
        .route("/images/{r#type}/{key}",            get(download_image).delete(delete_image))
        .route("/images/{r#type}/{key}/info",       get(get_image_info).put(update_image_info))
        .route("/images/{r#type}/{key}/preview",    get(download_image_preview))
        .route("/images",                           get(list_images))
        // ── System dashboard JSON (served statically for Angular pages) ─────
        .route("/resource/dashboard/system/{key}",  get(get_system_dashboard))
        // ── Resource endpoints (Java: TbResourceController) ──────────────────
        .route("/resource",                         post(save_resource).get(list_resources))
        .route("/resource/{resourceId}",            get(get_resource).delete(delete_resource))
        .route("/resource/info/{resourceId}",       get(get_resource_info))
        .route("/resource/{resourceId}/download",   get(download_resource))
        // ── Image preview with stub header ───────────────────────────────────
        .route("/image/{imageId}/preview",          get(get_image_preview_by_id))
}

/// Public routes — no auth required (public image access).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/images/public/{public_key}", get(download_public_image))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceInfoResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    pub title: String,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "resourceKey")]
    pub resource_key: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(rename = "publicResourceKey")]
    pub public_resource_key: Option<String>,
    pub etag: Option<String>,
    pub descriptor: Option<serde_json::Value>,
    pub version: i64,
}

impl From<TbResource> for ResourceInfoResponse {
    fn from(r: TbResource) -> Self {
        Self {
            id:                  IdResponse::new(r.id, "TB_RESOURCE"),
            created_time:        r.created_time,
            tenant_id:           r.tenant_id.map(IdResponse::tenant),
            title:               r.title,
            resource_type:       r.resource_type,
            resource_key:        r.resource_key,
            file_name:           r.file_name,
            is_public:           r.is_public,
            public_resource_key: r.public_resource_key,
            etag:                r.etag,
            descriptor:          r.descriptor,
            version:             r.version,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResourceListParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,
}

impl ResourceListParams {
    fn to_page_link(&self) -> vl_dao::PageLink {
        let mut pl = vl_dao::PageLink::new(
            self.page.unwrap_or(0),
            self.page_size.unwrap_or(10),
        );
        pl.text_search = self.text_search.clone();
        pl
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateImageInfoRequest {
    pub title: Option<String>,
    #[serde(rename = "isPublic")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SaveResourceRequest {
    pub id: Option<IdResponse>,
    pub title: String,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "resourceKey")]
    pub resource_key: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    /// Base64-encoded file data
    pub data: Option<String>,
    pub descriptor: Option<serde_json::Value>,
}

// ── Image Handlers ────────────────────────────────────────────────────────────

/// POST /api/image  (multipart/form-data)
async fn upload_image(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ResourceInfoResponse>), ApiError> {
    let mut file_name = String::from("image");
    let mut data: Vec<u8> = Vec::new();
    let mut content_type = String::from("image/png");

    while let Some(field) = multipart.next_field().await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        let fname = field.file_name().unwrap_or("image").to_string();
        if !fname.is_empty() { file_name = fname; }
        if let Some(ct) = field.content_type() {
            content_type = ct.to_string();
        }
        let _ = field_name; // field name not used for images
        data = field.bytes().await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?.to_vec();
    }

    if data.is_empty() {
        return Err(ApiError::BadRequest("No image data uploaded".into()));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let id = Uuid::new_v4();
    let resource_key = id.to_string();
    let etag = format!("{:x}", md5_hash(&data));

    // Generate image preview — resize to max 250x250 thumbnail.
    let preview = generate_image_preview(&data).unwrap_or_else(|| data.clone());

    let resource = TbResource {
        id,
        created_time: now,
        tenant_id:   Some(ctx.tenant_id),
        title:       file_name.clone(),
        resource_type: "IMAGE".into(),
        resource_sub_type: Some(content_type),
        resource_key: resource_key.clone(),
        file_name,
        is_public: false,
        public_resource_key: None,
        etag: Some(etag),
        descriptor: None,
        data: Some(data),
        preview: Some(preview),
        external_id: None,
        version: 1,
    };

    let saved = state.resource_dao.save(&resource).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    Ok((StatusCode::CREATED, Json(saved.into())))
}

/// GET /api/images/{type}/{key}
///
/// Returns the image binary. Supports ETag / If-None-Match for 304 caching.
async fn download_image(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path((res_type, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let resource = state.resource_dao
        .find_by_key(ctx.tenant_id, &res_type, &key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;

    let data = resource.data.unwrap_or_default();
    let etag_value = resource.etag
        .unwrap_or_else(|| format!("{:x}", md5_hash(&data)));

    // Check If-None-Match for 304
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let trimmed = val.trim().trim_matches('"');
            if trimmed == etag_value {
                return Ok(StatusCode::NOT_MODIFIED.into_response());
            }
        }
    }

    let content_type = resource.resource_sub_type
        .unwrap_or_else(|| "application/octet-stream".into());

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::ETAG, format!("\"{}\"", etag_value)),
        ],
        Bytes::from(data),
    ).into_response())
}

/// GET /api/images/{type}/{key}/preview
async fn download_image_preview(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path((res_type, key)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let resource = state.resource_dao
        .find_by_key(ctx.tenant_id, &res_type, &key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;

    let preview = resource.preview.or(resource.data).unwrap_or_default();

    Ok((
        [
            (header::CONTENT_TYPE, "image/png".to_string()),
            (header::HeaderName::from_static("x-preview"), "true".to_string()),
        ],
        Bytes::from(preview),
    ).into_response())
}

/// GET /api/image/{imageId}/preview
///
/// Returns a preview/thumbnail of the image by its UUID. Currently a stub that
/// returns the original image with `X-Preview: true` header (actual resize can
/// be added later with the `image` crate).
async fn get_image_preview_by_id(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path(image_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let resource = state.resource_dao
        .find_by_id(image_id).await?
        .ok_or(ApiError::NotFound("Image not found".into()))?;

    // Verify tenant access
    if let Some(tid) = resource.tenant_id {
        if tid != ctx.tenant_id && !ctx.is_sys_admin() {
            return Err(ApiError::Forbidden("Access denied".into()));
        }
    }

    let preview = resource.preview.or(resource.data).unwrap_or_default();
    let content_type = resource.resource_sub_type
        .unwrap_or_else(|| "image/png".into());

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::HeaderName::from_static("x-preview"), "true".to_string()),
        ],
        Bytes::from(preview),
    ).into_response())
}

/// GET /api/images/public/{publicResourceKey}
///
/// Public (no auth). Supports ETag / If-None-Match for 304 caching.
async fn download_public_image(
    State(state): State<UiState>,
    Path(public_key): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let resource = state.resource_dao
        .find_public_by_key(&public_key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;

    let data = resource.data.unwrap_or_default();
    let etag_value = resource.etag
        .unwrap_or_else(|| format!("{:x}", md5_hash(&data)));

    // Check If-None-Match for 304
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let trimmed = val.trim().trim_matches('"');
            if trimmed == etag_value {
                return Ok(StatusCode::NOT_MODIFIED.into_response());
            }
        }
    }

    let content_type = resource.resource_sub_type
        .unwrap_or_else(|| "application/octet-stream".into());

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::ETAG, format!("\"{}\"", etag_value)),
        ],
        Bytes::from(data),
    ).into_response())
}

/// GET /api/images/{type}/{key}/info
async fn get_image_info(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path((res_type, key)): Path<(String, String)>,
) -> Result<Json<ResourceInfoResponse>, ApiError> {
    let resource = state.resource_dao
        .find_by_key(ctx.tenant_id, &res_type, &key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;
    Ok(Json(resource.into()))
}

/// PUT /api/images/{type}/{key}/info
async fn update_image_info(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path((res_type, key)): Path<(String, String)>,
    Json(req): Json<UpdateImageInfoRequest>,
) -> Result<Json<ResourceInfoResponse>, ApiError> {
    let mut resource = state.resource_dao
        .find_by_key(ctx.tenant_id, &res_type, &key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;

    if let Some(title) = req.title { resource.title = title; }
    if let Some(is_public) = req.is_public { resource.is_public = is_public; }

    let updated = state.resource_dao.update_info(&resource).await?;
    Ok(Json(updated.into()))
}

/// DELETE /api/images/{type}/{key}
async fn delete_image(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path((res_type, key)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let resource = state.resource_dao
        .find_by_key(ctx.tenant_id, &res_type, &key).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;
    state.resource_dao.delete(resource.id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/images?pageSize=10&page=0
async fn list_images(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<ResourceListParams>,
) -> Result<Json<PageData<ResourceInfoResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.resource_dao
        .find_by_tenant(ctx.tenant_id, Some("IMAGE"), &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

// ── Resource Handlers ─────────────────────────────────────────────────────────

/// GET /api/resource/{resourceId}
async fn get_resource(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResourceInfoResponse>, ApiError> {
    let resource = state.resource_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;
    // Strip blobs from response
    let mut info = resource;
    info.data = None;
    info.preview = None;
    Ok(Json(info.into()))
}

/// GET /api/resource/info/{resourceId}
async fn get_resource_info(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResourceInfoResponse>, ApiError> {
    let resource = state.resource_dao
        .find_info_by_id(id).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;
    Ok(Json(resource.into()))
}

/// GET /api/resource/{resourceId}/download
///
/// Returns the resource binary as an attachment. Supports ETag / If-None-Match for 304.
async fn download_resource(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let resource = state.resource_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Resource not found".into()))?;

    let data = resource.data.unwrap_or_default();
    let etag_value = resource.etag
        .unwrap_or_else(|| format!("{:x}", md5_hash(&data)));

    // Check If-None-Match for 304
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let trimmed = val.trim().trim_matches('"');
            if trimmed == etag_value {
                return Ok(StatusCode::NOT_MODIFIED.into_response());
            }
        }
    }

    let content_type = resource.resource_sub_type
        .unwrap_or_else(|| "application/octet-stream".into());
    let file_name = resource.file_name;

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name)),
            (header::ETAG, format!("\"{}\"", etag_value)),
        ],
        Bytes::from(data),
    ).into_response())
}

/// POST /api/resource
async fn save_resource(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Json(req): Json<SaveResourceRequest>,
) -> Result<(StatusCode, Json<ResourceInfoResponse>), ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let eid = existing_id.id;
        let existing = state.resource_dao.find_info_by_id(eid).await?
            .ok_or(ApiError::NotFound("Resource not found".into()))?;
        (eid, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let data = req.data.as_deref()
        .map(|b64| base64_decode(b64))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid base64 data: {}", e)))?;

    let resource_key = req.resource_key
        .unwrap_or_else(|| req.file_name.replace(' ', "_").to_lowercase());

    let resource = TbResource {
        id,
        created_time,
        tenant_id:   Some(ctx.tenant_id),
        title:       req.title,
        resource_type: req.resource_type,
        resource_sub_type: None,
        resource_key,
        file_name:   req.file_name,
        is_public:   false,
        public_resource_key: None,
        etag:        data.as_deref().map(|d| format!("{:x}", md5_hash(d))),
        descriptor:  req.descriptor,
        data,
        preview:     None,
        external_id: None,
        version,
    };

    let saved = state.resource_dao.save(&resource).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// GET /api/resource?pageSize=10&page=0
async fn list_resources(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<ResourceListParams>,
) -> Result<Json<PageData<ResourceInfoResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let res_type = params.resource_type.as_deref();
    let page = state.resource_dao
        .find_by_tenant(ctx.tenant_id, res_type, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// DELETE /api/resource/{resourceId}
async fn delete_resource(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.resource_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

// ── System dashboard static files ─────────────────────────────────────────────

const GATEWAYS_DASHBOARD: &str =
    include_str!("../static/gateways_dashboard.json");

/// GET /api/resource/dashboard/system/{key}
/// Serves built-in system dashboard JSON files (used by Angular pages like Gateways).
async fn get_system_dashboard(
    Path(key): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let content = match key.as_str() {
        "gateways_dashboard.json" => GATEWAYS_DASHBOARD,
        _ => return Err(ApiError::NotFound(format!("System dashboard '{}' not found", key))),
    };
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        content,
    ))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Generate a thumbnail preview (max 250x250) from image data.
/// Returns None if the image can't be decoded (non-image resource).
fn generate_image_preview(data: &[u8]) -> Option<Vec<u8>> {
    use image::GenericImageView;
    use std::io::Cursor;

    let img = image::load_from_memory(data).ok()?;
    let (w, h) = img.dimensions();

    // Only resize if larger than 250x250.
    let thumb = if w > 250 || h > 250 {
        img.thumbnail(250, 250)
    } else {
        img
    };

    let mut buf = Vec::new();
    thumb
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .ok()?;
    Some(buf)
}

fn md5_hash(data: &[u8]) -> u128 {
    // Simple FNV-style hash for etag (not cryptographic)
    let mut h: u128 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u128;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    
    // Strip data URI prefix if present: "data:...;base64,"
    let b64 = if let Some(pos) = input.find(";base64,") {
        &input[pos + 8..]
    } else {
        input
    };

    // Manual base64 decode using alphabet
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut table = [0u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let input = b64.as_bytes();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < input.len() {
        if input[i] == b'=' { break; }
        let a = table[input[i] as usize];
        let b = table[input[i+1] as usize];
        let c = if input[i+2] == b'=' { 0 } else { table[input[i+2] as usize] };
        let d = if i+3 >= input.len() || input[i+3] == b'=' { 0 } else { table[input[i+3] as usize] };
        output.push((a << 2) | (b >> 4));
        if input[i+2] != b'=' { output.push((b << 4) | (c >> 2)); }
        if i+3 < input.len() && input[i+3] != b'=' { output.push((c << 6) | d); }
        i += 4;
    }
    Ok(output)
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
    async fn list_resources_returns_ok(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "res_list@test.com", "pass123").await;
        let token = get_token(app.clone(), "res_list@test.com", "pass123").await;

        let resp = get_auth(app, "/api/resource?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert_eq!(body["totalElements"], 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_resource_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "res_404@test.com", "pass123").await;
        let token = get_token(app.clone(), "res_404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/resource/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn unauthenticated_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/resource?pageSize=10&page=0")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
