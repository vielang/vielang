use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{
    ApiUsageStateValue, CheckoutSession, InvoiceSummary, SubscriptionPlan,
    TenantApiUsage, TenantSubscription, UsageMetric,
};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, BillingState, CoreState, AuthState}};

// ── Public router (no auth) ───────────────────────────────────────────────────

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/billing/plans", get(list_plans))
}

/// Webhook router — Stripe calls this without a JWT; auth is via HMAC signature.
pub fn webhook_router() -> Router<AppState> {
    Router::new()
        .route("/billing/webhook", post(stripe_webhook))
}

// ── Protected router (JWT required) ──────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/billing/subscription", get(get_subscription))
        .route("/billing/usage",        get(get_usage))
        .route("/billing/invoices",     get(list_invoices))
        .route("/billing/portal",       get(billing_portal))
        .route("/billing/checkout",     post(create_checkout))
        .route("/billing/cancel",       post(cancel_subscription))
        .route("/billing/reactivate",   post(reactivate_subscription))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutRequest {
    /// Accept either planId (UUID from upgrade page) or planName (string)
    pub plan_id:       Option<Uuid>,
    pub plan_name:     Option<String>,
    pub billing_cycle: String, // "monthly" | "annually"
    /// Stripe redirect URLs (optional — fall back to config)
    pub success_url:   Option<String>,
    pub cancel_url:    Option<String>,
}

/// Flat subscription + embedded plan DTO — matches frontend `TenantSubscription` type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionWithPlanResponse {
    pub id:                      Uuid,
    pub tenant_id:               Uuid,
    pub plan:                    SubscriptionPlan,
    pub stripe_customer_id:      Option<String>,
    pub stripe_subscription_id:  Option<String>,
    pub billing_cycle:           String, // "MONTHLY" | "ANNUALLY"
    pub status:                  String,
    pub current_period_start:    Option<i64>,
    pub current_period_end:      Option<i64>,
    pub trial_end:               Option<i64>,
    pub cancel_at_period_end:    bool,
    pub canceled_at:             Option<i64>,
}

/// Single quota metric — matches frontend `QuotaItem`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaDisplay {
    pub used:       i64,
    pub limit:      i64,
    pub percentage: f64,
    pub status:     String,
}

/// Billing period date range.
#[derive(Debug, Serialize)]
pub struct PeriodRange {
    pub start: i64,
    pub end:   i64,
}

/// Usage response — matches frontend `UsageResponse`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageDisplayResponse {
    pub period:    PeriodRange,
    pub devices:   QuotaDisplay,
    pub messages:  QuotaDisplay,
    pub rule_exec: QuotaDisplay,
    pub js_exec:   QuotaDisplay,
    pub email:     QuotaDisplay,
    pub sms:       QuotaDisplay,
    pub alarm:     QuotaDisplay,
}

// ── Handlers — public ─────────────────────────────────────────────────────────

/// GET /api/billing/plans — list active plans (no auth, for pricing page)
async fn list_plans(
    State(state): State<BillingState>,
) -> Result<Json<Vec<SubscriptionPlan>>, ApiError> {
    let plans = state.plan_dao.list_active().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(plans))
}

// ── Handlers — protected ──────────────────────────────────────────────────────

/// GET /api/billing/subscription — current plan + subscription details
async fn get_subscription(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<SubscriptionWithPlanResponse>, ApiError> {
    let sub = state.subscription_dao.find_by_tenant(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let (sub, plan) = match sub {
        Some(s) => {
            let plan = state.plan_dao.find_by_id(s.plan_id).await
                .map_err(|e| ApiError::Internal(e.to_string()))?
                .unwrap_or_else(free_plan_fallback);
            (s, plan)
        }
        None => {
            let plan = free_plan_fallback();
            let now  = chrono::Utc::now().timestamp_millis();
            let s = TenantSubscription {
                id:                     Uuid::new_v4(),
                created_time:           now,
                updated_time:           now,
                tenant_id:              ctx.tenant_id,
                plan_id:                plan.id,
                stripe_customer_id:     None,
                stripe_subscription_id: None,
                billing_cycle:          "monthly".into(),
                status:                 "active".into(),
                current_period_start:   None,
                current_period_end:     None,
                trial_end:              None,
                cancel_at_period_end:   false,
                canceled_at:            None,
            };
            (s, plan)
        }
    };

    let billing_cycle = if sub.billing_cycle.to_ascii_lowercase() == "annually" {
        "ANNUALLY".to_string()
    } else {
        "MONTHLY".to_string()
    };

    Ok(Json(SubscriptionWithPlanResponse {
        id:                     sub.id,
        tenant_id:              sub.tenant_id,
        plan,
        stripe_customer_id:     sub.stripe_customer_id,
        stripe_subscription_id: sub.stripe_subscription_id,
        billing_cycle,
        status:                 sub.status,
        current_period_start:   sub.current_period_start,
        current_period_end:     sub.current_period_end,
        trial_end:              sub.trial_end,
        cancel_at_period_end:   sub.cancel_at_period_end,
        canceled_at:            sub.canceled_at,
    }))
}

/// GET /api/billing/usage — current month usage vs plan limits
async fn get_usage(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UsageDisplayResponse>, ApiError> {
    let period_str = chrono::Utc::now().format("%Y-%m").to_string();

    // Get plan for limits
    let plan = match state.subscription_dao.find_by_tenant(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(sub) => state.plan_dao.find_by_id(sub.plan_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .unwrap_or_else(free_plan_fallback),
        None => free_plan_fallback(),
    };

    // Get current usage from DB
    let usage = match state.api_usage_dao.get_usage_domain(ctx.tenant_id, &plan).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(u) => u,
        None => TenantApiUsage {
            tenant_id:      ctx.tenant_id,
            billing_period: period_str.clone(),
            transport_msg:  UsageMetric::new("TRANSPORT_MSG",  0, plan.max_transport_msgs_month),
            transport_dp:   UsageMetric::new("TRANSPORT_DP",   0, plan.max_transport_msgs_month),
            re_exec:        UsageMetric::new("RE_EXEC",         0, plan.max_js_execs_month),
            js_exec:        UsageMetric::new("JS_EXEC",         0, plan.max_js_execs_month),
            email:          UsageMetric::new("EMAIL",            0, plan.max_emails_month as i64),
            sms:            UsageMetric::new("SMS",              0, plan.max_sms_month as i64),
            alarm:          UsageMetric::new("ALARM",            0, plan.max_alarms as i64),
            active_devices: UsageMetric::new("ACTIVE_DEVICES",  0, plan.max_devices as i64),
            storage_dp:     UsageMetric::new("STORAGE_DP",      0, 0),
            rpc:            UsageMetric::new("RPC",              0, 0),
            rule_engine_exec: UsageMetric::new("RULE_ENGINE_EXEC", 0, 0),
        },
    };

    Ok(Json(UsageDisplayResponse {
        period:    billing_period_range(&usage.billing_period),
        devices:   metric_to_quota(&usage.active_devices),
        messages:  metric_to_quota(&usage.transport_msg),
        rule_exec: metric_to_quota(&usage.re_exec),
        js_exec:   metric_to_quota(&usage.js_exec),
        email:     metric_to_quota(&usage.email),
        sms:       metric_to_quota(&usage.sms),
        alarm:     metric_to_quota(&usage.alarm),
    }))
}

/// GET /api/billing/invoices — Stripe invoice history
async fn list_invoices(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<InvoiceSummary>>, ApiError> {
    let stripe = match &state.stripe_service {
        Some(s) => s,
        None    => return Ok(Json(vec![])),
    };

    let sub = state.subscription_dao.find_by_tenant(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let cust_id = match sub.as_ref().and_then(|s| s.stripe_customer_id.as_deref()) {
        Some(id) => id,
        None     => return Ok(Json(vec![])),
    };

    let invoices = stripe.list_invoices(cust_id, 20).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(invoices))
}

/// GET /api/billing/portal — Stripe self-service portal URL
async fn billing_portal(
    State(state): State<BillingState>,
    State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stripe = state.stripe_service.as_ref()
        .ok_or(ApiError::NotImplemented("Stripe not configured".into()))?;

    let sub = state.subscription_dao.find_by_tenant(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::BadRequest("No subscription found".into()))?;

    let cust_id = sub.stripe_customer_id.as_deref()
        .ok_or(ApiError::BadRequest(
            "No Stripe customer. Complete a checkout first.".into()
        ))?;

    let return_url = format!("{}/billing", core.config.server.base_url());
    let portal_url = stripe.create_portal_session(cust_id, &return_url).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "url": portal_url })))
}

/// POST /api/billing/checkout — create Stripe Checkout Session
async fn create_checkout(
    State(state): State<BillingState>,
    State(auth): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutSession>, ApiError> {
    let stripe = state.stripe_service.as_ref()
        .ok_or(ApiError::NotImplemented("Stripe not configured".into()))?;

    // Resolve plan — accept planId (UUID) or planName (string)
    let plan = if let Some(plan_id) = req.plan_id {
        state.plan_dao.find_by_id(plan_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or(ApiError::NotFound(format!("Plan [{}] not found", plan_id)))?
    } else {
        let name = req.plan_name.as_deref().unwrap_or("");
        state.plan_dao.find_by_name(name).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or(ApiError::NotFound(format!("Plan '{}' not found", name)))?
    };

    // Resolve Stripe price ID
    let price_id = match req.billing_cycle.as_str() {
        "annual" => plan.stripe_price_id_annually.as_deref(),
        _        => plan.stripe_price_id_monthly.as_deref(),
    }.ok_or(ApiError::BadRequest(
        format!("Plan '{}' is not available for purchase", plan.name)
    ))?;

    // Get tenant email for Stripe customer creation
    let tenant = auth.tenant_dao.find_by_id(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or(ApiError::NotFound("Tenant not found".into()))?;

    let email = auth.user_dao.find_by_id(ctx.user_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|u| u.email)
        .unwrap_or_else(|| format!("tenant-{}@vielang.local", ctx.tenant_id));

    // Get or create Stripe customer
    let existing_sub = state.subscription_dao.find_by_tenant(ctx.tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let cust_id = match existing_sub.as_ref().and_then(|s| s.stripe_customer_id.as_deref()) {
        Some(id) => id.to_string(),
        None => {
            let id = stripe.get_or_create_customer(ctx.tenant_id, &email, &tenant.title).await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            // Persist customer ID
            let now = chrono::Utc::now().timestamp_millis();
            let free_plan = state.plan_dao.find_by_name("free").await
                .map_err(|e| ApiError::Internal(e.to_string()))?
                .unwrap_or_else(free_plan_fallback);
            let sub = TenantSubscription {
                id:                     existing_sub.as_ref().map(|s| s.id).unwrap_or_else(Uuid::new_v4),
                created_time:           existing_sub.as_ref().map(|s| s.created_time).unwrap_or(now),
                updated_time:           now,
                tenant_id:              ctx.tenant_id,
                plan_id:                existing_sub.as_ref().map(|s| s.plan_id).unwrap_or(free_plan.id),
                stripe_customer_id:     Some(id.clone()),
                stripe_subscription_id: existing_sub.as_ref().and_then(|s| s.stripe_subscription_id.clone()),
                billing_cycle:          "monthly".into(),
                status:                 existing_sub.as_ref().map(|s| s.status.clone()).unwrap_or_else(|| "free".into()),
                current_period_start:   existing_sub.as_ref().and_then(|s| s.current_period_start),
                current_period_end:     existing_sub.as_ref().and_then(|s| s.current_period_end),
                trial_end:              existing_sub.as_ref().and_then(|s| s.trial_end),
                cancel_at_period_end:   false,
                canceled_at:            None,
            };
            state.subscription_dao.upsert(&sub).await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            id
        }
    };

    let session = stripe.create_checkout_session(&cust_id, price_id, ctx.tenant_id, &req.billing_cycle)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(session))
}

/// POST /api/billing/cancel — cancel subscription at end of current period
async fn cancel_subscription(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.subscription_dao.update_status(ctx.tenant_id, "active", None, true).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "cancelAtPeriodEnd": true })))
}

/// POST /api/billing/reactivate — undo cancellation (remove cancel_at_period_end)
async fn reactivate_subscription(
    State(state): State<BillingState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.subscription_dao.update_status(ctx.tenant_id, "active", None, false).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "cancelAtPeriodEnd": false })))
}

// ── Webhook handler ───────────────────────────────────────────────────────────

/// POST /api/billing/webhook — Stripe sends events here (no JWT, HMAC verified)
async fn stripe_webhook(
    State(state): State<BillingState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stripe = state.stripe_service.as_ref()
        .ok_or(ApiError::NotImplemented("Stripe not configured".into()))?;

    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::BadRequest("Missing Stripe-Signature header".into()))?;

    let event = stripe.verify_webhook(&body, sig)
        .map_err(|e| ApiError::Unauthorized(format!("Webhook verification failed: {}", e)))?;

    let event_id   = event["id"].as_str().unwrap_or("").to_string();
    let event_type = event["type"].as_str().unwrap_or("").to_string();
    let now_ms = chrono::Utc::now().timestamp_millis();

    // Idempotency: skip if already processed
    let exists = state.subscription_dao.stripe_event_exists(&event_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if exists {
        return Ok(Json(serde_json::json!({ "status": "duplicate" })));
    }

    // Persist for audit trail
    state.subscription_dao.insert_stripe_event(&event_id, &event_type, &event, now_ms).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Dispatch webhook event
    let result = handle_webhook_event(&state, &event_type, &event).await;

    match result {
        Ok(()) => {
            state.subscription_dao.mark_stripe_event_processed(&event_id, None).await.ok();
            Ok(Json(serde_json::json!({ "status": "ok" })))
        }
        Err(e) => {
            let msg = format!("{:?}", e);
            tracing::error!("Stripe webhook {} processing error: {}", event_type, msg);
            state.subscription_dao.mark_stripe_event_processed(&event_id, Some(&msg)).await.ok();
            // Return 200 to Stripe (prevent retries for business logic errors)
            Ok(Json(serde_json::json!({ "status": "error", "message": msg })))
        }
    }
}

async fn handle_webhook_event(
    state:      &BillingState,
    event_type: &str,
    event:      &serde_json::Value,
) -> Result<(), ApiError> {
    match event_type {
        "checkout.session.completed" => {
            handle_checkout_completed(state, event).await
        }
        "customer.subscription.updated" => {
            handle_subscription_updated(state, event).await
        }
        "customer.subscription.deleted" => {
            handle_subscription_deleted(state, event).await
        }
        "invoice.payment_succeeded" => {
            handle_invoice_paid(state, event).await
        }
        "invoice.payment_failed" => {
            handle_invoice_failed(state, event).await
        }
        _ => Ok(()), // Ignore other events
    }
}

async fn handle_checkout_completed(state: &BillingState, event: &serde_json::Value) -> Result<(), ApiError> {
    let session  = &event["data"]["object"];
    let cust_id  = session["customer"].as_str().unwrap_or("").to_string();
    let sub_id   = session["subscription"].as_str().map(|s| s.to_string());
    let tenant_id_str = session["metadata"]["tenant_id"].as_str().unwrap_or("");
    let billing_cycle = session["metadata"]["billing_cycle"].as_str().unwrap_or("monthly").to_string();

    let tenant_id: Uuid = tenant_id_str.parse()
        .map_err(|_| ApiError::BadRequest("Invalid tenant_id in Stripe metadata".into()))?;

    // Find the plan from the Stripe price ID
    let line_items_price = session["display_items"][0]["plan"]["id"]
        .as_str()
        .or(session["line_items"]["data"][0]["price"]["id"].as_str())
        .unwrap_or("");

    // Find plan with matching Stripe price ID
    let plans = state.plan_dao.list_active().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let plan = plans.iter().find(|p| {
        p.stripe_price_id_monthly.as_deref() == Some(line_items_price)
            || p.stripe_price_id_annually.as_deref() == Some(line_items_price)
    }).cloned().unwrap_or_else(free_plan_fallback);

    let now = chrono::Utc::now().timestamp_millis();
    let existing = state.subscription_dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let sub = TenantSubscription {
        id:                     existing.as_ref().map(|s| s.id).unwrap_or_else(Uuid::new_v4),
        created_time:           existing.as_ref().map(|s| s.created_time).unwrap_or(now),
        updated_time:           now,
        tenant_id,
        plan_id:                plan.id,
        stripe_customer_id:     Some(cust_id),
        stripe_subscription_id: sub_id,
        billing_cycle,
        status:                 "active".into(),
        current_period_start:   Some(now),
        current_period_end:     Some(now + 30 * 24 * 3600 * 1000),
        trial_end:              None,
        cancel_at_period_end:   false,
        canceled_at:            None,
    };
    state.subscription_dao.upsert(&sub).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    tracing::info!("Tenant {} activated plan '{}' via Stripe", tenant_id, plan.name);
    Ok(())
}

async fn handle_subscription_updated(state: &BillingState, event: &serde_json::Value) -> Result<(), ApiError> {
    let sub_obj = &event["data"]["object"];
    let sub_id  = sub_obj["id"].as_str().unwrap_or("");
    let status  = sub_obj["status"].as_str().unwrap_or("active");
    let period_end = sub_obj["current_period_end"].as_i64().map(|t| t * 1000);
    let cancel_at = sub_obj["cancel_at_period_end"].as_bool().unwrap_or(false);

    let existing = state.subscription_dao.find_by_stripe_sub_id(sub_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if let Some(s) = existing {
        state.subscription_dao.update_status(s.tenant_id, status, period_end, cancel_at).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }
    Ok(())
}

async fn handle_subscription_deleted(state: &BillingState, event: &serde_json::Value) -> Result<(), ApiError> {
    let sub_id = event["data"]["object"]["id"].as_str().unwrap_or("");
    let canceled_at = event["data"]["object"]["canceled_at"].as_i64().map(|t| t * 1000);

    if let Some(mut s) = state.subscription_dao.find_by_stripe_sub_id(sub_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        // Downgrade to free plan
        let free = state.plan_dao.find_by_name("free").await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .unwrap_or_else(free_plan_fallback);

        let now = chrono::Utc::now().timestamp_millis();
        s.plan_id                = free.id;
        s.status                 = "canceled".into();
        s.canceled_at            = canceled_at.or(Some(now));
        s.cancel_at_period_end   = false;
        s.current_period_end     = None;
        s.updated_time           = now;
        state.subscription_dao.upsert(&s).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        tracing::info!("Tenant {} subscription canceled — downgraded to free", s.tenant_id);
    }
    Ok(())
}

async fn handle_invoice_paid(state: &BillingState, event: &serde_json::Value) -> Result<(), ApiError> {
    let inv      = &event["data"]["object"];
    let sub_id   = inv["subscription"].as_str().unwrap_or("");
    let period_end = inv["period_end"].as_i64().map(|t| t * 1000);

    if let Some(s) = state.subscription_dao.find_by_stripe_sub_id(sub_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        state.subscription_dao.update_status(s.tenant_id, "active", period_end, false).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }
    Ok(())
}

async fn handle_invoice_failed(state: &BillingState, event: &serde_json::Value) -> Result<(), ApiError> {
    let sub_id = event["data"]["object"]["subscription"].as_str().unwrap_or("");

    if let Some(s) = state.subscription_dao.find_by_stripe_sub_id(sub_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        state.subscription_dao.update_status(s.tenant_id, "past_due", None, false).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tracing::warn!("Tenant {} payment failed — status set to past_due", s.tenant_id);
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn metric_to_quota(m: &UsageMetric) -> QuotaDisplay {
    QuotaDisplay {
        used:       m.value,
        limit:      m.limit,
        percentage: m.pct_used as f64,
        status: match m.state {
            ApiUsageStateValue::Enabled  => "ENABLED".into(),
            ApiUsageStateValue::Warning  => "WARNING".into(),
            ApiUsageStateValue::Disabled => "DISABLED".into(),
        },
    }
}

/// Convert "YYYY-MM" period string to start/end millisecond timestamps.
fn billing_period_range(period: &str) -> PeriodRange {
    use chrono::{NaiveDate, TimeZone, Utc};
    let parts: Vec<&str> = period.splitn(2, '-').collect();
    if parts.len() == 2 {
        if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
            let start_ms = NaiveDate::from_ymd_opt(year, month, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| Utc.from_utc_datetime(&dt).timestamp_millis())
                .unwrap_or(0);
            let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
            let end_ms = NaiveDate::from_ymd_opt(ny, nm, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| Utc.from_utc_datetime(&dt).timestamp_millis() - 1)
                .unwrap_or(0);
            return PeriodRange { start: start_ms, end: end_ms };
        }
    }
    PeriodRange { start: 0, end: 0 }
}

/// Default free plan stub used when DB lookup fails or no subscription exists.
pub fn free_plan_fallback() -> SubscriptionPlan {
    SubscriptionPlan {
        id:                       Uuid::nil(),
        created_time:             0,
        name:                     "free".into(),
        display_name:             "Free".into(),
        description:              None,
        price_monthly:            0,
        price_annually:           0,
        stripe_price_id_monthly:  None,
        stripe_price_id_annually: None,
        max_devices:              10,
        max_users:                3,
        max_assets:               10,
        max_dashboards:           5,
        max_rule_chains:          2,
        max_edges:                0,
        max_transport_msgs_month: 50_000,
        max_js_execs_month:       5_000,
        max_emails_month:         50,
        max_sms_month:            0,
        max_alarms:               50,
        max_api_keys:             1,
        feature_white_label:      false,
        feature_edge_computing:   false,
        feature_advanced_rbac:    false,
        feature_audit_log:        false,
        feature_sso:              false,
        feature_api_export:       false,
        sort_order:               0,
        is_active:                true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_creates_without_panic() {
        let _ = router();
    }

    #[test]
    fn public_router_creates_without_panic() {
        let _ = public_router();
    }

    #[test]
    fn webhook_router_creates_without_panic() {
        let _ = webhook_router();
    }

    #[test]
    fn free_plan_fallback_has_correct_defaults() {
        let plan = free_plan_fallback();
        assert_eq!(plan.name, "free");
        assert_eq!(plan.display_name, "Free");
        assert_eq!(plan.max_devices, 10);
        assert_eq!(plan.max_users, 3);
        assert!(plan.is_active);
        assert!(!plan.feature_white_label);
        assert_eq!(plan.id, Uuid::nil());
    }

    #[test]
    fn free_plan_serializes_to_camel_case() {
        let plan = free_plan_fallback();
        let json = serde_json::to_value(&plan).unwrap();
        assert!(json.get("displayName").is_some());
        assert!(json.get("maxDevices").is_some());
        assert!(json.get("maxTransportMsgsMonth").is_some());
        assert!(json.get("featureWhiteLabel").is_some());
        // Ensure snake_case fields are NOT present
        assert!(json.get("display_name").is_none());
        assert!(json.get("max_devices").is_none());
    }

    #[test]
    fn subscription_with_plan_response_serializes_to_camel_case() {
        let resp = SubscriptionWithPlanResponse {
            id:                      Uuid::nil(),
            tenant_id:               Uuid::nil(),
            plan:                    free_plan_fallback(),
            stripe_customer_id:      None,
            stripe_subscription_id:  None,
            billing_cycle:           "MONTHLY".into(),
            status:                  "active".into(),
            current_period_start:    Some(1000),
            current_period_end:      Some(2000),
            trial_end:               None,
            cancel_at_period_end:    false,
            canceled_at:             None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["billingCycle"], "MONTHLY");
        assert_eq!(json["status"], "active");
        assert_eq!(json["cancelAtPeriodEnd"], false);
        assert!(json.get("tenantId").is_some());
        assert!(json.get("currentPeriodStart").is_some());
    }

    #[test]
    fn checkout_request_deserializes_from_camel_case() {
        let json = serde_json::json!({
            "planName": "pro",
            "billingCycle": "monthly",
            "successUrl": "https://example.com/success"
        });
        let req: CheckoutRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.plan_name.as_deref(), Some("pro"));
        assert_eq!(req.billing_cycle, "monthly");
        assert_eq!(req.success_url.as_deref(), Some("https://example.com/success"));
        assert!(req.plan_id.is_none());
    }

    #[test]
    fn quota_display_serializes_correctly() {
        let q = QuotaDisplay {
            used: 5,
            limit: 10,
            percentage: 50.0,
            status: "ENABLED".into(),
        };
        let json = serde_json::to_value(&q).unwrap();
        assert_eq!(json["used"], 5);
        assert_eq!(json["limit"], 10);
        assert_eq!(json["percentage"], 50.0);
        assert_eq!(json["status"], "ENABLED");
    }

    #[test]
    fn billing_period_range_parses_valid_period() {
        let range = billing_period_range("2026-03");
        assert!(range.start > 0);
        assert!(range.end > range.start);
    }

    #[test]
    fn billing_period_range_returns_zero_for_invalid() {
        let range = billing_period_range("invalid");
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 0);
    }

    #[test]
    fn metric_to_quota_converts_correctly() {
        let m = UsageMetric::new("TEST", 42, 100);
        let q = metric_to_quota(&m);
        assert_eq!(q.used, 42);
        assert_eq!(q.limit, 100);
    }
}
