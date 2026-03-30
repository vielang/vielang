use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{SubscriptionPlan, TenantSubscription};

use crate::error::DaoError;

// ── SubscriptionPlanDao ───────────────────────────────────────────────────────

pub struct SubscriptionPlanDao {
    pool: PgPool,
}

impl SubscriptionPlanDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List all active subscription plans, ordered by sort_order.
    pub async fn list_active(&self) -> Result<Vec<SubscriptionPlan>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT id, created_time, name, display_name, description,
                      price_monthly, price_annually,
                      stripe_price_id_monthly, stripe_price_id_annually,
                      max_devices, max_users, max_assets, max_dashboards,
                      max_rule_chains, max_edges, max_transport_msgs_month,
                      max_js_execs_month, max_emails_month, max_sms_month,
                      max_alarms, max_api_keys,
                      feature_white_label, feature_edge_computing,
                      feature_advanced_rbac, feature_audit_log,
                      feature_sso, feature_api_export,
                      sort_order, is_active
               FROM subscription_plan
               WHERE is_active = true
               ORDER BY sort_order"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubscriptionPlan {
                id:                       r.id,
                created_time:             r.created_time,
                name:                     r.name,
                display_name:             r.display_name,
                description:              r.description,
                price_monthly:            r.price_monthly,
                price_annually:           r.price_annually,
                stripe_price_id_monthly:  r.stripe_price_id_monthly,
                stripe_price_id_annually: r.stripe_price_id_annually,
                max_devices:              r.max_devices,
                max_users:                r.max_users,
                max_assets:               r.max_assets,
                max_dashboards:           r.max_dashboards,
                max_rule_chains:          r.max_rule_chains,
                max_edges:                r.max_edges,
                max_transport_msgs_month: r.max_transport_msgs_month,
                max_js_execs_month:       r.max_js_execs_month,
                max_emails_month:         r.max_emails_month,
                max_sms_month:            r.max_sms_month,
                max_alarms:               r.max_alarms,
                max_api_keys:             r.max_api_keys,
                feature_white_label:      r.feature_white_label,
                feature_edge_computing:   r.feature_edge_computing,
                feature_advanced_rbac:    r.feature_advanced_rbac,
                feature_audit_log:        r.feature_audit_log,
                feature_sso:              r.feature_sso,
                feature_api_export:       r.feature_api_export,
                sort_order:               r.sort_order,
                is_active:                r.is_active,
            })
            .collect())
    }

    /// Find a plan by its unique name (e.g. "free", "starter", "pro", "enterprise").
    pub async fn find_by_name(&self, name: &str) -> Result<Option<SubscriptionPlan>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, name, display_name, description,
                      price_monthly, price_annually,
                      stripe_price_id_monthly, stripe_price_id_annually,
                      max_devices, max_users, max_assets, max_dashboards,
                      max_rule_chains, max_edges, max_transport_msgs_month,
                      max_js_execs_month, max_emails_month, max_sms_month,
                      max_alarms, max_api_keys,
                      feature_white_label, feature_edge_computing,
                      feature_advanced_rbac, feature_audit_log,
                      feature_sso, feature_api_export,
                      sort_order, is_active
               FROM subscription_plan
               WHERE name = $1"#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SubscriptionPlan {
            id:                       r.id,
            created_time:             r.created_time,
            name:                     r.name,
            display_name:             r.display_name,
            description:              r.description,
            price_monthly:            r.price_monthly,
            price_annually:           r.price_annually,
            stripe_price_id_monthly:  r.stripe_price_id_monthly,
            stripe_price_id_annually: r.stripe_price_id_annually,
            max_devices:              r.max_devices,
            max_users:                r.max_users,
            max_assets:               r.max_assets,
            max_dashboards:           r.max_dashboards,
            max_rule_chains:          r.max_rule_chains,
            max_edges:                r.max_edges,
            max_transport_msgs_month: r.max_transport_msgs_month,
            max_js_execs_month:       r.max_js_execs_month,
            max_emails_month:         r.max_emails_month,
            max_sms_month:            r.max_sms_month,
            max_alarms:               r.max_alarms,
            max_api_keys:             r.max_api_keys,
            feature_white_label:      r.feature_white_label,
            feature_edge_computing:   r.feature_edge_computing,
            feature_advanced_rbac:    r.feature_advanced_rbac,
            feature_audit_log:        r.feature_audit_log,
            feature_sso:              r.feature_sso,
            feature_api_export:       r.feature_api_export,
            sort_order:               r.sort_order,
            is_active:                r.is_active,
        }))
    }

    /// Find a plan by its UUID primary key.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SubscriptionPlan>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, name, display_name, description,
                      price_monthly, price_annually,
                      stripe_price_id_monthly, stripe_price_id_annually,
                      max_devices, max_users, max_assets, max_dashboards,
                      max_rule_chains, max_edges, max_transport_msgs_month,
                      max_js_execs_month, max_emails_month, max_sms_month,
                      max_alarms, max_api_keys,
                      feature_white_label, feature_edge_computing,
                      feature_advanced_rbac, feature_audit_log,
                      feature_sso, feature_api_export,
                      sort_order, is_active
               FROM subscription_plan
               WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SubscriptionPlan {
            id:                       r.id,
            created_time:             r.created_time,
            name:                     r.name,
            display_name:             r.display_name,
            description:              r.description,
            price_monthly:            r.price_monthly,
            price_annually:           r.price_annually,
            stripe_price_id_monthly:  r.stripe_price_id_monthly,
            stripe_price_id_annually: r.stripe_price_id_annually,
            max_devices:              r.max_devices,
            max_users:                r.max_users,
            max_assets:               r.max_assets,
            max_dashboards:           r.max_dashboards,
            max_rule_chains:          r.max_rule_chains,
            max_edges:                r.max_edges,
            max_transport_msgs_month: r.max_transport_msgs_month,
            max_js_execs_month:       r.max_js_execs_month,
            max_emails_month:         r.max_emails_month,
            max_sms_month:            r.max_sms_month,
            max_alarms:               r.max_alarms,
            max_api_keys:             r.max_api_keys,
            feature_white_label:      r.feature_white_label,
            feature_edge_computing:   r.feature_edge_computing,
            feature_advanced_rbac:    r.feature_advanced_rbac,
            feature_audit_log:        r.feature_audit_log,
            feature_sso:              r.feature_sso,
            feature_api_export:       r.feature_api_export,
            sort_order:               r.sort_order,
            is_active:                r.is_active,
        }))
    }
}

// ── TenantSubscriptionDao ─────────────────────────────────────────────────────

pub struct TenantSubscriptionDao {
    pool: PgPool,
}

impl TenantSubscriptionDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find the active subscription for a tenant (at most one per UNIQUE constraint).
    pub async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantSubscription>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, updated_time, tenant_id, plan_id,
                      stripe_customer_id, stripe_subscription_id, billing_cycle, status,
                      current_period_start, current_period_end, trial_end,
                      cancel_at_period_end, canceled_at
               FROM tenant_subscription
               WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TenantSubscription {
            id:                     r.id,
            created_time:           r.created_time,
            updated_time:           r.updated_time,
            tenant_id:              r.tenant_id,
            plan_id:                r.plan_id,
            stripe_customer_id:     r.stripe_customer_id,
            stripe_subscription_id: r.stripe_subscription_id,
            billing_cycle:          r.billing_cycle,
            status:                 r.status,
            current_period_start:   r.current_period_start,
            current_period_end:     r.current_period_end,
            trial_end:              r.trial_end,
            cancel_at_period_end:   r.cancel_at_period_end,
            canceled_at:            r.canceled_at,
        }))
    }

    /// Find a subscription by Stripe customer ID (for webhook processing).
    pub async fn find_by_stripe_customer(
        &self,
        cust_id: &str,
    ) -> Result<Option<TenantSubscription>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, updated_time, tenant_id, plan_id,
                      stripe_customer_id, stripe_subscription_id, billing_cycle, status,
                      current_period_start, current_period_end, trial_end,
                      cancel_at_period_end, canceled_at
               FROM tenant_subscription
               WHERE stripe_customer_id = $1"#,
            cust_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TenantSubscription {
            id:                     r.id,
            created_time:           r.created_time,
            updated_time:           r.updated_time,
            tenant_id:              r.tenant_id,
            plan_id:                r.plan_id,
            stripe_customer_id:     r.stripe_customer_id,
            stripe_subscription_id: r.stripe_subscription_id,
            billing_cycle:          r.billing_cycle,
            status:                 r.status,
            current_period_start:   r.current_period_start,
            current_period_end:     r.current_period_end,
            trial_end:              r.trial_end,
            cancel_at_period_end:   r.cancel_at_period_end,
            canceled_at:            r.canceled_at,
        }))
    }

    /// Find a subscription by Stripe subscription ID (for webhook processing).
    pub async fn find_by_stripe_sub_id(
        &self,
        sub_id: &str,
    ) -> Result<Option<TenantSubscription>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT id, created_time, updated_time, tenant_id, plan_id,
                      stripe_customer_id, stripe_subscription_id, billing_cycle, status,
                      current_period_start, current_period_end, trial_end,
                      cancel_at_period_end, canceled_at
               FROM tenant_subscription
               WHERE stripe_subscription_id = $1"#,
            sub_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TenantSubscription {
            id:                     r.id,
            created_time:           r.created_time,
            updated_time:           r.updated_time,
            tenant_id:              r.tenant_id,
            plan_id:                r.plan_id,
            stripe_customer_id:     r.stripe_customer_id,
            stripe_subscription_id: r.stripe_subscription_id,
            billing_cycle:          r.billing_cycle,
            status:                 r.status,
            current_period_start:   r.current_period_start,
            current_period_end:     r.current_period_end,
            trial_end:              r.trial_end,
            cancel_at_period_end:   r.cancel_at_period_end,
            canceled_at:            r.canceled_at,
        }))
    }

    /// Insert or update a full `TenantSubscription` record (keyed on tenant_id).
    pub async fn upsert(&self, sub: &TenantSubscription) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            r#"INSERT INTO tenant_subscription (
                   id, created_time, updated_time, tenant_id, plan_id,
                   stripe_customer_id, stripe_subscription_id, billing_cycle, status,
                   current_period_start, current_period_end, trial_end,
                   cancel_at_period_end, canceled_at
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (tenant_id) DO UPDATE SET
                   updated_time           = $3,
                   plan_id                = EXCLUDED.plan_id,
                   stripe_customer_id     = EXCLUDED.stripe_customer_id,
                   stripe_subscription_id = EXCLUDED.stripe_subscription_id,
                   billing_cycle          = EXCLUDED.billing_cycle,
                   status                 = EXCLUDED.status,
                   current_period_start   = EXCLUDED.current_period_start,
                   current_period_end     = EXCLUDED.current_period_end,
                   trial_end              = EXCLUDED.trial_end,
                   cancel_at_period_end   = EXCLUDED.cancel_at_period_end,
                   canceled_at            = EXCLUDED.canceled_at"#,
            sub.id,
            sub.created_time,
            now,
            sub.tenant_id,
            sub.plan_id,
            sub.stripe_customer_id,
            sub.stripe_subscription_id,
            sub.billing_cycle,
            sub.status,
            sub.current_period_start,
            sub.current_period_end,
            sub.trial_end,
            sub.cancel_at_period_end,
            sub.canceled_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lightweight status update — used by webhook handler after payment events.
    pub async fn update_status(
        &self,
        tenant_id: Uuid,
        status: &str,
        period_end: Option<i64>,
        cancel_at_period_end: bool,
    ) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            r#"UPDATE tenant_subscription
               SET status               = $2,
                   current_period_end   = $3,
                   cancel_at_period_end = $4,
                   updated_time         = $5
               WHERE tenant_id = $1"#,
            tenant_id,
            status,
            period_end,
            cancel_at_period_end,
            now,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Count subscriptions grouped by status — for the admin dashboard.
    pub async fn count_by_status(&self) -> Result<Vec<(String, i64)>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT status, COUNT(*) AS cnt
               FROM tenant_subscription
               GROUP BY status"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.status, r.cnt.unwrap_or(0)))
            .collect())
    }

    /// Compute Monthly Recurring Revenue (MRR) in USD cents across all active/trialing tenants.
    pub async fn mrr_cents(&self) -> Result<i64, DaoError> {
        let val = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(CASE ts.billing_cycle
                   WHEN 'monthly' THEN sp.price_monthly
                   WHEN 'annual'  THEN sp.price_annually / 12
                   ELSE 0 END), 0)::bigint
               FROM tenant_subscription ts
               JOIN subscription_plan sp ON sp.id = ts.plan_id
               WHERE ts.status IN ('active', 'trialing')
                 AND sp.price_monthly > 0"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(val.unwrap_or(0))
    }

    // ── Stripe event idempotency ──────────────────────────────────────────────

    /// Return true if this Stripe event ID has already been received.
    pub async fn stripe_event_exists(&self, event_id: &str) -> Result<bool, DaoError> {
        let cnt = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM stripe_event WHERE stripe_event_id = $1",
            event_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(cnt.unwrap_or(0) > 0)
    }

    /// Persist a newly received Stripe event for idempotency tracking.
    pub async fn insert_stripe_event(
        &self,
        event_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
        received_time: i64,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"INSERT INTO stripe_event (stripe_event_id, event_type, payload, received_time)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (stripe_event_id) DO NOTHING"#,
            event_id,
            event_type,
            payload,
            received_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark a previously received Stripe event as processed (or failed with an error message).
    pub async fn mark_stripe_event_processed(
        &self,
        event_id: &str,
        error: Option<&str>,
    ) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query!(
            r#"UPDATE stripe_event
               SET processed      = true,
                   processed_time = $2,
                   error          = $3
               WHERE stripe_event_id = $1"#,
            event_id,
            now,
            error,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
