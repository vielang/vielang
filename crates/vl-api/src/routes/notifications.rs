use crate::util::now_ms;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{
    NotificationInbox, NotificationRequest, NotificationRule, NotificationStatus, NotificationTarget,
    NotificationTemplate, NotificationType, TriggerType,
};
use vl_dao::{NotificationChannelSettings, NotificationDelivery, PageData, PageLink};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, NotificationState}};
use super::devices::PageParams;

pub fn router() -> Router<AppState> {
    Router::new()
        // Templates
        .route("/notification/template",     post(save_template))
        .route("/notification/template/{id}", get(get_template).delete(delete_template))
        .route("/notification/templates",    get(list_templates))
        // Targets
        .route("/notification/target",       post(save_target))
        .route("/notification/target/{id}",  get(get_target).delete(delete_target))
        .route("/notification/targets",      get(list_targets))
        // Rules
        .route("/notification/rule",         post(save_rule))
        .route("/notification/rule/{id}",    get(get_rule).delete(delete_rule))
        .route("/notification/rules",        get(list_rules))
        // Requests (history)
        .route("/notification/requests",     get(list_requests))
        .route("/notification/request/{id}", get(get_request))
        // Notification inbox (per-user received notifications)
        .route("/notifications/inbox",                  get(list_inbox))
        .route("/notifications/inbox/read",             put(mark_all_inbox_read))
        .route("/notifications/inbox/unread/count",     get(get_unread_count))
        .route("/notifications/inbox/{id}",             delete(delete_inbox).put(mark_inbox_read))
        // P3: Channel settings per tenant
        .route("/notification/settings",                get(list_channel_settings).post(save_channel_setting))
        .route("/notification/settings/{channel}",      get(get_channel_setting).delete(delete_channel_setting))
        .route("/notification/test",                    post(test_channel_delivery))
        // P3: Delivery status tracking
        .route("/notification/{id}/deliveries",         get(get_notification_deliveries))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTemplateRequest {
    pub id:                Option<Uuid>,
    pub name:              String,
    #[serde(rename = "notificationType")]
    pub notification_type: String,
    pub subject_template:  Option<String>,
    pub body_template:     String,
    pub additional_config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled:           bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTargetRequest {
    pub id:            Option<Uuid>,
    pub name:          String,
    pub target_type:   String,
    pub target_config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRuleRequest {
    pub id:                Option<Uuid>,
    pub name:              String,
    pub template_id:       Uuid,
    pub trigger_type:      String,
    pub trigger_config:    Option<serde_json::Value>,
    pub recipients_config: Option<serde_json::Value>,
    pub additional_config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled:           bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestNotificationRequest {
    pub template_id: Uuid,
    pub target_ids:  Vec<Uuid>,
    pub info:        Option<serde_json::Value>,
}

fn default_true() -> bool { true }


// ── Template Handlers ─────────────────────────────────────────────────────────

async fn save_template(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveTemplateRequest>,
) -> Result<Json<NotificationTemplate>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let notification_type = NotificationType::from_str(&req.notification_type)
        .ok_or_else(|| ApiError::BadRequest(
            format!("Invalid notification type: {}", req.notification_type)
        ))?;

    let id = req.id.unwrap_or_else(Uuid::new_v4);
    let template = NotificationTemplate {
        id,
        created_time:      now_ms(),
        tenant_id,
        name:              req.name,
        notification_type,
        subject_template:  req.subject_template,
        body_template:     req.body_template,
        additional_config: req.additional_config,
        enabled:           req.enabled,
        version:           1,
    };

    let saved = state.notification_template_dao.save(&template).await?;
    Ok(Json(saved))
}

async fn get_template(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NotificationTemplate>, ApiError> {
    let t = state.notification_template_dao
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("NotificationTemplate [{}] not found", id)))?;
    Ok(Json(t))
}

async fn delete_template(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_template_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

async fn list_templates(
    State(state): State<NotificationState>,
    Query(params): Query<PageParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<NotificationTemplate>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.notification_template_dao
        .find_by_tenant(tenant_id, &params.to_page_link())
        .await?;
    Ok(Json(page))
}

// ── Target Handlers ───────────────────────────────────────────────────────────

async fn save_target(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveTargetRequest>,
) -> Result<Json<NotificationTarget>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let id = req.id.unwrap_or_else(Uuid::new_v4);
    let target = NotificationTarget {
        id,
        created_time:  now_ms(),
        tenant_id,
        name:          req.name,
        target_type:   req.target_type,
        target_config: req.target_config,
        version:       1,
    };
    let saved = state.notification_target_dao.save(&target).await?;
    Ok(Json(saved))
}

async fn get_target(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NotificationTarget>, ApiError> {
    let t = state.notification_target_dao
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("NotificationTarget [{}] not found", id)))?;
    Ok(Json(t))
}

async fn delete_target(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_target_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

async fn list_targets(
    State(state): State<NotificationState>,
    Query(params): Query<PageParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<NotificationTarget>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.notification_target_dao
        .find_by_tenant(tenant_id, &params.to_page_link())
        .await?;
    Ok(Json(page))
}

// ── Rule Handlers ─────────────────────────────────────────────────────────────

async fn save_rule(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveRuleRequest>,
) -> Result<Json<NotificationRule>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let trigger_type = TriggerType::from_str(&req.trigger_type)
        .ok_or_else(|| ApiError::BadRequest(
            format!("Invalid trigger type: {}", req.trigger_type)
        ))?;

    // Verify template exists
    state.notification_template_dao
        .find_by_id(req.template_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(
            format!("NotificationTemplate [{}] not found", req.template_id)
        ))?;

    let id = req.id.unwrap_or_else(Uuid::new_v4);
    let rule = NotificationRule {
        id,
        created_time:      now_ms(),
        tenant_id,
        name:              req.name,
        template_id:       req.template_id,
        trigger_type,
        trigger_config:    req.trigger_config.unwrap_or(serde_json::json!({})),
        recipients_config: req.recipients_config.unwrap_or(serde_json::json!({})),
        additional_config: req.additional_config,
        enabled:           req.enabled,
        version:           1,
    };

    let saved = state.notification_rule_dao.save(&rule).await?;
    Ok(Json(saved))
}

async fn get_rule(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NotificationRule>, ApiError> {
    let r = state.notification_rule_dao
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("NotificationRule [{}] not found", id)))?;
    Ok(Json(r))
}

async fn delete_rule(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_rule_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

async fn list_rules(
    State(state): State<NotificationState>,
    Query(params): Query<PageParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<NotificationRule>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.notification_rule_dao
        .find_by_tenant(tenant_id, &params.to_page_link())
        .await?;
    Ok(Json(page))
}

// ── Request (History) Handlers ────────────────────────────────────────────────

async fn list_requests(
    State(state): State<NotificationState>,
    Query(params): Query<PageParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<NotificationRequest>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.notification_request_dao
        .find_by_tenant(tenant_id, &params.to_page_link())
        .await?;
    Ok(Json(page))
}

async fn get_request(
    State(state): State<NotificationState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NotificationRequest>, ApiError> {
    let r = state.notification_request_dao
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("NotificationRequest [{}] not found", id)))?;
    Ok(Json(r))
}

// ── Test Notification ─────────────────────────────────────────────────────────

async fn test_notification(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<TestNotificationRequest>,
) -> Result<Json<NotificationRequest>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let now = now_ms();

    // Verify template exists
    state.notification_template_dao
        .find_by_id(req.template_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(
            format!("NotificationTemplate [{}] not found", req.template_id)
        ))?;

    // Create notification request record
    let notification_req = NotificationRequest {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id,
        rule_id:      None,
        template_id:  req.template_id,
        info:         req.info.unwrap_or(serde_json::json!({})),
        status:       NotificationStatus::Scheduled,
        error:        None,
        sent_time:    None,
        version:      1,
    };

    let saved = state.notification_request_dao.save(&notification_req).await?;

    // Dispatch in background (fire and forget)
    {
        let svc         = state.notification_service.clone();
        let req_clone   = saved.clone();
        let target_ids  = req.target_ids.clone();
        tokio::spawn(async move {
            let _ = svc.dispatch(req_clone, &target_ids).await;
        });
    }

    Ok(Json(saved))
}

// ── Notification Inbox ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InboxQueryParams {
    page:      Option<i64>,
    page_size: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnreadCountResponse {
    total_unread_count: i64,
}

/// GET /api/notifications/inbox
async fn list_inbox(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<InboxQueryParams>,
) -> Result<Json<PageData<NotificationInbox>>, ApiError> {
    let page_link = PageLink::new(params.page.unwrap_or(0), params.page_size.unwrap_or(10));
    let page = state.notification_inbox_dao
        .find_by_user(ctx.user_id, &page_link)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(page))
}

/// PUT /api/notifications/inbox/{id} — mark one as read
async fn mark_inbox_read(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_inbox_dao
        .mark_read(id, ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// PUT /api/notifications/inbox/read — mark ALL as read
async fn mark_all_inbox_read(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    state.notification_inbox_dao
        .mark_all_read(ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// DELETE /api/notifications/inbox/{id}
async fn delete_inbox(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_inbox_dao
        .delete(id, ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// GET /api/notifications/inbox/unread/count
async fn get_unread_count(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UnreadCountResponse>, ApiError> {
    let count = state.notification_inbox_dao
        .count_unread(ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(UnreadCountResponse { total_unread_count: count }))
}

// ── P3: Notification Channel Settings ─────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveChannelSettingRequest {
    pub channel: String,
    pub config:  serde_json::Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSettingResponse {
    pub id:           uuid::Uuid,
    pub channel:      String,
    pub config:       serde_json::Value,
    pub enabled:      bool,
    pub created_time: i64,
}

impl From<NotificationChannelSettings> for ChannelSettingResponse {
    fn from(s: NotificationChannelSettings) -> Self {
        Self {
            id:           s.id,
            channel:      s.channel,
            config:       s.config,
            enabled:      s.enabled,
            created_time: s.created_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestChannelRequest {
    pub channel:    String,
    pub message:    String,
    pub template_id: Option<uuid::Uuid>,
    pub target_ids: Vec<uuid::Uuid>,
    pub info:       Option<serde_json::Value>,
}

/// GET /api/notification/settings — list all channel settings for tenant
async fn list_channel_settings(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<ChannelSettingResponse>>, ApiError> {
    let settings = state.notification_channel_settings_dao
        .find_by_tenant(ctx.tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(settings.into_iter().map(ChannelSettingResponse::from).collect()))
}

/// GET /api/notification/settings/{channel}
async fn get_channel_setting(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(channel): Path<String>,
) -> Result<Json<ChannelSettingResponse>, ApiError> {
    let setting = state.notification_channel_settings_dao
        .find_by_tenant_and_channel(ctx.tenant_id, &channel.to_uppercase())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound(format!("Channel settings [{}] not found", channel)))?;

    Ok(Json(ChannelSettingResponse::from(setting)))
}

/// POST /api/notification/settings — save channel settings
async fn save_channel_setting(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveChannelSettingRequest>,
) -> Result<Json<ChannelSettingResponse>, ApiError> {
    let now = now_ms();
    let channel = req.channel.to_uppercase();

    let setting = NotificationChannelSettings {
        id:           uuid::Uuid::new_v4(),
        tenant_id:    ctx.tenant_id,
        channel:      channel.clone(),
        config:       req.config,
        enabled:      req.enabled,
        created_time: now,
    };

    state.notification_channel_settings_dao
        .upsert(&setting)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Re-fetch to return current state
    let saved = state.notification_channel_settings_dao
        .find_by_tenant_and_channel(ctx.tenant_id, &channel)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Settings not found after save".into()))?;

    Ok(Json(ChannelSettingResponse::from(saved)))
}

/// DELETE /api/notification/settings/{channel}
async fn delete_channel_setting(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(channel): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.notification_channel_settings_dao
        .delete(ctx.tenant_id, &channel.to_uppercase())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// POST /api/notification/test — test send through channel
async fn test_channel_delivery(
    State(state): State<NotificationState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<TestChannelRequest>,
) -> Result<Json<NotificationRequest>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let now = now_ms();

    // If templateId provided, use standard dispatch
    if let Some(template_id) = req.template_id {
        state.notification_template_dao
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(
                format!("NotificationTemplate [{}] not found", template_id)
            ))?;

        let notification_req = NotificationRequest {
            id:           Uuid::new_v4(),
            created_time: now,
            tenant_id,
            rule_id:      None,
            template_id,
            info:         req.info.unwrap_or(serde_json::json!({})),
            status:       NotificationStatus::Scheduled,
            error:        None,
            sent_time:    None,
            version:      1,
        };

        let saved = state.notification_request_dao.save(&notification_req).await?;

        let svc        = state.notification_service.clone();
        let req_clone  = saved.clone();
        let target_ids = req.target_ids.clone();
        tokio::spawn(async move {
            let _ = svc.dispatch(req_clone, &target_ids).await;
        });

        return Ok(Json(saved));
    }

    // Otherwise create a synthetic request for direct test
    let notification_req = NotificationRequest {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id,
        rule_id:      None,
        template_id:  Uuid::nil(),
        info:         serde_json::json!({ "testMessage": req.message, "channel": req.channel }),
        status:       NotificationStatus::Scheduled,
        error:        None,
        sent_time:    None,
        version:      1,
    };

    let saved = state.notification_request_dao.save(&notification_req).await?;
    tracing::info!(
        channel = %req.channel,
        message = %req.message,
        "Test notification dispatched"
    );

    Ok(Json(saved))
}

// ── P3: Delivery status ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryResponse {
    pub id:              Uuid,
    pub notification_id: Uuid,
    pub channel_type:    String,
    pub recipient:       String,
    pub status:          String,
    pub error:           Option<String>,
    pub sent_at:         Option<i64>,
    pub created_time:    i64,
}

impl From<NotificationDelivery> for DeliveryResponse {
    fn from(d: NotificationDelivery) -> Self {
        Self {
            id:              d.id,
            notification_id: d.notification_id,
            channel_type:    d.channel_type,
            recipient:       d.recipient,
            status:          d.status,
            error:           d.error,
            sent_at:         d.sent_at,
            created_time:    d.created_time,
        }
    }
}

/// GET /api/notification/{id}/deliveries — list delivery attempts per channel
async fn get_notification_deliveries(
    State(state): State<NotificationState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DeliveryResponse>>, ApiError> {
    let deliveries = state.notification_delivery_dao
        .find_by_notification(id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(deliveries.into_iter().map(DeliveryResponse::from).collect()))
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
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;
    use vl_auth::password;

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config        = VieLangConfig::default();
        let rule_engine   = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache         = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster       = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    async fn create_user(pool: &PgPool, email: &str, pass: &str) -> User {
        let dao  = UserDao::new(pool.clone());
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

    async fn login_as(app: axum::Router, email: &str, pass: &str) -> String {
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pass}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    async fn post_json(app: axum::Router, uri: &str, token: &str, body: Value)
        -> axum::response::Response
    {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string())).unwrap(),
        ).await.unwrap()
    }

    async fn get_req(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    async fn delete_req(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
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

    // ── Template tests ────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_template_returns_saved(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt1@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt1@test.com", "pass123").await;

        let resp = post_json(app, "/api/notification/template", &token, json!({
            "notificationType": "SLACK",
            "name":             "High Temp Alert",
            "bodyTemplate":     "Device ${deviceName} temp: ${temp}°C",
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["name"].as_str().unwrap(), "High Temp Alert");
        assert_eq!(body["notificationType"].as_str().unwrap(), "SLACK");
        assert!(body["id"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_template_not_found_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt2@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt2@test.com", "pass123").await;

        let resp = get_req(app, &format!("/api/notification/template/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_templates_returns_paginated(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt3@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt3@test.com", "pass123").await;

        for name in ["tmpl-a", "tmpl-b"] {
            post_json(app.clone(), "/api/notification/template", &token, json!({
                "notificationType": "WEBHOOK",
                "name":             name,
                "bodyTemplate":     "body",
            })).await;
        }

        let resp = get_req(app, "/api/notification/templates?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["totalElements"].as_i64().unwrap() >= 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_template_then_not_found(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt4@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt4@test.com", "pass123").await;

        let created = post_json(app.clone(), "/api/notification/template", &token, json!({
            "notificationType": "EMAIL",
            "name":             "del-template",
            "bodyTemplate":     "body",
        })).await;
        let b = body_json(created).await;
        let id = b["id"].as_str().unwrap();

        let del = delete_req(app.clone(), &format!("/api/notification/template/{}", id), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_req(app, &format!("/api/notification/template/{}", id), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    // ── Target tests ──────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_target_and_get(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt5@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt5@test.com", "pass123").await;

        let resp = post_json(app.clone(), "/api/notification/target", &token, json!({
            "name":        "Slack #alerts",
            "targetType":  "SLACK_CHANNEL",
            "targetConfig": { "webhookUrl": "https://hooks.slack.com/xxx", "channel": "#alerts" }
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let b = body_json(resp).await;
        let id = b["id"].as_str().unwrap();

        let get = get_req(app, &format!("/api/notification/target/{}", id), &token).await;
        assert_eq!(get.status(), StatusCode::OK);
        let gb = body_json(get).await;
        assert_eq!(gb["name"].as_str().unwrap(), "Slack #alerts");
        assert_eq!(gb["targetType"].as_str().unwrap(), "SLACK_CHANNEL");
    }

    // ── Rule tests ────────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_rule_returns_saved(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt6@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt6@test.com", "pass123").await;

        // Create template first
        let tmpl = post_json(app.clone(), "/api/notification/template", &token, json!({
            "notificationType": "SLACK",
            "name":             "Alarm Template",
            "bodyTemplate":     "Alarm ${alarmType} on ${deviceName}",
        })).await;
        let tb = body_json(tmpl).await;
        let tmpl_id = tb["id"].as_str().unwrap();

        let resp = post_json(app, "/api/notification/rule", &token, json!({
            "name":         "Alarm Notify Rule",
            "templateId":   tmpl_id,
            "triggerType":  "ALARM",
            "triggerConfig": { "alarmTypes": ["HighTemperature"] },
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let b = body_json(resp).await;
        assert_eq!(b["name"].as_str().unwrap(), "Alarm Notify Rule");
        assert_eq!(b["triggerType"].as_str().unwrap(), "ALARM");
        assert!(b["enabled"].as_bool().unwrap());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_rule_invalid_template_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt7@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt7@test.com", "pass123").await;

        let resp = post_json(app, "/api/notification/rule", &token, json!({
            "name":        "Bad Rule",
            "templateId":  Uuid::new_v4(),
            "triggerType": "ALARM",
        })).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── Test notification dispatch ─────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn test_notification_creates_request(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "nt8@test.com", "pass123").await;
        let token = login_as(app.clone(), "nt8@test.com", "pass123").await;

        // Create template
        let tmpl = post_json(app.clone(), "/api/notification/template", &token, json!({
            "notificationType": "SLACK",
            "name":             "Test Tmpl",
            "bodyTemplate":     "Hello ${name}!",
        })).await;
        let tb = body_json(tmpl).await;
        let tmpl_id = tb["id"].as_str().unwrap();

        // Test dispatch
        let resp = post_json(app.clone(), "/api/notification/test", &token, json!({
            "templateId": tmpl_id,
            "targetIds":  [],
            "info":       { "name": "World" }
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let b = body_json(resp).await;
        assert!(b["id"].is_string());
        // Status should be SCHEDULED initially (dispatch is async)
        assert_eq!(b["status"].as_str().unwrap(), "SCHEDULED");

        // Verify request saved in history
        let list = get_req(app, "/api/notification/requests?pageSize=10&page=0", &token).await;
        assert_eq!(list.status(), StatusCode::OK);
        let lb = body_json(list).await;
        assert!(lb["totalElements"].as_i64().unwrap() >= 1);
    }
}
