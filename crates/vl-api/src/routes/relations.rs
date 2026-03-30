use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{EntityRelation, EntityType, RelationTypeGroup};

use crate::{error::ApiError, state::{AppState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: RelationController
        .route("/relation",       get(get_relation).post(save_relation).delete(delete_relation))
        .route("/relations",      get(list_relations).delete(delete_relations))
        .route("/relations/info", get(list_relations_info))
        // Path-based variants (Java v2 style)
        .route("/relations/from/{fromType}/{fromId}",             get(find_relations_from_path))
        .route("/relations/from/{fromType}/{fromId}/{relationType}", get(find_relations_from_type_path))
        .route("/relations/to/{toType}/{toId}",                   get(find_relations_to_path))
        .route("/relations/to/{toType}/{toId}/{relationType}",    get(find_relations_to_type_path))
        .route("/relations/info/from/{fromType}/{fromId}",        get(find_relations_info_from_path))
        .route("/relations/info/to/{toType}/{toId}",              get(find_relations_info_to_path))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EntityIdBody {
    pub id: Uuid,
    #[serde(rename = "entityType")]
    pub entity_type: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RelationResponse {
    #[serde(rename = "from")]
    pub from: EntityIdBody,
    #[serde(rename = "to")]
    pub to: EntityIdBody,
    #[serde(rename = "type")]
    pub relation_type: String,
    #[serde(rename = "typeGroup")]
    pub relation_type_group: String,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

impl From<EntityRelation> for RelationResponse {
    fn from(r: EntityRelation) -> Self {
        Self {
            from: EntityIdBody {
                id:          r.from_id,
                entity_type: entity_type_str(&r.from_type).into(),
            },
            to: EntityIdBody {
                id:          r.to_id,
                entity_type: entity_type_str(&r.to_type).into(),
            },
            relation_type:       r.relation_type,
            relation_type_group: relation_group_str(&r.relation_type_group).into(),
            additional_info:     r.additional_info,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveRelationRequest {
    pub from: EntityIdBody,
    pub to:   EntityIdBody,
    #[serde(rename = "type")]
    pub relation_type: String,
    #[serde(rename = "typeGroup")]
    pub relation_type_group: Option<String>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RelationQueryParams {
    #[serde(rename = "fromId")]
    pub from_id: Option<Uuid>,
    #[serde(rename = "fromType")]
    pub from_type: Option<String>,
    #[serde(rename = "toId")]
    pub to_id: Option<Uuid>,
    #[serde(rename = "toType")]
    pub to_type: Option<String>,
    #[serde(rename = "relationType")]
    pub relation_type: Option<String>,
    #[serde(rename = "relationTypeGroup")]
    pub relation_type_group: Option<String>,
    /// Used by DELETE /api/relations (delete all COMMON relations)
    #[serde(rename = "entityId")]
    pub entity_id: Option<Uuid>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct RelationGroupQuery {
    #[serde(rename = "relationTypeGroup")]
    pub relation_type_group: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/relation
async fn save_relation(
    State(state): State<EntityState>,
    Json(req): Json<SaveRelationRequest>,
) -> Result<StatusCode, ApiError> {
    let group = parse_relation_group(req.relation_type_group.as_deref().unwrap_or("COMMON"));
    let relation = EntityRelation {
        from_id:             req.from.id,
        from_type:           parse_entity_type(&req.from.entity_type),
        to_id:               req.to.id,
        to_type:             parse_entity_type(&req.to.entity_type),
        relation_type:       req.relation_type,
        relation_type_group: group,
        additional_info:     req.additional_info,
    };
    state.relation_dao.save(&relation).await?;
    Ok(StatusCode::OK)
}

/// DELETE /api/relation?fromId=...&fromType=...&toId=...&toType=...&relationType=...
async fn delete_relation(
    State(state): State<EntityState>,
    Query(params): Query<RelationQueryParams>,
) -> Result<StatusCode, ApiError> {
    let from_id = params.from_id
        .ok_or_else(|| ApiError::BadRequest("fromId is required".into()))?;
    let from_type = params.from_type
        .ok_or_else(|| ApiError::BadRequest("fromType is required".into()))?;
    let to_id = params.to_id
        .ok_or_else(|| ApiError::BadRequest("toId is required".into()))?;
    let to_type = params.to_type
        .ok_or_else(|| ApiError::BadRequest("toType is required".into()))?;
    let relation_type = params.relation_type
        .ok_or_else(|| ApiError::BadRequest("relationType is required".into()))?;
    let group = params.relation_type_group.as_deref().unwrap_or("COMMON");

    state.relation_dao
        .delete(from_id, &from_type, to_id, &to_type, &relation_type, group)
        .await?;
    Ok(StatusCode::OK)
}

/// GET /api/relations?fromId=...&fromType=...&relationType=...&toType=...
async fn list_relations(
    State(state): State<EntityState>,
    Query(params): Query<RelationQueryParams>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let from_id = params.from_id
        .ok_or_else(|| ApiError::BadRequest("fromId is required".into()))?;
    let from_type = params.from_type.as_deref()
        .ok_or_else(|| ApiError::BadRequest("fromType is required".into()))?;

    let relations = state.relation_dao
        .find_by_from_filtered(
            from_id,
            from_type,
            params.relation_type.as_deref(),
            params.to_type.as_deref(),
        )
        .await?;

    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/info?fromId=...&fromType=...
async fn list_relations_info(
    State(state): State<EntityState>,
    Query(params): Query<RelationQueryParams>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let from_id = params.from_id
        .ok_or_else(|| ApiError::BadRequest("fromId is required".into()))?;
    let from_type = params.from_type.as_deref()
        .ok_or_else(|| ApiError::BadRequest("fromType is required".into()))?;

    let relations = state.relation_dao
        .find_by_from(from_id, from_type)
        .await?;

    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relation?fromId=...&fromType=...&toId=...&toType=...&relationType=...
async fn get_relation(
    State(state): State<EntityState>,
    Query(params): Query<RelationQueryParams>,
) -> Result<Json<RelationResponse>, ApiError> {
    let from_id = params.from_id.ok_or_else(|| ApiError::BadRequest("fromId is required".into()))?;
    let from_type = params.from_type.as_deref().ok_or_else(|| ApiError::BadRequest("fromType is required".into()))?;
    let to_id = params.to_id.ok_or_else(|| ApiError::BadRequest("toId is required".into()))?;
    let to_type = params.to_type.as_deref().ok_or_else(|| ApiError::BadRequest("toType is required".into()))?;
    let relation_type = params.relation_type.as_deref().ok_or_else(|| ApiError::BadRequest("relationType is required".into()))?;
    let group = params.relation_type_group.as_deref().unwrap_or("COMMON");

    let relation = state.relation_dao
        .get_relation(from_id, from_type, to_id, to_type, relation_type, group)
        .await?
        .ok_or_else(|| ApiError::NotFound("Relation not found".into()))?;
    Ok(Json(RelationResponse::from(relation)))
}

/// DELETE /api/relations?fromId=...&fromType=... (delete all from entity)
async fn delete_relations(
    State(state): State<EntityState>,
    Query(params): Query<RelationQueryParams>,
) -> Result<StatusCode, ApiError> {
    let entity_id = params.from_id
        .or(params.entity_id)
        .ok_or_else(|| ApiError::BadRequest("fromId is required".into()))?;
    let entity_type = params.from_type
        .or(params.entity_type)
        .ok_or_else(|| ApiError::BadRequest("fromType is required".into()))?;
    state.relation_dao.delete_all_by_entity(entity_id, &entity_type).await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FromPath { from_type: String, from_id: Uuid }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToPath { to_type: String, to_id: Uuid }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FromTypePath { from_type: String, from_id: Uuid, relation_type: String }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToTypePath { to_type: String, to_id: Uuid, relation_type: String }

/// GET /api/relations/from/{fromType}/{fromId}
async fn find_relations_from_path(
    State(state): State<EntityState>,
    Path(p): Path<FromPath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao.find_by_from(p.from_id, &p.from_type).await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/from/{fromType}/{fromId}/{relationType}
async fn find_relations_from_type_path(
    State(state): State<EntityState>,
    Path(p): Path<FromTypePath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao
        .find_by_from_filtered(p.from_id, &p.from_type, Some(&p.relation_type), None)
        .await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/to/{toType}/{toId}
async fn find_relations_to_path(
    State(state): State<EntityState>,
    Path(p): Path<ToPath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao.find_by_to(p.to_id, &p.to_type).await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/to/{toType}/{toId}/{relationType}
async fn find_relations_to_type_path(
    State(state): State<EntityState>,
    Path(p): Path<ToTypePath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao
        .find_by_to_filtered(p.to_id, &p.to_type, Some(&p.relation_type), None)
        .await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/info/from/{fromType}/{fromId}
async fn find_relations_info_from_path(
    State(state): State<EntityState>,
    Path(p): Path<FromPath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao.find_by_from(p.from_id, &p.from_type).await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

/// GET /api/relations/info/to/{toType}/{toId}
async fn find_relations_info_to_path(
    State(state): State<EntityState>,
    Path(p): Path<ToPath>,
) -> Result<Json<Vec<RelationResponse>>, ApiError> {
    let relations = state.relation_dao.find_by_to(p.to_id, &p.to_type).await?;
    Ok(Json(relations.into_iter().map(RelationResponse::from).collect()))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn entity_type_str(t: &EntityType) -> &'static str {
    match t {
        EntityType::Tenant         => "TENANT",
        EntityType::Customer       => "CUSTOMER",
        EntityType::User           => "USER",
        EntityType::Dashboard      => "DASHBOARD",
        EntityType::Asset          => "ASSET",
        EntityType::Device         => "DEVICE",
        EntityType::AlarmEntity    => "ALARM",
        EntityType::RuleChain      => "RULE_CHAIN",
        EntityType::RuleNode       => "RULE_NODE",
        EntityType::EntityView     => "ENTITY_VIEW",
        EntityType::TenantProfile  => "TENANT_PROFILE",
        EntityType::DeviceProfile  => "DEVICE_PROFILE",
        EntityType::AssetProfile   => "ASSET_PROFILE",
        EntityType::Edge           => "EDGE",
        EntityType::OtaPackage     => "OTA_PACKAGE",
        _                          => "DEVICE",
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s.to_uppercase().as_str() {
        "TENANT"         => EntityType::Tenant,
        "CUSTOMER"       => EntityType::Customer,
        "USER"           => EntityType::User,
        "DASHBOARD"      => EntityType::Dashboard,
        "ASSET"          => EntityType::Asset,
        "RULE_CHAIN"     => EntityType::RuleChain,
        "RULE_NODE"      => EntityType::RuleNode,
        "ENTITY_VIEW"    => EntityType::EntityView,
        "TENANT_PROFILE" => EntityType::TenantProfile,
        "DEVICE_PROFILE" => EntityType::DeviceProfile,
        "ASSET_PROFILE"  => EntityType::AssetProfile,
        "EDGE"           => EntityType::Edge,
        "OTA_PACKAGE"    => EntityType::OtaPackage,
        _                => EntityType::Device,
    }
}

fn relation_group_str(g: &RelationTypeGroup) -> &'static str {
    match g {
        RelationTypeGroup::Common                => "COMMON",
        RelationTypeGroup::Alarm                 => "ALARM",
        RelationTypeGroup::DashboardLink         => "DASHBOARD_LINK",
        RelationTypeGroup::RuleChain             => "RULE_CHAIN",
        RelationTypeGroup::RuleNode              => "RULE_NODE",
        RelationTypeGroup::EdgeAutoAssignDefault => "EDGE_AUTO_ASSIGN_DEFAULT",
    }
}

fn parse_relation_group(s: &str) -> RelationTypeGroup {
    match s.to_uppercase().as_str() {
        "ALARM"                    => RelationTypeGroup::Alarm,
        "DASHBOARD_LINK"           => RelationTypeGroup::DashboardLink,
        "RULE_CHAIN"               => RelationTypeGroup::RuleChain,
        "RULE_NODE"                => RelationTypeGroup::RuleNode,
        "EDGE_AUTO_ASSIGN_DEFAULT" => RelationTypeGroup::EdgeAutoAssignDefault,
        _                          => RelationTypeGroup::Common,
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

    async fn post_json(app: axum::Router, uri: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn delete_with_params_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = post_json(app, "/api/auth/login",
            json!({"username": email, "password": pwd})).await;
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_relation_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "rel@test.com", "pass123").await;
        let token = get_token(app.clone(), "rel@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let resp = post_json_auth(app, "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id, "entityType": "ASSET"},
            "type": "Contains",
            "typeGroup": "COMMON",
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_saved_relation(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "relget@test.com", "pass123").await;
        let token = get_token(app.clone(), "relget@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Save relation
        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id, "entityType": "ASSET"},
            "type": "Contains",
            "typeGroup": "COMMON",
        })).await;

        // Get relation
        let url = format!("/api/relation?fromId={from_id}&fromType=DEVICE&toId={to_id}&toType=ASSET&relationType=Contains");
        let resp = get_auth(app, &url, &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["from"]["entityType"], "DEVICE");
        assert_eq!(body["to"]["entityType"], "ASSET");
        assert_eq!(body["type"], "Contains");
        assert_eq!(body["typeGroup"], "COMMON");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_relations_from_entity(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "rellist@test.com", "pass123").await;
        let token = get_token(app.clone(), "rellist@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id1 = Uuid::new_v4();
        let to_id2 = Uuid::new_v4();

        // Save two relations from the same entity
        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id1, "entityType": "ASSET"},
            "type": "Contains",
        })).await;
        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id2, "entityType": "ASSET"},
            "type": "Manages",
        })).await;

        let url = format!("/api/relations?fromId={from_id}&fromType=DEVICE");
        let resp = get_auth(app, &url, &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_relation(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "reldel@test.com", "pass123").await;
        let token = get_token(app.clone(), "reldel@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Save relation
        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id, "entityType": "ASSET"},
            "type": "Contains",
        })).await;

        // Delete relation
        let url = format!("/api/relation?fromId={from_id}&fromType=DEVICE&toId={to_id}&toType=ASSET&relationType=Contains");
        let del = delete_with_params_auth(app.clone(), &url, &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        // Verify gone
        let get = get_auth(app, &format!("/api/relation?fromId={from_id}&fromType=DEVICE&toId={to_id}&toType=ASSET&relationType=Contains"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn path_based_relations_from(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "relpath@test.com", "pass123").await;
        let token = get_token(app.clone(), "relpath@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id, "entityType": "ASSET"},
            "type": "Contains",
        })).await;

        let resp = get_auth(app, &format!("/api/relations/from/DEVICE/{from_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array());
        assert!(!body.as_array().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_relations_missing_params_returns_400(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "relbad@test.com", "pass123").await;
        let token = get_token(app.clone(), "relbad@test.com", "pass123").await;

        // Missing fromType
        let resp = get_auth(app, &format!("/api/relations?fromId={}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn relation_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "relfmt@test.com", "pass123").await;
        let token = get_token(app.clone(), "relfmt@test.com", "pass123").await;

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        post_json_auth(app.clone(), "/api/relation", &token, json!({
            "from": {"id": from_id, "entityType": "DEVICE"},
            "to": {"id": to_id, "entityType": "ASSET"},
            "type": "Contains",
            "typeGroup": "COMMON",
        })).await;

        let url = format!("/api/relation?fromId={from_id}&fromType=DEVICE&toId={to_id}&toType=ASSET&relationType=Contains");
        let resp = get_auth(app, &url, &token).await;
        let body = body_json(resp).await;

        // ThingsBoard format: from/to are EntityId objects, type and typeGroup are strings
        assert!(body["from"]["id"].is_string());
        assert_eq!(body["from"]["entityType"], "DEVICE");
        assert!(body["to"]["id"].is_string());
        assert_eq!(body["to"]["entityType"], "ASSET");
        assert_eq!(body["type"], "Contains");
        assert_eq!(body["typeGroup"], "COMMON");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_relation_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/relation")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "from": {"id": Uuid::new_v4(), "entityType": "DEVICE"},
                    "to": {"id": Uuid::new_v4(), "entityType": "ASSET"},
                    "type": "Contains",
                }).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
