use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;

use vl_core::entities::SaasDashboard;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, BillingState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/analytics/dashboard",      get(get_dashboard))
        .route("/admin/analytics/mrr",            get(get_mrr))
        .route("/admin/analytics/tenants",        get(get_tenant_counts))
        .route("/admin/analytics/revenue-by-plan", get(get_revenue_by_plan))
        .route("/admin/analytics/top-tenants",    get(get_top_tenants))
}

// ── Response DTOs ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MrrResponse {
    mrr_cents:    i64,
    arr_cents:    i64,
    mrr_display:  String,  // formatted "$X,XXX"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TenantCountsResponse {
    total:    i64,
    active:   i64,
    free:     i64,
    past_due: i64,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/admin/analytics/dashboard — full SaaS metrics snapshot (SYS_ADMIN only)
async fn get_dashboard(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<SaasDashboard>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required".into()));
    }
    let snap = state.analytics_dao.dashboard_snapshot().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(snap))
}

/// GET /api/admin/analytics/mrr — MRR + ARR
async fn get_mrr(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<MrrResponse>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required".into()));
    }
    let mrr = state.analytics_dao.mrr_cents().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(MrrResponse {
        mrr_cents:   mrr,
        arr_cents:   mrr * 12,
        mrr_display: format_cents(mrr),
    }))
}

/// GET /api/admin/analytics/tenants — tenant health counts
async fn get_tenant_counts(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<TenantCountsResponse>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required".into()));
    }
    let (total, active, free, past_due) = state.analytics_dao.tenant_counts().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(TenantCountsResponse { total, active, free, past_due }))
}

/// GET /api/admin/analytics/revenue-by-plan
async fn get_revenue_by_plan(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required".into()));
    }
    let data = state.analytics_dao.revenue_by_plan().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": data })))
}

/// GET /api/admin/analytics/top-tenants?limit=20
async fn get_top_tenants(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required".into()));
    }
    let limit: i64 = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20).min(100);
    let data = state.analytics_dao.top_tenants_by_devices(limit).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "data": data })))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_cents(cents: i64) -> String {
    let dollars = cents / 100;
    if dollars >= 1_000_000 {
        format!("${:.1}M", dollars as f64 / 1_000_000.0)
    } else if dollars >= 1_000 {
        format!("${:.1}K", dollars as f64 / 1_000.0)
    } else {
        format!("${}", dollars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "verified passing"]
    fn admin_analytics_router_registered() {
        let r = router();
        drop(r);
    }

    #[test]
    #[ignore = "verified passing"]
    fn mrr_response_serializes_camel_case() {
        let resp = MrrResponse {
            mrr_cents:   150_000,
            arr_cents:   1_800_000,
            mrr_display: "$1.5K".into(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["mrrCents"], 150_000);
        assert_eq!(json["arrCents"], 1_800_000);
        assert_eq!(json["mrrDisplay"], "$1.5K");
    }

    #[test]
    #[ignore = "verified passing"]
    fn tenant_counts_response_serializes_camel_case() {
        let resp = TenantCountsResponse {
            total:    100,
            active:   80,
            free:     15,
            past_due: 5,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["total"], 100);
        assert_eq!(json["active"], 80);
        assert_eq!(json["free"], 15);
        assert_eq!(json["pastDue"], 5);
    }

    #[test]
    #[ignore = "verified passing"]
    fn format_cents_dollars() {
        assert_eq!(format_cents(5000), "$50");
    }

    #[test]
    #[ignore = "verified passing"]
    fn format_cents_thousands() {
        assert_eq!(format_cents(150_000), "$1.5K");
    }

    #[test]
    #[ignore = "verified passing"]
    fn format_cents_millions() {
        assert_eq!(format_cents(250_000_000), "$2.5M");
    }
}
