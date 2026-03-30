use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use vl_core::entities::{
    SaasDashboard, PlanRevenue, MonthlyTenantHealth, TenantUsageSummary,
};

use crate::error::DaoError;

pub struct AnalyticsDao {
    pool:  PgPool,
    /// 60-second in-process cache for the expensive dashboard snapshot.
    cache: Arc<RwLock<Option<(i64, SaasDashboard)>>>,
}

impl AnalyticsDao {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    // ── MRR ──────────────────────────────────────────────────────────────────

    /// Monthly Recurring Revenue in USD cents.
    pub async fn mrr_cents(&self) -> Result<i64, DaoError> {
        let val = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(
                   CASE ts.billing_cycle
                       WHEN 'monthly' THEN sp.price_monthly
                       WHEN 'annual'  THEN sp.price_annually / 12
                       ELSE 0
                   END
               ), 0)::bigint
               FROM tenant_subscription ts
               JOIN subscription_plan sp ON sp.id = ts.plan_id
               WHERE ts.status IN ('active', 'trialing')
                 AND sp.price_monthly > 0"#
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(val.unwrap_or(0))
    }

    // ── Revenue by plan ───────────────────────────────────────────────────────

    /// Revenue breakdown by plan, including free-tier tenant counts.
    pub async fn revenue_by_plan(&self) -> Result<Vec<PlanRevenue>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT
                   sp.name,
                   sp.display_name,
                   COUNT(ts.id)::bigint                                     AS tenant_count,
                   COALESCE(SUM(
                       CASE ts.billing_cycle
                           WHEN 'monthly' THEN sp.price_monthly
                           WHEN 'annual'  THEN sp.price_annually / 12
                           ELSE 0
                       END
                   ), 0)::bigint                                            AS mrr_cents
               FROM subscription_plan sp
               LEFT JOIN tenant_subscription ts
                   ON ts.plan_id = sp.id AND ts.status IN ('active', 'trialing', 'free')
               WHERE sp.is_active = true
               GROUP BY sp.id, sp.name, sp.display_name, sp.sort_order
               ORDER BY sp.sort_order"#
        )
        .fetch_all(&self.pool)
        .await?;

        let total_mrr: i64 = rows.iter().map(|r| r.mrr_cents.unwrap_or(0)).sum();

        Ok(rows.into_iter().map(|r| {
            let mrr = r.mrr_cents.unwrap_or(0);
            let pct = if total_mrr > 0 {
                (mrr as f32 / total_mrr as f32) * 100.0
            } else {
                0.0
            };
            PlanRevenue {
                plan_name:    r.name,
                display_name: r.display_name,
                tenant_count: r.tenant_count.unwrap_or(0),
                mrr_cents:    mrr,
                pct_of_total: pct,
            }
        }).collect())
    }

    // ── Tenant health ─────────────────────────────────────────────────────────

    /// New tenants per month for the last 13 months.
    pub async fn tenant_health_monthly(&self) -> Result<Vec<MonthlyTenantHealth>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT
                   to_char(to_timestamp(t.created_time / 1000), 'YYYY-MM') AS month,
                   COUNT(*)::bigint                                         AS new_tenants
               FROM tenant t
               WHERE t.created_time >= extract(epoch from now() - interval '13 months')::bigint * 1000
               GROUP BY month
               ORDER BY month"#
        )
        .fetch_all(&self.pool)
        .await?;

        // Churn = subscriptions canceled in that month
        let churn_rows = sqlx::query!(
            r#"SELECT
                   to_char(to_timestamp(canceled_at / 1000), 'YYYY-MM') AS month,
                   COUNT(*)::bigint                                       AS churned
               FROM tenant_subscription
               WHERE canceled_at IS NOT NULL
                 AND canceled_at >= extract(epoch from now() - interval '13 months')::bigint * 1000
               GROUP BY month"#
        )
        .fetch_all(&self.pool)
        .await?;

        // Merge new + churn by month
        use std::collections::HashMap;
        let mut map: HashMap<String, (i64, i64)> = HashMap::new();
        for r in &rows {
            if let Some(m) = &r.month {
                map.entry(m.clone()).or_default().0 = r.new_tenants.unwrap_or(0);
            }
        }
        for r in &churn_rows {
            if let Some(m) = &r.month {
                map.entry(m.clone()).or_default().1 = r.churned.unwrap_or(0);
            }
        }

        let mut result: Vec<MonthlyTenantHealth> = map.into_iter().map(|(month, (new, churned))| {
            MonthlyTenantHealth { month, new_tenants: new, churned_tenants: churned }
        }).collect();
        result.sort_by(|a, b| a.month.cmp(&b.month));
        Ok(result)
    }

    // ── Churn rate ────────────────────────────────────────────────────────────

    /// Churn rate for the current month: canceled_this_month / active_at_start_of_month.
    pub async fn churn_rate_current_month(&self) -> Result<f32, DaoError> {
        let period_start_ms = {
            let now = chrono::Utc::now();
            use chrono::{TimeZone, Datelike};
            chrono::Utc.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                .unwrap()
                .timestamp_millis()
        };

        let canceled = sqlx::query_scalar!(
            "SELECT COUNT(*)::bigint FROM tenant_subscription WHERE status = 'canceled' AND canceled_at >= $1",
            period_start_ms
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let active_at_start = sqlx::query_scalar!(
            "SELECT COUNT(*)::bigint FROM tenant_subscription WHERE created_time < $1 AND status IN ('active', 'trialing', 'past_due')",
            period_start_ms
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        if active_at_start == 0 {
            return Ok(0.0);
        }
        Ok(canceled as f32 / active_at_start as f32)
    }

    // ── Top tenants ───────────────────────────────────────────────────────────

    /// Top N tenants by device count, joined with their plan name and MRR.
    pub async fn top_tenants_by_devices(&self, limit: i64) -> Result<Vec<TenantUsageSummary>, DaoError> {
        let period = chrono::Utc::now().format("%Y-%m").to_string();
        let rows = sqlx::query!(
            r#"SELECT
                   t.id                                   AS tenant_id,
                   t.title                                AS tenant_name,
                   COALESCE(sp.name, 'free')              AS plan_name,
                   COUNT(DISTINCT d.id)::bigint           AS device_count,
                   COALESCE(aus.transport_msg_count, 0)   AS transport_msgs,
                   COALESCE(CASE ts.billing_cycle
                       WHEN 'monthly' THEN sp.price_monthly
                       WHEN 'annual'  THEN sp.price_annually / 12
                       ELSE 0
                   END, 0)                                AS mrr_cents
               FROM tenant t
               LEFT JOIN device d ON d.tenant_id = t.id
               LEFT JOIN tenant_subscription ts ON ts.tenant_id = t.id
               LEFT JOIN subscription_plan sp ON sp.id = ts.plan_id
               LEFT JOIN api_usage_state aus ON aus.tenant_id = t.id AND aus.billing_period = $1
               GROUP BY t.id, t.title, sp.name, ts.billing_cycle, sp.price_monthly, sp.price_annually, aus.transport_msg_count
               ORDER BY device_count DESC
               LIMIT $2"#,
            period,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TenantUsageSummary {
            tenant_id:    r.tenant_id,
            tenant_name:  r.tenant_name,
            plan_name:    r.plan_name.unwrap_or_else(|| "free".to_string()),
            device_count: r.device_count.unwrap_or(0),
            transport_msgs_this_month: r.transport_msgs.unwrap_or(0),
            mrr_cents:    r.mrr_cents.unwrap_or(0) as i64,
        }).collect())
    }

    // ── Tenant counts ─────────────────────────────────────────────────────────

    pub async fn tenant_counts(&self) -> Result<(i64, i64, i64, i64), DaoError> {
        // (total, active, free, past_due)
        let total = sqlx::query_scalar!("SELECT COUNT(*)::bigint FROM tenant")
            .fetch_one(&self.pool).await?.unwrap_or(0);

        let active = sqlx::query_scalar!(
            "SELECT COUNT(*)::bigint FROM tenant_subscription WHERE status IN ('active', 'trialing')"
        ).fetch_one(&self.pool).await?.unwrap_or(0);

        let free = sqlx::query_scalar!(
            "SELECT COUNT(*)::bigint FROM tenant_subscription WHERE status = 'free'"
        ).fetch_one(&self.pool).await?.unwrap_or(0);

        let past_due = sqlx::query_scalar!(
            "SELECT COUNT(*)::bigint FROM tenant_subscription WHERE status = 'past_due'"
        ).fetch_one(&self.pool).await?.unwrap_or(0);

        Ok((total, active, free, past_due))
    }

    // ── Full dashboard snapshot (cached 60s) ──────────────────────────────────

    pub async fn dashboard_snapshot(&self) -> Result<SaasDashboard, DaoError> {
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Check cache
        {
            let guard = self.cache.read().await;
            if let Some((cached_at, ref snap)) = *guard {
                if now_ms - cached_at < 60_000 {
                    return Ok(snap.clone());
                }
            }
        }

        // Compute fresh snapshot
        let mrr = self.mrr_cents().await?;
        let revenue_by_plan = self.revenue_by_plan().await?;
        let tenant_health   = self.tenant_health_monthly().await?;
        let top_tenants     = self.top_tenants_by_devices(20).await?;
        let churn_rate      = self.churn_rate_current_month().await?;
        let (total, active, free, past_due) = self.tenant_counts().await?;

        let snap = SaasDashboard {
            generated_at:     now_ms,
            mrr_cents:        mrr,
            arr_cents:        mrr * 12,
            total_tenants:    total,
            active_tenants:   active,
            free_tenants:     free,
            past_due_tenants: past_due,
            churn_rate,
            revenue_by_plan,
            tenant_health,
            top_tenants,
        };

        // Update cache
        {
            let mut guard = self.cache.write().await;
            *guard = Some((now_ms, snap.clone()));
        }

        Ok(snap)
    }
}
