use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full SaaS analytics snapshot returned by GET /api/admin/analytics/dashboard.
/// Only accessible to SYS_ADMIN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaasDashboard {
    /// Epoch milliseconds when this snapshot was generated.
    pub generated_at:     i64,
    /// Monthly Recurring Revenue in USD cents.
    pub mrr_cents:        i64,
    /// Annual Run Rate in USD cents (mrr_cents * 12).
    pub arr_cents:        i64,
    /// Total number of tenant rows in the system.
    pub total_tenants:    i64,
    /// Tenants with subscription status = 'active'.
    pub active_tenants:   i64,
    /// Tenants on the free plan.
    pub free_tenants:     i64,
    /// Tenants with subscription status = 'past_due'.
    pub past_due_tenants: i64,
    /// Rolling 30-day churn rate (churned / start-of-period tenants).  Range: 0.0–1.0.
    pub churn_rate:       f32,
    /// MRR broken down by subscription plan.
    pub revenue_by_plan:  Vec<PlanRevenue>,
    /// Rolling 13-month new vs churned tenant cohort data.
    pub tenant_health:    Vec<MonthlyTenantHealth>,
    /// Top 20 tenants by transport message volume in the current billing month.
    pub top_tenants:      Vec<TenantUsageSummary>,
}

/// Revenue contribution from a single subscription plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRevenue {
    /// Internal plan name, e.g. "free" | "starter" | "pro" | "enterprise".
    pub plan_name:    String,
    /// Human-readable plan name shown in UI, e.g. "Pro".
    pub display_name: String,
    /// Number of active/trialing tenants on this plan.
    pub tenant_count: i64,
    /// Total MRR from this plan in USD cents.
    pub mrr_cents:    i64,
    /// This plan's share of total MRR as a percentage (0.0–100.0).
    pub pct_of_total: f32,
}

/// New vs churned tenant counts for a single calendar month.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyTenantHealth {
    /// Calendar month in "YYYY-MM" format.
    pub month:           String,
    /// Tenants whose account was created in this month.
    pub new_tenants:     i64,
    /// Tenants whose subscription was canceled in this month.
    pub churned_tenants: i64,
}

/// Per-tenant usage snapshot used in the top-tenants list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantUsageSummary {
    pub tenant_id:    Uuid,
    pub tenant_name:  String,
    /// Internal plan name, e.g. "pro".
    pub plan_name:    String,
    /// Total devices owned by this tenant.
    pub device_count: i64,
    /// Transport messages recorded in the current billing month.
    pub transport_msgs_this_month: i64,
    /// Monthly Recurring Revenue for this tenant in USD cents.
    pub mrr_cents:    i64,
}
