use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::AppState};

/// QuotaMiddleware — Phase 71
///
/// Reads the tenant's current subscription plan limits and compares against
/// the live api_usage_state for the current billing period. If a critical
/// quota is fully consumed (state = DISABLED), returns HTTP 429.
///
/// Only enforced for authenticated TENANT_ADMIN / CUSTOMER_USER requests.
/// SYS_ADMIN is never quota-limited.
///
/// Adds response header:
///   `X-Quota-State: ok | warning | disabled`
pub async fn quota_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Only enforce for tenant-scoped requests
    let ctx = request.extensions().get::<SecurityContext>().cloned();
    let tenant_id = match ctx {
        Some(ref c) if !c.is_sys_admin() => c.tenant_id,
        _ => return Ok(next.run(request).await),
    };

    let quota_state = check_quota(&state, tenant_id).await?;

    if quota_state == QuotaState::Disabled {
        return Err(ApiError::TooManyRequests(
            "Monthly quota exceeded. Please upgrade your plan.".into(),
        ));
    }

    let mut response = next.run(request).await;
    let header_val = match quota_state {
        QuotaState::Ok       => "ok",
        QuotaState::Warning  => "warning",
        QuotaState::Disabled => "disabled",
    };
    response.headers_mut().insert(
        "x-quota-state",
        axum::http::HeaderValue::from_static(header_val),
    );
    Ok(response)
}

#[derive(PartialEq)]
enum QuotaState { Ok, Warning, Disabled }

async fn check_quota(state: &AppState, tenant_id: Uuid) -> Result<QuotaState, ApiError> {
    // Load plan limits
    let plan = match state.subscription_dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(sub) => state.plan_dao.find_by_id(sub.plan_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .unwrap_or_else(crate::routes::billing::free_plan_fallback),
        None => crate::routes::billing::free_plan_fallback(),
    };

    // Load usage via public domain method (no row = no usage yet → ok)
    let usage = state.api_usage_dao
        .get_usage_domain(tenant_id, &plan)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let usage = match usage {
        Some(u) => u,
        None    => return Ok(QuotaState::Ok),
    };

    // Extract raw counters from UsageMetric values
    let transport_msg = usage.transport_msg.value;
    let alarm         = usage.alarm.value;

    // Compute worst-case ratio across enforced metrics
    let ratios = [
        ratio(transport_msg, plan.max_transport_msgs_month),
        ratio(alarm,         plan.max_alarms as i64),
    ];

    let max = ratios.iter().cloned().fold(0.0_f64, f64::max);

    Ok(if max >= 1.0 {
        QuotaState::Disabled
    } else if max >= 0.8 {
        QuotaState::Warning
    } else {
        QuotaState::Ok
    })
}

fn ratio(used: i64, limit: i64) -> f64 {
    if limit <= 0 { return 0.0; } // unlimited or unset → never disabled
    used as f64 / limit as f64
}
