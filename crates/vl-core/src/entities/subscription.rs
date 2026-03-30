use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors the `subscription_plan` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPlan {
    pub id:                        Uuid,
    pub created_time:              i64,
    pub name:                      String,
    pub display_name:              String,
    pub description:               Option<String>,
    pub price_monthly:             i32,
    pub price_annually:            i32,
    pub stripe_price_id_monthly:   Option<String>,
    pub stripe_price_id_annually:  Option<String>,
    pub max_devices:               i32,
    pub max_users:                 i32,
    pub max_assets:                i32,
    pub max_dashboards:            i32,
    pub max_rule_chains:           i32,
    pub max_edges:                 i32,
    pub max_transport_msgs_month:  i64,
    pub max_js_execs_month:        i64,
    pub max_emails_month:          i32,
    pub max_sms_month:             i32,
    pub max_alarms:                i32,
    pub max_api_keys:              i32,
    pub feature_white_label:       bool,
    pub feature_edge_computing:    bool,
    pub feature_advanced_rbac:     bool,
    pub feature_audit_log:         bool,
    pub feature_sso:               bool,
    pub feature_api_export:        bool,
    pub sort_order:                i32,
    pub is_active:                 bool,
}

/// Mirrors the `tenant_subscription` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantSubscription {
    pub id:                    Uuid,
    pub created_time:          i64,
    pub updated_time:          i64,
    pub tenant_id:             Uuid,
    pub plan_id:               Uuid,
    pub stripe_customer_id:    Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub billing_cycle:         String,
    pub status:                String,
    pub current_period_start:  Option<i64>,
    pub current_period_end:    Option<i64>,
    pub trial_end:             Option<i64>,
    pub cancel_at_period_end:  bool,
    pub canceled_at:           Option<i64>,
}

/// Returned when creating a Stripe Checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutSession {
    pub session_id:  String,
    pub session_url: String,
}

/// Summary of a single Stripe invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSummary {
    pub id:           String,
    pub number:       Option<String>,
    pub amount_paid:  i64,
    pub currency:     String,
    pub status:       String,
    pub period_start: i64,
    pub period_end:   i64,
    pub invoice_url:  Option<String>,
    pub pdf_url:      Option<String>,
}
