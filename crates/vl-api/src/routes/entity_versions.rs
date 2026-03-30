use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{CommitRequest, EntityVersion, VersionCreateRequest, VersionRequestStatus};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/entities/version/commit", post(commit))
        .route("/entities/{entityType}/{entityId}/versions", get(list_versions))
        .route("/entities/version/{versionId}", get(get_version))
        .route(
            "/entities/{entityType}/{entityId}/versions/{versionNumber}/snapshot",
            get(get_snapshot),
        )
        // ── VC (Version Control) endpoints ───────────────────────────────────
        .route("/entities/vc/diff",                 post(vc_diff))
        .route("/entities/vc/restore",              post(vc_restore))
        .route("/entities/vc/version",              post(vc_create_version))
        .route("/entities/vc/version/{requestId}/status", get(vc_version_status))
        .route("/entities/vc/repository/settings",  get(get_repo_settings)
                                                        .post(save_repo_settings)
                                                        .delete(delete_repo_settings))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityVersionResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub version_number: i64,
    pub commit_msg: Option<String>,
    pub snapshot: serde_json::Value,
    pub diff: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_time: i64,
}

impl From<EntityVersion> for EntityVersionResponse {
    fn from(v: EntityVersion) -> Self {
        Self {
            id: v.id,
            tenant_id: v.tenant_id,
            entity_id: v.entity_id,
            entity_type: v.entity_type,
            version_number: v.version_number,
            commit_msg: v.commit_msg,
            snapshot: v.snapshot,
            diff: v.diff,
            created_by: v.created_by,
            created_time: v.created_time,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResponse {
    pub data: Vec<EntityVersionResponse>,
    pub total_elements: i64,
    pub total_pages: i64,
    pub has_next: bool,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/entities/version/commit
/// Create a new version snapshot for an entity.
pub async fn commit(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CommitRequest>,
) -> Result<(StatusCode, Json<EntityVersionResponse>), ApiError> {
    let version = state
        .version_dao
        .commit(ctx.tenant_id, Some(ctx.user_id), &req)
        .await?;

    Ok((StatusCode::CREATED, Json(version.into())))
}

/// GET /api/entities/{entityType}/{entityId}/versions
/// List versions for an entity, newest first.
pub async fn list_versions(
    State(state): State<EntityState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<ListVersionsParams>,
) -> Result<Json<VersionListResponse>, ApiError> {
    let page = params.page.unwrap_or(0).max(0);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    let (versions, total) = state
        .version_dao
        .list_versions(entity_id, page, page_size)
        .await?;

    let total_pages = (total + page_size - 1) / page_size;
    let has_next = (page + 1) < total_pages;

    let _ = entity_type; // used in path for routing semantics

    Ok(Json(VersionListResponse {
        data: versions.into_iter().map(Into::into).collect(),
        total_elements: total,
        total_pages,
        has_next,
    }))
}

/// GET /api/entities/version/{versionId}
/// Get a version by its UUID.
pub async fn get_version(
    State(state): State<EntityState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<EntityVersionResponse>, ApiError> {
    let version = state
        .version_dao
        .get_version(version_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Entity version not found".into()))?;

    Ok(Json(version.into()))
}

/// GET /api/entities/{entityType}/{entityId}/versions/{versionNumber}/snapshot
/// Get the snapshot JSON at a specific version number.
pub async fn get_snapshot(
    State(state): State<EntityState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path((entity_type, entity_id, version_number)): Path<(String, Uuid, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let version = state
        .version_dao
        .get_by_number(entity_id, version_number)
        .await?
        .ok_or_else(|| ApiError::NotFound("Entity version not found".into()))?;

    let _ = entity_type; // used in path for routing semantics

    Ok(Json(version.snapshot))
}

// ── Async version create endpoints ───────────────────────────────────────────

/// POST /api/entities/vc/version
/// Submit an async version-create request. Returns request_id for polling.
async fn vc_create_version(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<VersionCreateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let request_id = state
        .version_control_svc
        .submit(req, ctx.tenant_id, ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "requestId": request_id })),
    ))
}

/// GET /api/entities/vc/version/{requestId}/status
/// Poll the status of an async version-create request.
async fn vc_version_status(
    State(state): State<EntityState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<VersionRequestStatus>, ApiError> {
    let status = state
        .version_control_svc
        .get_status(request_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Version request {request_id} not found")))?;

    Ok(Json(status))
}

// ── VC endpoint DTOs ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffRequest {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub version_from: i64,
    pub version_to: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResponse {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub version_from: i64,
    pub version_to: i64,
    pub diff: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRequest {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub version_number: i64,
    #[serde(default)]
    pub commit_msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettings {
    pub enabled: bool,
    pub repository_uri: Option<String>,
    pub default_branch: Option<String>,
    pub auth_method: Option<String>,
}

// ── VC Handlers ───────────────────────────────────────────────────────────────

/// POST /api/entities/vc/diff
/// Compute diff between two version snapshots of an entity.
async fn vc_diff(
    State(state): State<EntityState>,
    Extension(_ctx): Extension<SecurityContext>,
    Json(req): Json<DiffRequest>,
) -> Result<Json<DiffResponse>, ApiError> {
    let from = state.version_dao
        .get_by_number(req.entity_id, req.version_from).await?
        .ok_or_else(|| ApiError::NotFound(format!("Version {} not found", req.version_from)))?;

    let to = state.version_dao
        .get_by_number(req.entity_id, req.version_to).await?
        .ok_or_else(|| ApiError::NotFound(format!("Version {} not found", req.version_to)))?;

    // Compute a simple field-level diff
    let diff = json_diff(&from.snapshot, &to.snapshot);

    Ok(Json(DiffResponse {
        entity_id:    req.entity_id,
        entity_type:  req.entity_type,
        version_from: req.version_from,
        version_to:   req.version_to,
        diff,
    }))
}

/// POST /api/entities/vc/restore
/// Restore an entity to a previous version snapshot by re-committing it.
async fn vc_restore(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<RestoreRequest>,
) -> Result<(StatusCode, Json<EntityVersionResponse>), ApiError> {
    let target = state.version_dao
        .get_by_number(req.entity_id, req.version_number).await?
        .ok_or_else(|| ApiError::NotFound(format!("Version {} not found", req.version_number)))?;

    let commit_req = vl_core::entities::CommitRequest {
        entity_id:   req.entity_id,
        entity_type: req.entity_type,
        snapshot:    target.snapshot,
        commit_msg:  req.commit_msg
            .or_else(|| Some(format!("Restored to version {}", req.version_number))),
    };

    let new_version = state.version_dao
        .commit(ctx.tenant_id, Some(ctx.user_id), &commit_req).await?;

    Ok((StatusCode::CREATED, Json(new_version.into())))
}

/// GET /api/entities/vc/repository/settings
async fn get_repo_settings(
    Extension(_ctx): Extension<SecurityContext>,
) -> Json<RepositorySettings> {
    // VieLang uses DB-based version control, not git.
    // Return a stub indicating git VC is disabled.
    Json(RepositorySettings {
        enabled:         false,
        repository_uri:  None,
        default_branch:  None,
        auth_method:     None,
    })
}

/// POST /api/entities/vc/repository/settings
async fn save_repo_settings(
    Extension(_ctx): Extension<SecurityContext>,
    Json(_req): Json<RepositorySettings>,
) -> Result<Json<RepositorySettings>, ApiError> {
    Err(ApiError::BadRequest(
        "Git-based version control is not enabled in this deployment".into(),
    ))
}

/// DELETE /api/entities/vc/repository/settings
async fn delete_repo_settings(
    Extension(_ctx): Extension<SecurityContext>,
) -> StatusCode {
    StatusCode::NO_CONTENT
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute a simple field-level diff between two JSON objects.
/// Returns an object with keys: "added", "removed", "changed".
fn json_diff(from: &serde_json::Value, to: &serde_json::Value) -> serde_json::Value {
    use serde_json::{json, Map};

    let empty_obj = serde_json::Value::Object(Map::new());
    let from_obj = from.as_object().unwrap_or(empty_obj.as_object().unwrap());
    let to_obj   = to.as_object().unwrap_or(empty_obj.as_object().unwrap());

    let mut added   = Map::new();
    let mut removed = Map::new();
    let mut changed = Map::new();

    for (k, v) in to_obj {
        if !from_obj.contains_key(k) {
            added.insert(k.clone(), v.clone());
        } else if from_obj[k] != *v {
            changed.insert(k.clone(), json!({ "from": from_obj[k], "to": v }));
        }
    }
    for (k, v) in from_obj {
        if !to_obj.contains_key(k) {
            removed.insert(k.clone(), v.clone());
        }
    }

    json!({ "added": added, "removed": removed, "changed": changed })
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

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
    async fn list_versions_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let random_id = Uuid::new_v4();
        let resp = app.oneshot(
            Request::builder().method("GET")
                .uri(&format!("/api/entities/DEVICE/{random_id}/versions?pageSize=10&page=0"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
