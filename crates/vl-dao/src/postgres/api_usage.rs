use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{ApiUsageHistory, SubscriptionPlan, TenantApiUsage, UsageMetric};

use crate::error::DaoError;

// ── Internal row struct ───────────────────────────────────────────────────────

pub struct ApiUsageStateRow {
    pub id:                    Uuid,
    pub tenant_id:             Uuid,
    pub billing_period:        String,
    pub transport_msg_count:   i64,
    pub transport_dp_count:    i64,
    pub re_exec_count:         i64,
    pub js_exec_count:         i64,
    pub email_count:           i32,
    pub sms_count:             i32,
    pub alarm_count:           i32,
    pub active_device_count:   i32,
    // P12 limit columns
    pub transport_msg_limit:   i64,
    pub transport_dp_limit:    i64,
    pub re_exec_limit:         i64,
    pub js_exec_limit:         i64,
    pub email_limit:           i64,
    pub sms_limit:             i64,
    pub alarm_limit:           i64,
    pub active_device_limit:   i64,
    // P12 new counters + limits
    pub storage_dp_count:      i64,
    pub storage_dp_limit:      i64,
    pub rpc_count:             i64,
    pub rpc_limit:             i64,
    pub rule_engine_exec_count: i64,
    pub rule_engine_exec_limit: i64,
    pub created_time:          i64,
    pub updated_time:          i64,
}

// ── ApiUsageDao ───────────────────────────────────────────────────────────────

pub struct ApiUsageDao {
    pool: PgPool,
}

impl ApiUsageDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find the usage record for a tenant in the current billing period.
    pub async fn find_current(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<ApiUsageStateRow>, DaoError> {
        let period = current_period();
        let row = sqlx::query(
            r#"SELECT id, tenant_id, billing_period,
                      transport_msg_count, transport_dp_count,
                      re_exec_count, js_exec_count,
                      email_count, sms_count, alarm_count, active_device_count,
                      transport_msg_limit, transport_dp_limit,
                      re_exec_limit, js_exec_limit,
                      email_limit, sms_limit, alarm_limit, active_device_limit,
                      storage_dp_count, storage_dp_limit,
                      rpc_count, rpc_limit,
                      rule_engine_exec_count, rule_engine_exec_limit,
                      created_time, updated_time
               FROM api_usage_state
               WHERE tenant_id = $1 AND billing_period = $2"#,
        )
        .bind(tenant_id)
        .bind(&period)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(map_row))
    }

    /// Atomically increment usage counters for a tenant in the given billing period.
    ///
    /// Uses INSERT … ON CONFLICT DO UPDATE with additive increments — safe for
    /// concurrent calls from multiple transport/rule-engine workers.
    pub async fn increment(
        &self,
        tenant_id: Uuid,
        period: &str,
        transport_msg: i64,
        transport_dp: i64,
        re_exec: i64,
        js_exec: i64,
        email: i32,
        sms: i32,
        alarm: i32,
        storage_dp: i64,
        rpc: i64,
        rule_engine_exec: i64,
    ) -> Result<(), DaoError> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            r#"INSERT INTO api_usage_state
                   (id, tenant_id, billing_period,
                    transport_msg_count, transport_dp_count,
                    re_exec_count, js_exec_count,
                    email_count, sms_count, alarm_count,
                    active_device_count,
                    storage_dp_count, rpc_count, rule_engine_exec_count,
                    created_time, updated_time)
               VALUES
                   (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, 0,
                    $10, $11, $12, $13, $13)
               ON CONFLICT (tenant_id, billing_period) DO UPDATE SET
                   transport_msg_count     = api_usage_state.transport_msg_count     + EXCLUDED.transport_msg_count,
                   transport_dp_count      = api_usage_state.transport_dp_count      + EXCLUDED.transport_dp_count,
                   re_exec_count           = api_usage_state.re_exec_count           + EXCLUDED.re_exec_count,
                   js_exec_count           = api_usage_state.js_exec_count           + EXCLUDED.js_exec_count,
                   email_count             = api_usage_state.email_count             + EXCLUDED.email_count,
                   sms_count               = api_usage_state.sms_count               + EXCLUDED.sms_count,
                   alarm_count             = api_usage_state.alarm_count             + EXCLUDED.alarm_count,
                   storage_dp_count        = api_usage_state.storage_dp_count        + EXCLUDED.storage_dp_count,
                   rpc_count               = api_usage_state.rpc_count               + EXCLUDED.rpc_count,
                   rule_engine_exec_count  = api_usage_state.rule_engine_exec_count  + EXCLUDED.rule_engine_exec_count,
                   updated_time            = EXCLUDED.updated_time"#,
        )
        .bind(tenant_id)
        .bind(period)
        .bind(transport_msg)
        .bind(transport_dp)
        .bind(re_exec)
        .bind(js_exec)
        .bind(email)
        .bind(sms)
        .bind(alarm)
        .bind(storage_dp)
        .bind(rpc)
        .bind(rule_engine_exec)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Increment a single counter by 1 for the current billing period.
    pub async fn increment_one(
        &self,
        tenant_id: Uuid,
        counter_col: &str,
    ) -> Result<(), DaoError> {
        let period = current_period();
        let now = chrono::Utc::now().timestamp_millis();
        // Build the query dynamically for the named column.
        // counter_col must be a trusted internal enum-derived string — not user input.
        let sql = format!(
            r#"INSERT INTO api_usage_state
                   (id, tenant_id, billing_period, {col}, created_time, updated_time)
               VALUES (gen_random_uuid(), $1, $2, 1, $3, $3)
               ON CONFLICT (tenant_id, billing_period) DO UPDATE SET
                   {col} = api_usage_state.{col} + 1,
                   updated_time = EXCLUDED.updated_time"#,
            col = counter_col,
        );
        sqlx::query(&sql)
            .bind(tenant_id)
            .bind(&period)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Archive the current period's counters to `api_usage_history` and reset them to 0.
    ///
    /// Called at the start of a new billing period (e.g. by a monthly cron job).
    pub async fn reset_period(
        &self,
        tenant_id: Uuid,
    ) -> Result<(), DaoError> {
        let period = current_period();
        let now = chrono::Utc::now().timestamp_millis();

        // 1. Fetch current counters as JSON snapshot
        let snapshot: Option<serde_json::Value> = sqlx::query_scalar(
            r#"SELECT row_to_json(t) FROM (
                   SELECT transport_msg_count, transport_dp_count,
                          re_exec_count, js_exec_count,
                          email_count, sms_count, alarm_count, active_device_count,
                          storage_dp_count, rpc_count, rule_engine_exec_count
                   FROM api_usage_state
                   WHERE tenant_id = $1 AND billing_period = $2
               ) t"#,
        )
        .bind(tenant_id)
        .bind(&period)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(counters) = snapshot {
            // 2. Determine period window (first/last millisecond of the month)
            let (period_start, period_end) = period_range_ms(&period);

            // 3. Insert history row
            sqlx::query(
                r#"INSERT INTO api_usage_history
                       (id, tenant_id, period_start, period_end, counters, created_time)
                   VALUES (gen_random_uuid(), $1, $2, $3, $4, $5)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(tenant_id)
            .bind(period_start)
            .bind(period_end)
            .bind(&counters)
            .bind(now)
            .execute(&self.pool)
            .await?;

            // 4. Zero out all counters for the tenant-period
            sqlx::query(
                r#"UPDATE api_usage_state SET
                       transport_msg_count    = 0,
                       transport_dp_count     = 0,
                       re_exec_count          = 0,
                       js_exec_count          = 0,
                       email_count            = 0,
                       sms_count              = 0,
                       alarm_count            = 0,
                       active_device_count    = 0,
                       storage_dp_count       = 0,
                       rpc_count              = 0,
                       rule_engine_exec_count = 0,
                       updated_time           = $3
                   WHERE tenant_id = $1 AND billing_period = $2"#,
            )
            .bind(tenant_id)
            .bind(&period)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Retrieve paginated usage history for a tenant, newest first.
    pub async fn get_history(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiUsageHistory>, DaoError> {
        let rows = sqlx::query(
            r#"SELECT id, tenant_id, period_start, period_end, counters, created_time
               FROM api_usage_history
               WHERE tenant_id = $1
               ORDER BY period_start DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        use sqlx::Row;
        Ok(rows
            .into_iter()
            .map(|r| ApiUsageHistory {
                id:           r.get("id"),
                tenant_id:    r.get("tenant_id"),
                period_start: r.get("period_start"),
                period_end:   r.get("period_end"),
                counters:     r.get("counters"),
                created_time: r.get("created_time"),
            })
            .collect())
    }

    /// Retrieve all usage for the given billing period as domain objects (admin view).
    ///
    /// Uses unlimited (-1) limits so the caller can see raw counts without plan caps.
    pub async fn get_all_usage_domain(
        &self,
        period: &str,
    ) -> Result<Vec<TenantApiUsage>, DaoError> {
        let rows = self.get_all_current_period(period).await?;
        let plan = unlimited_plan();
        Ok(rows.iter().map(|r| Self::to_domain(r, &plan)).collect())
    }

    /// Retrieve all usage records for the given billing period — for admin analytics.
    pub async fn get_all_current_period(
        &self,
        period: &str,
    ) -> Result<Vec<ApiUsageStateRow>, DaoError> {
        let rows = sqlx::query(
            r#"SELECT id, tenant_id, billing_period,
                      transport_msg_count, transport_dp_count,
                      re_exec_count, js_exec_count,
                      email_count, sms_count, alarm_count, active_device_count,
                      transport_msg_limit, transport_dp_limit,
                      re_exec_limit, js_exec_limit,
                      email_limit, sms_limit, alarm_limit, active_device_limit,
                      storage_dp_count, storage_dp_limit,
                      rpc_count, rpc_limit,
                      rule_engine_exec_count, rule_engine_exec_limit,
                      created_time, updated_time
               FROM api_usage_state
               WHERE billing_period = $1
               ORDER BY tenant_id"#,
        )
        .bind(period)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(map_row).collect())
    }

    /// Fetch usage for a tenant and convert to domain type in one call.
    /// Returns `None` if no usage record exists yet for the current period.
    pub async fn get_usage_domain(
        &self,
        tenant_id: Uuid,
        plan: &SubscriptionPlan,
    ) -> Result<Option<TenantApiUsage>, DaoError> {
        Ok(self.find_current(tenant_id).await?.map(|row| Self::to_domain(&row, plan)))
    }

    /// Convert an `ApiUsageStateRow` + a `SubscriptionPlan` into the public
    /// `TenantApiUsage` domain struct, computing per-metric state and percentage.
    pub fn to_domain(row: &ApiUsageStateRow, plan: &SubscriptionPlan) -> TenantApiUsage {
        // Prefer DB limit columns (set per-tenant); fall back to plan defaults.
        let msg_limit = if row.transport_msg_limit >= 0 { row.transport_msg_limit }
                        else { plan.max_transport_msgs_month };
        let dp_limit  = if row.transport_dp_limit >= 0 { row.transport_dp_limit }
                        else { plan.max_transport_msgs_month };
        let re_limit  = if row.re_exec_limit >= 0 { row.re_exec_limit }
                        else { plan.max_js_execs_month };
        let js_limit  = if row.js_exec_limit >= 0 { row.js_exec_limit }
                        else { plan.max_js_execs_month };
        let em_limit  = if row.email_limit >= 0 { row.email_limit }
                        else { plan.max_emails_month as i64 };
        let sms_limit = if row.sms_limit >= 0 { row.sms_limit }
                        else { plan.max_sms_month as i64 };
        let alm_limit = if row.alarm_limit >= 0 { row.alarm_limit }
                        else { plan.max_alarms as i64 };
        let dev_limit = if row.active_device_limit >= 0 { row.active_device_limit }
                        else { plan.max_devices as i64 };

        TenantApiUsage {
            tenant_id:        row.tenant_id,
            billing_period:   row.billing_period.clone(),
            transport_msg:    UsageMetric::new("TRANSPORT_MSG",  row.transport_msg_count,   msg_limit),
            transport_dp:     UsageMetric::new("TRANSPORT_DP",   row.transport_dp_count,    dp_limit),
            re_exec:          UsageMetric::new("RE_EXEC",         row.re_exec_count,         re_limit),
            js_exec:          UsageMetric::new("JS_EXEC",         row.js_exec_count,         js_limit),
            email:            UsageMetric::new("EMAIL",           row.email_count as i64,    em_limit),
            sms:              UsageMetric::new("SMS",             row.sms_count as i64,      sms_limit),
            alarm:            UsageMetric::new("ALARM",           row.alarm_count as i64,    alm_limit),
            active_devices:   UsageMetric::new("ACTIVE_DEVICES",  row.active_device_count as i64, dev_limit),
            // P12: DB-stored limits (-1 = unlimited by default)
            storage_dp:       UsageMetric::new("STORAGE_DP",      row.storage_dp_count,      row.storage_dp_limit),
            rpc:              UsageMetric::new("RPC",             row.rpc_count,             row.rpc_limit),
            rule_engine_exec: UsageMetric::new("RULE_ENGINE_EXEC", row.rule_engine_exec_count, row.rule_engine_exec_limit),
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Return the current billing period as "YYYY-MM".
fn current_period() -> String {
    chrono::Utc::now().format("%Y-%m").to_string()
}

/// Return (start_ms, end_ms) for a "YYYY-MM" period string.
fn period_range_ms(period: &str) -> (i64, i64) {
    use chrono::{NaiveDate, TimeZone, Utc};
    let parts: Vec<&str> = period.split('-').collect();
    let year:  i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(2024);
    let month: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let end_month = if month == 12 { 1 } else { month + 1 };
    let end_year  = if month == 12 { year + 1 } else { year };
    let end = NaiveDate::from_ymd_opt(end_year, end_month, 1).unwrap_or_default();

    let start_ms = Utc.from_utc_datetime(&start.and_hms_opt(0, 0, 0).unwrap_or_default()).timestamp_millis();
    let end_ms   = Utc.from_utc_datetime(&end.and_hms_opt(0, 0, 0).unwrap_or_default()).timestamp_millis() - 1;
    (start_ms, end_ms)
}

/// A synthetic plan with all limits set to -1 (unlimited) for raw admin views.
fn unlimited_plan() -> SubscriptionPlan {
    SubscriptionPlan {
        id:                       uuid::Uuid::nil(),
        created_time:             0,
        name:                     "unlimited".into(),
        display_name:             "Unlimited".into(),
        description:              None,
        price_monthly:            0,
        price_annually:           0,
        stripe_price_id_monthly:  None,
        stripe_price_id_annually: None,
        max_devices:              -1,
        max_users:                -1,
        max_assets:               -1,
        max_dashboards:           -1,
        max_rule_chains:          -1,
        max_edges:                -1,
        max_transport_msgs_month: -1,
        max_js_execs_month:       -1,
        max_emails_month:         -1,
        max_sms_month:            -1,
        max_alarms:               -1,
        max_api_keys:             -1,
        feature_white_label:      true,
        feature_edge_computing:   true,
        feature_advanced_rbac:    true,
        feature_audit_log:        true,
        feature_sso:              true,
        feature_api_export:       true,
        sort_order:               0,
        is_active:                true,
    }
}

fn map_row(r: sqlx::postgres::PgRow) -> ApiUsageStateRow {
    use sqlx::Row;
    ApiUsageStateRow {
        id:                    r.get("id"),
        tenant_id:             r.get("tenant_id"),
        billing_period:        r.get("billing_period"),
        transport_msg_count:   r.get("transport_msg_count"),
        transport_dp_count:    r.get("transport_dp_count"),
        re_exec_count:         r.get("re_exec_count"),
        js_exec_count:         r.get("js_exec_count"),
        email_count:           r.get("email_count"),
        sms_count:             r.get("sms_count"),
        alarm_count:           r.get("alarm_count"),
        active_device_count:   r.get("active_device_count"),
        transport_msg_limit:   r.get("transport_msg_limit"),
        transport_dp_limit:    r.get("transport_dp_limit"),
        re_exec_limit:         r.get("re_exec_limit"),
        js_exec_limit:         r.get("js_exec_limit"),
        email_limit:           r.get("email_limit"),
        sms_limit:             r.get("sms_limit"),
        alarm_limit:           r.get("alarm_limit"),
        active_device_limit:   r.get("active_device_limit"),
        storage_dp_count:      r.get("storage_dp_count"),
        storage_dp_limit:      r.get("storage_dp_limit"),
        rpc_count:             r.get("rpc_count"),
        rpc_limit:             r.get("rpc_limit"),
        rule_engine_exec_count: r.get("rule_engine_exec_count"),
        rule_engine_exec_limit: r.get("rule_engine_exec_limit"),
        created_time:          r.get("created_time"),
        updated_time:          r.get("updated_time"),
    }
}
