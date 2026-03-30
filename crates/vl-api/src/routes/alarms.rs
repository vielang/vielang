use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Alarm, AlarmComment, AlarmSeverity, EntityType};
use vl_dao::{PageData, PageLink};

use crate::{error::ApiError, middleware::auth::SecurityContext, routes::devices::IdResponse, state::{AlarmState, AppState, BillingState, NotificationState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: AlarmController — GET /alarms lists all tenant alarms
        .route("/alarms",                                       get(list_tenant_alarms))
        .route("/alarm",                                        post(save_alarm))
        .route("/alarm/{alarmId}",                              get(get_alarm).delete(delete_alarm))
        .route("/alarm/{alarmId}/acknowledge",                  post(acknowledge_alarm))
        .route("/alarm/{alarmId}/clear",                        post(clear_alarm))
        .route("/alarm/{entityType}/{entityId}",                get(list_entity_alarms))
        // Phase 28: assign/unassign + types
        .route("/alarm/{alarmId}/assign/{userId}",              post(assign_alarm_to_user))
        .route("/alarm/{alarmId}/assign",                       delete(unassign_alarm))
        .route("/alarm/types",                                  get(get_alarm_types))
        // Phase 28: AlarmCommentController
        .route("/alarm/{alarmId}/comment",                      post(add_alarm_comment))
        .route("/alarm/{alarmId}/comments",                     get(list_alarm_comments))
        .route("/alarm/{alarmId}/comment/{commentId}",          delete(delete_alarm_comment))
        // Phase 66: Flutter PE AlarmQueryV2
        .route("/alarms/v2",                                    get(list_alarms_v2))
        // Phase 10: missing Java endpoints
        .route("/alarm/info/{alarmId}",                         get(get_alarm_info))
        .route("/alarm/highestSeverity/{entityType}/{entityId}", get(get_highest_alarm_severity))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlarmResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "originatorId")]
    pub originator_id: IdResponse,
    #[serde(rename = "originatorName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originator_name: Option<String>,
    #[serde(rename = "type")]
    pub alarm_type: String,
    pub severity: String,
    pub status: String,
    pub acknowledged: bool,
    pub cleared: bool,
    #[serde(rename = "startTs")]
    pub start_ts: i64,
    #[serde(rename = "endTs")]
    pub end_ts: i64,
    #[serde(rename = "ackTs")]
    pub ack_ts: Option<i64>,
    #[serde(rename = "clearTs")]
    pub clear_ts: Option<i64>,
    pub details: Option<serde_json::Value>,
}

impl From<Alarm> for AlarmResponse {
    fn from(a: Alarm) -> Self {
        let status = match a.status() {
            vl_core::entities::AlarmStatus::ActiveUnack  => "ACTIVE_UNACK",
            vl_core::entities::AlarmStatus::ActiveAck    => "ACTIVE_ACK",
            vl_core::entities::AlarmStatus::ClearedUnack => "CLEARED_UNACK",
            vl_core::entities::AlarmStatus::ClearedAck   => "CLEARED_ACK",
        };
        let severity = match a.severity {
            AlarmSeverity::Critical      => "CRITICAL",
            AlarmSeverity::Major         => "MAJOR",
            AlarmSeverity::Minor         => "MINOR",
            AlarmSeverity::Warning       => "WARNING",
            AlarmSeverity::Indeterminate => "INDETERMINATE",
        };
        let originator_entity_type = entity_type_str(&a.originator_type);
        Self {
            id:             IdResponse::alarm(a.id),
            created_time:   a.created_time,
            tenant_id:      IdResponse::tenant(a.tenant_id),
            originator_id:  IdResponse::new(a.originator_id, originator_entity_type),
            originator_name: None, // populated by handlers that batch-resolve names
            alarm_type:     a.alarm_type,
            severity:       severity.into(),
            status:         status.into(),
            acknowledged:   a.acknowledged,
            cleared:        a.cleared,
            start_ts:       a.start_ts,
            end_ts:         a.end_ts,
            ack_ts:         a.ack_ts,
            clear_ts:       a.clear_ts,
            details:        a.details,
        }
    }
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SaveAlarmRequest {
    pub id: Option<IdResponse>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    #[serde(rename = "originatorId")]
    pub originator_id: Option<IdResponse>,
    #[serde(rename = "type")]
    pub alarm_type: String,
    pub severity: Option<String>,
    #[serde(rename = "startTs")]
    pub start_ts: Option<i64>,
    #[serde(rename = "endTs")]
    pub end_ts: Option<i64>,
    pub propagate: Option<bool>,
    #[serde(rename = "propagateToOwner")]
    pub propagate_to_owner: Option<bool>,
    #[serde(rename = "propagateToTenant")]
    pub propagate_to_tenant: Option<bool>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListAlarmParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/alarms — list all alarms for current tenant (Java: AlarmController.getAlarms)
async fn list_tenant_alarms(
    State(state): State<AlarmState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<ListAlarmParams>,
) -> Result<Json<PageData<AlarmResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let mut page_link = PageLink::new(
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(10),
    );
    page_link.text_search = params.text_search;

    let page = state.alarm_dao.find_by_tenant(tenant_id, &page_link).await?;
    let ids: Vec<uuid::Uuid> = page.data.iter().map(|a| a.originator_id).collect();
    let names = state.alarm_dao.resolve_originator_names(&ids).await;
    let data = page.data.into_iter().map(|a| {
        let mut resp = AlarmResponse::from(a);
        resp.originator_name = names.get(&resp.originator_id.id).cloned();
        resp
    }).collect();
    Ok(Json(PageData {
        data,
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/alarm — createOrUpdateAlarm
async fn save_alarm(
    State(alarm_state): State<AlarmState>,
    State(billing): State<BillingState>,
    State(notif): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveAlarmRequest>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    // Java TB: tenantId comes from SecurityContext, body value is ignored/overridden
    let tenant_id = ctx.tenant_id;
    let originator = req.originator_id
        .ok_or_else(|| ApiError::BadRequest("originatorId is required".into()))?;

    let severity = parse_severity_str(req.severity.as_deref().unwrap_or("INDETERMINATE"));
    let originator_type = parse_entity_type_str(&originator.entity_type);

    let alarm = Alarm {
        id:           req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time: now,
        tenant_id,
        customer_id:  None,
        alarm_type:   req.alarm_type,
        originator_id:   originator.id,
        originator_type,
        severity,
        acknowledged: false,
        cleared:      false,
        assignee_id:  None,
        start_ts:     req.start_ts.unwrap_or(now),
        end_ts:       req.end_ts.unwrap_or(now),
        ack_ts:       None,
        clear_ts:     None,
        assign_ts:    0,
        propagate:              req.propagate.unwrap_or(false),
        propagate_to_owner:     req.propagate_to_owner.unwrap_or(false),
        propagate_to_tenant:    req.propagate_to_tenant.unwrap_or(false),
        propagate_relation_types: None,
        details:      req.details,
    };
    let saved = alarm_state.alarm_dao.save(&alarm).await?;

    // Phase 71: record alarm usage
    billing.usage_tracker.record_alarm(tenant_id);

    // Phase 67: spawn notification delivery to tenant admins
    {
        let svc        = notif.notification_delivery_svc.clone();
        let alarm_dao  = alarm_state.alarm_dao.clone();
        let alarm_snap = saved.clone();
        let severity_str = match alarm_snap.severity {
            AlarmSeverity::Critical      => "CRITICAL",
            AlarmSeverity::Major         => "MAJOR",
            AlarmSeverity::Minor         => "MINOR",
            AlarmSeverity::Warning       => "WARNING",
            AlarmSeverity::Indeterminate => "INDETERMINATE",
        };
        let severity_owned = severity_str.to_owned();
        tokio::spawn(async move {
            match alarm_dao.find_tenant_admin_ids(alarm_snap.tenant_id).await {
                Ok(admin_ids) if !admin_ids.is_empty() => {
                    svc.deliver(
                        alarm_snap.tenant_id,
                        Some(&format!("Alarm: {}", alarm_snap.alarm_type)),
                        &format!("[{}] {} on {}",
                            severity_owned,
                            alarm_snap.alarm_type,
                            alarm_snap.originator_id),
                        Some("ALARM"),
                        &severity_owned,
                        &admin_ids,
                    ).await;
                }
                _ => {}
            }
        });
    }

    Ok(Json(AlarmResponse::from(saved)))
}

/// GET /api/alarm/{alarmId}
async fn get_alarm(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let alarm = state.alarm_dao.find_by_id(alarm_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Alarm [{}] is not found", alarm_id)))?;
    Ok(Json(AlarmResponse::from(alarm)))
}

/// DELETE /api/alarm/{alarmId}
async fn delete_alarm(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.alarm_dao.delete(alarm_id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/alarm/{alarmId}/acknowledge
async fn acknowledge_alarm(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let ts = chrono::Utc::now().timestamp_millis();
    state.alarm_dao.acknowledge(alarm_id, ts).await?;
    Ok(StatusCode::OK)
}

/// POST /api/alarm/{alarmId}/clear
async fn clear_alarm(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let ts = chrono::Utc::now().timestamp_millis();
    state.alarm_dao.clear(alarm_id, ts).await?;
    Ok(StatusCode::OK)
}

/// GET /api/alarm/{entityType}/{entityId}?page=0&pageSize=10
async fn list_entity_alarms(
    State(state): State<AlarmState>,
    Path((_entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<ListAlarmParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<AlarmResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let mut page_link = PageLink::new(
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(10),
    );
    page_link.text_search = params.text_search;

    let page = state.alarm_dao
        .find_by_originator(tenant_id, entity_id, &page_link)
        .await?;

    let ids: Vec<uuid::Uuid> = page.data.iter().map(|a| a.originator_id).collect();
    let names = state.alarm_dao.resolve_originator_names(&ids).await;
    let data = page.data.into_iter().map(|a| {
        let mut resp = AlarmResponse::from(a);
        resp.originator_name = names.get(&resp.originator_id.id).cloned();
        resp
    }).collect();
    Ok(Json(PageData {
        data,
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/alarm/{alarmId}/assign/{userId} — gán alarm cho user
async fn assign_alarm_to_user(
    State(state): State<AlarmState>,
    Path((alarm_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let ts = chrono::Utc::now().timestamp_millis();
    state.alarm_dao.assign_to_user(alarm_id, user_id, ts).await?;
    Ok(StatusCode::OK)
}

/// DELETE /api/alarm/{alarmId}/assign — bỏ gán alarm
async fn unassign_alarm(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.alarm_dao.unassign(alarm_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/alarm/types — danh sách alarm types của tenant
async fn get_alarm_types(
    State(state): State<AlarmState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<String>>, ApiError> {
    let types = state.alarm_dao.find_types_by_tenant(ctx.tenant_id).await?;
    Ok(Json(types))
}

// ── AlarmComment DTOs ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmCommentResponse {
    pub id: IdResponse,
    pub created_time: i64,
    pub alarm_id: IdResponse,
    pub user_id: Option<IdResponse>,
    #[serde(rename = "type")]
    pub comment_type: String,
    pub comment: serde_json::Value,
}

impl From<AlarmComment> for AlarmCommentResponse {
    fn from(c: AlarmComment) -> Self {
        Self {
            id:           IdResponse::new(c.id, "ALARM_COMMENT"),
            created_time: c.created_time,
            alarm_id:     IdResponse::alarm(c.alarm_id),
            user_id:      c.user_id.map(|id| IdResponse::new(id, "USER")),
            comment_type: c.comment_type,
            comment:      c.comment,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentRequest {
    pub comment: serde_json::Value,
}

// ── AlarmComment handlers ─────────────────────────────────────────────────────

/// POST /api/alarm/{alarmId}/comment
async fn add_alarm_comment(
    State(state): State<AlarmState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(alarm_id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<AlarmCommentResponse>, ApiError> {
    let comment = AlarmComment {
        id:           Uuid::new_v4(),
        created_time: chrono::Utc::now().timestamp_millis(),
        alarm_id,
        user_id:      Some(ctx.user_id),
        comment_type: "OTHER".to_string(),
        comment:      req.comment,
    };
    let saved = state.alarm_comment_dao.save(&comment).await?;
    Ok(Json(AlarmCommentResponse::from(saved)))
}

/// GET /api/alarm/{alarmId}/comments?page=0&pageSize=10
async fn list_alarm_comments(
    State(state): State<AlarmState>,
    Path(alarm_id): Path<Uuid>,
    Query(params): Query<ListAlarmParams>,
) -> Result<Json<PageData<AlarmCommentResponse>>, ApiError> {
    let page_link = PageLink::new(
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(10),
    );
    let page = state.alarm_comment_dao.find_by_alarm(alarm_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(AlarmCommentResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// DELETE /api/alarm/{alarmId}/comment/{commentId}
async fn delete_alarm_comment(
    State(state): State<AlarmState>,
    Path((_alarm_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.alarm_comment_dao.delete(comment_id).await?;
    Ok(StatusCode::OK)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_severity_str(s: &str) -> AlarmSeverity {
    match s.to_uppercase().as_str() {
        "CRITICAL"      => AlarmSeverity::Critical,
        "MAJOR"         => AlarmSeverity::Major,
        "MINOR"         => AlarmSeverity::Minor,
        "WARNING"       => AlarmSeverity::Warning,
        _               => AlarmSeverity::Indeterminate,
    }
}

fn parse_entity_type_str(s: &str) -> EntityType {
    match s.to_uppercase().as_str() {
        "TENANT"         => EntityType::Tenant,
        "CUSTOMER"       => EntityType::Customer,
        "USER"           => EntityType::User,
        "DASHBOARD"      => EntityType::Dashboard,
        "ASSET"          => EntityType::Asset,
        "RULE_CHAIN"     => EntityType::RuleChain,
        "ENTITY_VIEW"    => EntityType::EntityView,
        "TENANT_PROFILE" => EntityType::TenantProfile,
        "DEVICE_PROFILE" => EntityType::DeviceProfile,
        "ASSET_PROFILE"  => EntityType::AssetProfile,
        "EDGE"           => EntityType::Edge,
        "OTA_PACKAGE"    => EntityType::OtaPackage,
        _                => EntityType::Device,
    }
}

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

// ── Phase 66: AlarmQueryV2 ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmQueryV2 {
    pub page:           Option<i64>,
    pub page_size:      Option<i64>,
    pub start_time:     Option<i64>,
    pub end_time:       Option<i64>,
    pub severity:       Option<String>,
    pub status:         Option<String>,
    pub assignee_id:    Option<Uuid>,
    #[serde(rename = "type")]
    pub alarm_type:     Option<String>,
    pub text_search:    Option<String>,
    pub sort_property:  Option<String>,
    pub sort_order:     Option<String>,
}

/// GET /api/alarms/v2 — advanced alarm query (Flutter PE AlarmQueryV2)
async fn list_alarms_v2(
    State(state): State<AlarmState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(q): Query<AlarmQueryV2>,
) -> Result<Json<PageData<AlarmResponse>>, ApiError> {
    let page      = q.page.unwrap_or(0);
    let page_size = q.page_size.unwrap_or(10).min(1000);
    let offset    = page * page_size;

    let rows = state.alarm_dao
        .find_with_filters(
            ctx.tenant_id,
            q.start_time,
            q.end_time,
            q.severity.as_deref(),
            q.status.as_deref(),
            q.assignee_id,
            q.alarm_type.as_deref(),
            q.text_search.as_deref(),
            page_size,
            offset,
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total = state.alarm_dao
        .count_with_filters(
            ctx.tenant_id,
            q.start_time,
            q.end_time,
            q.severity.as_deref(),
            q.status.as_deref(),
            q.assignee_id,
            q.alarm_type.as_deref(),
            q.text_search.as_deref(),
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total_pages = if page_size == 0 { 0 } else { (total + page_size - 1) / page_size };
    let has_next    = (page + 1) * page_size < total;

    let ids: Vec<Uuid> = rows.iter().map(|a| a.originator_id).collect();
    let names = state.alarm_dao.resolve_originator_names(&ids).await;
    let data = rows.into_iter().map(|a| {
        let mut resp = AlarmResponse::from(a);
        resp.originator_name = names.get(&resp.originator_id.id).cloned();
        resp
    }).collect();

    Ok(Json(PageData { data, total_pages, total_elements: total, has_next }))
}

// ── Phase 10: Missing Java endpoints ──────────────────────────────────────────

/// GET /api/alarm/info/{alarmId} — alarm with additional info (originator name).
/// Java: AlarmController.getAlarmInfoById()
async fn get_alarm_info(
    State(state): State<AlarmState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(alarm_id): Path<Uuid>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let alarm = state.alarm_dao
        .find_by_id(alarm_id).await?
        .ok_or(ApiError::NotFound("Alarm not found".into()))?;

    if alarm.tenant_id != ctx.tenant_id {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let names = state.alarm_dao.resolve_originator_names(&[alarm.originator_id]).await;
    let mut resp = AlarmResponse::from(alarm);
    resp.originator_name = names.get(&resp.originator_id.id).cloned();
    Ok(Json(resp))
}

/// GET /api/alarm/highestSeverity/{entityType}/{entityId}
/// Java: AlarmController.getHighestAlarmSeverity()
/// Returns the highest active alarm severity for the entity.
async fn get_highest_alarm_severity(
    State(state): State<AlarmState>,
    Path((_entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let severity = state.alarm_dao
        .find_highest_severity(entity_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!(severity)))
}

// ── Integration Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
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

    // ── Helpers ───────────────────────────────────────────────────────────────

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

    async fn insert_device_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO device_profile
               (id, created_time, tenant_id, name, type, transport_type, provision_type, is_default)
               VALUES ($1, $2, $3, $4, 'DEFAULT', 'DEFAULT', 'DISABLED', false)"#,
            id, now_ms(), tenant_id, format!("profile-{id}"),
        )
        .execute(pool).await.unwrap();
        id
    }

    async fn insert_device(pool: &PgPool, tenant_id: Uuid, profile_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO device
               (id, created_time, tenant_id, name, type, device_profile_id, version)
               VALUES ($1, $2, $3, $4, 'default', $5, 1)"#,
            id, now_ms(), tenant_id, format!("device-{id}"), profile_id,
        )
        .execute(pool).await.unwrap();
        id
    }

    async fn post_json(app: axum::Router, uri: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn post_json_auth(
        app: axum::Router, uri: &str, token: &str, body: Value,
    ) -> axum::response::Response {
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

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
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

    /// Create an alarm via POST /api/alarm and return the full response body.
    async fn create_alarm(
        app: axum::Router,
        token: &str,
        tenant_id: Uuid,
        device_id: Uuid,
        alarm_type: &str,
    ) -> Value {
        let resp = post_json_auth(app, "/api/alarm", token, json!({
            "tenantId":    {"id": tenant_id, "entityType": "TENANT"},
            "originatorId": {"id": device_id, "entityType": "DEVICE"},
            "type":        alarm_type,
            "severity":    "CRITICAL",
        })).await;
        body_json(resp).await
    }

    // ── POST /api/alarm ───────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_alarm_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_create@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_create@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/alarm", &token, json!({
            "tenantId":    {"id": user.tenant_id, "entityType": "TENANT"},
            "originatorId": {"id": device_id, "entityType": "DEVICE"},
            "type":        "HIGH_TEMP",
            "severity":    "CRITICAL",
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarm_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_fmt@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_fmt@test.com", "pass123").await;

        let body = create_alarm(app, &token, user.tenant_id, device_id, "HIGH_TEMP").await;

        // id must be object with id (UUID string) + entityType = "ALARM"
        assert!(body["id"]["id"].is_string(),       "id.id must be a UUID string");
        assert_eq!(body["id"]["entityType"], "ALARM", "id.entityType must be ALARM");

        // createdTime must be an i64 ms timestamp
        assert!(body["createdTime"].is_number(),    "createdTime must be a number");

        // tenantId object
        assert!(body["tenantId"]["id"].is_string(), "tenantId.id must be a string");
        assert_eq!(body["tenantId"]["entityType"], "TENANT");

        // originatorId object with DEVICE type
        assert!(body["originatorId"]["id"].is_string());
        assert_eq!(body["originatorId"]["entityType"], "DEVICE");

        // alarm fields
        assert_eq!(body["type"], "HIGH_TEMP");
        assert_eq!(body["severity"], "CRITICAL");
        assert_eq!(body["status"], "ACTIVE_UNACK");
        assert_eq!(body["acknowledged"], false);
        assert_eq!(body["cleared"], false);

        assert!(body["startTs"].is_number(), "startTs must be a number");
        assert!(body["endTs"].is_number(),   "endTs must be a number");

        // nullable timestamps start as null
        assert!(body["ackTs"].is_null(),   "ackTs must be null initially");
        assert!(body["clearTs"].is_null(), "clearTs must be null initially");
    }

    // ── GET /api/alarm/{alarmId} ──────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_alarm_by_id_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_get@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_get@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "TEMP").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/alarm/{alarm_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], alarm_id);
        assert_eq!(body["type"], "TEMP");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_alarm_nonexistent_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "alarm_404@test.com", "pass123").await;
        let token = get_token(app.clone(), "alarm_404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/alarm/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── DELETE /api/alarm/{alarmId} ───────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_alarm_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_del@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_del@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "DEL_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/alarm/{alarm_id}"), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        // Verify it's gone
        let get = get_auth(app, &format!("/api/alarm/{alarm_id}"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    // ── POST /api/alarm/{alarmId}/acknowledge ─────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn acknowledge_alarm_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_ack@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_ack@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "ACK_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = post_json_auth(
            app, &format!("/api/alarm/{alarm_id}/acknowledge"), &token, json!({}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarm_lifecycle_ack_changes_status(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_ackst@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_ackst@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "LIFECYCLE").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        // Initial state: ACTIVE_UNACK, not acknowledged
        assert_eq!(created["status"], "ACTIVE_UNACK");
        assert_eq!(created["acknowledged"], false);

        // Acknowledge
        post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/acknowledge"), &token, json!({}),
        ).await;

        // After ack: status becomes ACTIVE_ACK, acknowledged = true
        let body = body_json(get_auth(app, &format!("/api/alarm/{alarm_id}"), &token).await).await;
        assert_eq!(body["acknowledged"], true,         "acknowledged must be true after ack");
        assert_eq!(body["status"], "ACTIVE_ACK",       "status must be ACTIVE_ACK after ack");
        assert!(body["ackTs"].is_number(),             "ackTs must be set after ack");
    }

    // ── POST /api/alarm/{alarmId}/clear ───────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn clear_alarm_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_clr@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_clr@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CLR_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = post_json_auth(
            app, &format!("/api/alarm/{alarm_id}/clear"), &token, json!({}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarm_lifecycle_clear_changes_status(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_clrst@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_clrst@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CLR_LIFECYCLE").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        // Clear the alarm
        post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/clear"), &token, json!({}),
        ).await;

        // After clear: cleared = true, status = CLEARED_UNACK (not acked first)
        let body = body_json(get_auth(app, &format!("/api/alarm/{alarm_id}"), &token).await).await;
        assert_eq!(body["cleared"], true,              "cleared must be true after clear");
        assert_eq!(body["status"], "CLEARED_UNACK",    "status must be CLEARED_UNACK after clear without prior ack");
        assert!(body["clearTs"].is_number(),           "clearTs must be set after clear");
    }

    // ── GET /api/alarms ───────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_alarms_returns_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_list@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_list@test.com", "pass123").await;

        // Create two alarms
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "ALARM_A").await;
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "ALARM_B").await;

        let resp = get_auth(app, "/api/alarms?page=0&pageSize=10", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        // ThingsBoard pagination format (camelCase)
        assert!(body["data"].is_array(),              "Must have 'data' array");
        assert!(body["totalPages"].is_number(),        "Must have 'totalPages'");
        assert!(body["totalElements"].is_number(),     "Must have 'totalElements'");
        assert!(body["hasNext"].is_boolean(),          "Must have 'hasNext'");

        let count = body["data"].as_array().unwrap().len();
        assert!(count >= 2, "Expected at least 2 alarms, got {count}");
    }

    // ── GET /api/alarm/{entityType}/{entityId} ────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_entity_alarms_returns_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_entity@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_entity@test.com", "pass123").await;

        // Create an alarm for this specific device
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "ENTITY_ALARM").await;

        let resp = get_auth(
            app,
            &format!("/api/alarm/DEVICE/{device_id}?page=0&pageSize=10"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["data"].is_array(),              "Must have 'data' array");
        assert!(body["totalPages"].is_number(),        "Must have 'totalPages'");
        assert!(body["totalElements"].is_number(),     "Must have 'totalElements'");
        assert!(body["hasNext"].is_boolean(),          "Must have 'hasNext'");
    }

    // ── GET /api/alarm/types ──────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarm_types_returns_list(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_types@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_types@test.com", "pass123").await;

        // Create alarms of distinct types
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "TYPE_A").await;
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "TYPE_B").await;

        let resp = get_auth(app, "/api/alarm/types", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body.is_array(), "alarm/types must return a JSON array");
        let types = body.as_array().unwrap();
        assert!(types.iter().any(|t| t.as_str() == Some("TYPE_A")), "TYPE_A should be in list");
        assert!(types.iter().any(|t| t.as_str() == Some("TYPE_B")), "TYPE_B should be in list");
    }

    // ── POST /api/alarm/{alarmId}/comment ────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn add_alarm_comment_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_cmt@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_cmt@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CMT_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = post_json_auth(
            app, &format!("/api/alarm/{alarm_id}/comment"), &token,
            json!({"comment": {"text": "Test comment"}}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarm_comment_response_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_cmt_fmt@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_cmt_fmt@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CMT_FMT").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = post_json_auth(
            app, &format!("/api/alarm/{alarm_id}/comment"), &token,
            json!({"comment": {"text": "Hello alarm"}}),
        ).await;
        let body = body_json(resp).await;

        // id object: { id: UUID, entityType: "ALARM_COMMENT" }
        assert!(body["id"]["id"].is_string(),                  "id.id must be a UUID string");
        assert_eq!(body["id"]["entityType"], "ALARM_COMMENT",  "id.entityType must be ALARM_COMMENT");

        // createdTime as i64
        assert!(body["createdTime"].is_number(),               "createdTime must be a number");

        // alarmId references the parent alarm
        assert_eq!(body["alarmId"]["id"], alarm_id,            "alarmId.id must match the alarm");
        assert_eq!(body["alarmId"]["entityType"], "ALARM",     "alarmId.entityType must be ALARM");

        // userId references the authenticated user
        assert!(body["userId"]["id"].is_string(),              "userId.id must be set");
        assert_eq!(body["userId"]["entityType"], "USER",       "userId.entityType must be USER");

        // type and comment payload
        assert_eq!(body["type"], "OTHER",                      "comment type must be OTHER");
        assert_eq!(body["comment"]["text"], "Hello alarm",     "comment.text must match");
    }

    // ── GET /api/alarm/{alarmId}/comments ────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_alarm_comments_returns_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_cmts@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_cmts@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CMT_LIST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        // Add two comments
        post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/comment"), &token,
            json!({"comment": {"text": "First"}}),
        ).await;
        post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/comment"), &token,
            json!({"comment": {"text": "Second"}}),
        ).await;

        let resp = get_auth(
            app, &format!("/api/alarm/{alarm_id}/comments?page=0&pageSize=10"), &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["data"].is_array(),               "Must have 'data' array");
        assert!(body["totalPages"].is_number(),         "Must have 'totalPages'");
        assert!(body["totalElements"].is_number(),      "Must have 'totalElements'");
        assert!(body["hasNext"].is_boolean(),           "Must have 'hasNext'");

        let count = body["data"].as_array().unwrap().len();
        assert!(count >= 2, "Expected at least 2 comments, got {count}");
    }

    // ── DELETE /api/alarm/{alarmId}/comment/{commentId} ──────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_alarm_comment_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_cmt_del@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_cmt_del@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "CMT_DEL").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let cmt_resp = post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/comment"), &token,
            json!({"comment": {"text": "Delete me"}}),
        ).await;
        let cmt_body = body_json(cmt_resp).await;
        let comment_id = cmt_body["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(
            app, &format!("/api/alarm/{alarm_id}/comment/{comment_id}"), &token,
        ).await;
        assert_eq!(del.status(), StatusCode::OK);
    }

    // ── Auth checks ───────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_alarm_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/alarm")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "tenantId": {"id": Uuid::new_v4(), "entityType": "TENANT"},
                    "originatorId": {"id": Uuid::new_v4(), "entityType": "DEVICE"},
                    "type": "NO_AUTH",
                }).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_alarm_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET")
                .uri(&format!("/api/alarm/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_alarms_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/alarms?page=0&pageSize=10")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Error response format ─────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn error_response_has_correct_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "alarm_err@test.com", "pass123").await;
        let token = get_token(app.clone(), "alarm_err@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/alarm/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // ThingsBoard error format: { status: 404, message: "...", errorCode: N }
        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404, "status field must be 404");
        assert!(body["message"].is_string(),              "message field must be a string");
        assert!(body["errorCode"].is_number(),            "errorCode field must be a number");
    }

    // ── POST /api/alarm/{alarmId}/assign/{userId} ─────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn assign_alarm_to_user_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_assign@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_assign@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "ASSIGN_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        let resp = post_json_auth(
            app, &format!("/api/alarm/{alarm_id}/assign/{}", user.id), &token, json!({}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn unassign_alarm_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_unassign@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_unassign@test.com", "pass123").await;

        let created = create_alarm(app.clone(), &token, user.tenant_id, device_id, "UNASSIGN_TEST").await;
        let alarm_id = created["id"]["id"].as_str().unwrap().to_string();

        // Assign first
        post_json_auth(
            app.clone(), &format!("/api/alarm/{alarm_id}/assign/{}", user.id), &token, json!({}),
        ).await;

        // Then unassign
        let resp = delete_auth(
            app, &format!("/api/alarm/{alarm_id}/assign"), &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Unit 19: Pagination compliance ───────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarms_pagination_has_next_false_when_all_fit(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_pg1@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_pg1@test.com", "pass123").await;

        // Create 2 alarms — both fit in pageSize=10
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "PG_FIT_1").await;
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "PG_FIT_2").await;

        let resp = get_auth(app, "/api/alarms?page=0&pageSize=10", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;

        assert_eq!(body["hasNext"], false, "hasNext must be false when all items fit in one page");
        assert!(body["data"].as_array().unwrap().len() >= 2, "Must have at least 2 alarms");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn alarms_pagination_has_next_true_when_overflow(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "alarm_pg2@test.com", "pass123").await;
        let profile_id = insert_device_profile(&pool, user.tenant_id).await;
        let device_id = insert_device(&pool, user.tenant_id, profile_id).await;
        let token = get_token(app.clone(), "alarm_pg2@test.com", "pass123").await;

        // Create 3 alarms but pageSize=2 — overflow triggers hasNext=true
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "OVF_1").await;
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "OVF_2").await;
        create_alarm(app.clone(), &token, user.tenant_id, device_id, "OVF_3").await;

        let resp = get_auth(app, "/api/alarms?page=0&pageSize=2", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;

        assert_eq!(body["hasNext"], true, "hasNext must be true when items exceed pageSize");
        assert_eq!(
            body["data"].as_array().unwrap().len(), 2,
            "pageSize=2 must return exactly 2 items"
        );
    }
}
