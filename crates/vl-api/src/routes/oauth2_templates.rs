use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use uuid::Uuid;

use vl_core::entities::OAuth2ClientRegistrationTemplate;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AuthState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/config/template",      get(list_templates).post(save_template))
        .route("/oauth2/config/template/{id}", delete(delete_template))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/oauth2/config/template — list all OAuth2 provider templates (SYS_ADMIN)
async fn list_templates(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<OAuth2ClientRegistrationTemplate>>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let templates = state.oauth2_template_dao.find_all().await?;
    Ok(Json(templates))
}

/// POST /api/oauth2/config/template — create or update OAuth2 provider template (SYS_ADMIN)
async fn save_template(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(mut body): Json<OAuth2ClientRegistrationTemplate>,
) -> Result<Json<OAuth2ClientRegistrationTemplate>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    if body.id == Uuid::nil() {
        body.id = Uuid::new_v4();
    }
    if body.created_time == 0 {
        body.created_time = chrono::Utc::now().timestamp_millis();
    }
    let saved = state.oauth2_template_dao.save(&body).await?;
    Ok(Json(saved))
}

/// DELETE /api/oauth2/config/template/{id} — delete OAuth2 provider template (SYS_ADMIN)
async fn delete_template(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    state.oauth2_template_dao.delete(id).await?;
    Ok(StatusCode::OK)
}
