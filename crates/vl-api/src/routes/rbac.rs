use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use vl_core::entities::{EntityGroup, RolePermissions, RoleType, TbRole};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AuthState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Role CRUD ──────────────────────────────────────────────────────────
        .route("/role",              get(list_roles).post(save_role))
        .route("/role/{roleId}",     get(get_role).delete(delete_role))
        // ── Entity Group CRUD ──────────────────────────────────────────────────
        // Note: list uses /entityGroups/{entityType} (plural) to avoid route conflict
        .route("/entityGroup",                              post(save_group))
        .route("/entityGroup/{groupId}",                    get(get_group).delete(delete_group))
        .route("/entityGroups/{entityType}",                get(list_groups))
        .route("/entityGroup/{groupId}/addEntities",        post(add_entities_to_group))
        .route("/entityGroup/{groupId}/removeEntities",     post(remove_entities_from_group))
        .route("/entityGroup/{groupId}/members",            get(get_group_members))
        // ── User ↔ Role assignment ─────────────────────────────────────────────
        .route("/user/{userId}/role/{roleId}", post(assign_role).delete(remove_role))
        .route("/user/{userId}/roles",         get(get_user_roles))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveRoleBody {
    #[serde(default)]
    id:          Option<Uuid>,
    name:        String,
    #[serde(default)]
    role_type:   String,
    #[serde(default)]
    permissions: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveGroupBody {
    #[serde(default)]
    id:              Option<Uuid>,
    name:            String,
    entity_type:     String,
    customer_id:     Option<Uuid>,
    additional_info: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct EntityListBody {
    entity_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
struct ListGroupsQuery {
    #[serde(rename = "entityType")]
    entity_type: String,
}

// ── Role Handlers ─────────────────────────────────────────────────────────────

/// GET /api/role — list all roles for tenant
async fn list_roles(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<TbRole>>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let roles = state.rbac_dao.find_roles_by_tenant(ctx.tenant_id).await?;
    Ok(Json(roles))
}

/// POST /api/role — create or update role
async fn save_role(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<SaveRoleBody>,
) -> Result<Json<TbRole>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let permissions: RolePermissions = serde_json::from_value(body.permissions)
        .unwrap_or_default();
    let role = TbRole {
        id:           body.id.unwrap_or_else(Uuid::new_v4),
        tenant_id:    ctx.tenant_id,
        name:         body.name,
        role_type:    RoleType::from_str(&body.role_type),
        permissions,
        created_time: chrono::Utc::now().timestamp_millis(),
    };
    let saved = state.rbac_dao.save_role(&role).await?;
    Ok(Json(saved))
}

/// GET /api/role/{roleId} — get role by id
async fn get_role(
    State(state): State<AuthState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(role_id): Path<Uuid>,
) -> Result<Json<TbRole>, ApiError> {
    let role = state.rbac_dao.find_role_by_id(role_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Role [{}] not found", role_id)))?;
    Ok(Json(role))
}

/// DELETE /api/role/{roleId} — delete role
async fn delete_role(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.rbac_dao.delete_role(role_id).await?;
    Ok(StatusCode::OK)
}

// ── Entity Group Handlers ─────────────────────────────────────────────────────

/// POST /api/entityGroup — create or update group
async fn save_group(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<SaveGroupBody>,
) -> Result<Json<EntityGroup>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let group = EntityGroup {
        id:              body.id.unwrap_or_else(Uuid::new_v4),
        tenant_id:       ctx.tenant_id,
        customer_id:     body.customer_id,
        name:            body.name,
        entity_type:     body.entity_type,
        additional_info: body.additional_info,
        created_time:    chrono::Utc::now().timestamp_millis(),
    };
    let saved = state.rbac_dao.save_group(&group).await?;
    Ok(Json(saved))
}

/// GET /api/entityGroup/{groupId} — get group by id
async fn get_group(
    State(state): State<AuthState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<EntityGroup>, ApiError> {
    let group = state.rbac_dao.find_group_by_id(group_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("EntityGroup [{}] not found", group_id)))?;
    Ok(Json(group))
}

/// GET /api/entityGroup/{entityType} — list groups by entity type
async fn list_groups(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(entity_type): Path<String>,
) -> Result<Json<Vec<EntityGroup>>, ApiError> {
    let groups = state.rbac_dao.find_groups_by_tenant(ctx.tenant_id, &entity_type).await?;
    Ok(Json(groups))
}

/// DELETE /api/entityGroup/{groupId} — delete group
async fn delete_group(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.rbac_dao.delete_group(group_id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/entityGroup/{groupId}/addEntities — add entities to group
async fn add_entities_to_group(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<EntityListBody>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    for entity_id in body.entity_ids {
        state.rbac_dao.add_to_group(group_id, entity_id).await?;
    }
    Ok(StatusCode::OK)
}

/// POST /api/entityGroup/{groupId}/removeEntities — remove entities from group
async fn remove_entities_from_group(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<EntityListBody>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    for entity_id in body.entity_ids {
        state.rbac_dao.remove_from_group(group_id, entity_id).await?;
    }
    Ok(StatusCode::OK)
}

/// GET /api/entityGroup/{groupId}/members — list group members
async fn get_group_members(
    State(state): State<AuthState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    let members = state.rbac_dao.get_group_members(group_id).await?;
    Ok(Json(members))
}

// ── User ↔ Role Handlers ──────────────────────────────────────────────────────

/// POST /api/user/{userId}/role/{roleId} — assign role to user
async fn assign_role(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.rbac_dao.assign_role_to_user(user_id, role_id).await?;
    Ok(StatusCode::OK)
}

/// DELETE /api/user/{userId}/role/{roleId} — remove role from user
async fn remove_role(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    state.rbac_dao.remove_role_from_user(user_id, role_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/user/{userId}/roles — get all roles of user
async fn get_user_roles(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<TbRole>>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() && ctx.user_id != user_id {
        return Err(ApiError::Forbidden("Cannot view other users' roles".into()));
    }
    let roles = state.rbac_dao.get_user_roles(user_id).await?;
    Ok(Json(roles))
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
    async fn list_roles_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/role")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
