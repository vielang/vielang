use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use vl_core::entities::{AuditLog, AuditActionType, AuditActionStatus};
use uuid::Uuid;

use crate::middleware::auth::SecurityContext;
use crate::state::AppState;

/// Middleware that logs every mutating HTTP request as a persisted audit trail entry.
///
/// GET/HEAD/OPTIONS are only logged to tracing (not persisted) to reduce write volume.
/// POST/PUT/PATCH/DELETE are saved to `audit_log` table via `AuditLogDao`.
pub async fn audit_log_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let start  = Instant::now();
    let method = request.method().clone();

    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());

    // Extract security context before consuming the request
    let ctx = request.extensions().get::<SecurityContext>().cloned();

    let response = next.run(request).await;

    let duration_ms = start.elapsed().as_millis();
    let status      = response.status().as_u16();
    let user_id_str = ctx.as_ref().map(|c| c.user_id.to_string()).unwrap_or_else(|| "anonymous".into());

    tracing::info!(
        target = "vielang::audit",
        method  = %method,
        path    = %path,
        status  = status,
        duration_ms = duration_ms,
        user_id = %user_id_str,
        "HTTP request"
    );

    // Persist write operations to DB (fire-and-forget — never block the response)
    let is_write = matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
    if is_write {
        let action_type = method_to_action_type(method.as_str());
        let action_status = if status < 400 {
            AuditActionStatus::Success
        } else {
            AuditActionStatus::Failure
        };
        let failure_details = if status >= 400 {
            Some(format!("HTTP {}", status))
        } else {
            None
        };

        let log = AuditLog {
            id:                     Uuid::new_v4(),
            created_time:           chrono::Utc::now().timestamp_millis(),
            tenant_id:              ctx.as_ref().map(|c| c.tenant_id).unwrap_or(Uuid::nil()),
            user_id:                ctx.as_ref().map(|c| c.user_id),
            user_name:              None,
            action_type,
            action_data:            serde_json::json!({ "path": path }),
            action_status,
            action_failure_details: failure_details,
            entity_type:            None,
            entity_id:              None,
            entity_name:            None,
        };

        let dao = state.audit_log_dao.clone();
        tokio::spawn(async move {
            if let Err(e) = dao.save(&log).await {
                tracing::warn!("Failed to persist audit log: {}", e);
            }
        });
    }

    response
}

fn method_to_action_type(method: &str) -> AuditActionType {
    match method {
        "POST"   => AuditActionType::Added,
        "PUT"    => AuditActionType::Updated,
        "PATCH"  => AuditActionType::Updated,
        "DELETE" => AuditActionType::Deleted,
        _        => AuditActionType::Updated,
    }
}
