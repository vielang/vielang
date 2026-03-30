use async_trait::async_trait;
use uuid::Uuid;

/// Abstraction over external billing/payment providers (Stripe, Paddle, etc.).
///
/// The default implementation (`NoBillingProvider`) is a no-op stub used for
/// self-hosted deployments that don't need external billing.
#[async_trait]
pub trait BillingProvider: Send + Sync {
    /// Called when a tenant's usage has exceeded a plan limit.
    /// Implementations may notify the user, pause features, or log the event.
    async fn on_quota_exceeded(&self, tenant_id: Uuid, metric: &str);

    /// Called when a tenant transitions from Warning → Enabled (usage drops below 80%).
    async fn on_quota_restored(&self, tenant_id: Uuid, metric: &str);

    /// Called at the start of each billing period to reset metering state in
    /// the external provider (e.g. Stripe metered billing).
    async fn on_period_reset(&self, tenant_id: Uuid, period: &str);
}

/// No-op billing provider — used for self-hosted / open-core deployments.
pub struct NoBillingProvider;

#[async_trait]
impl BillingProvider for NoBillingProvider {
    async fn on_quota_exceeded(&self, _tenant_id: Uuid, _metric: &str) {}
    async fn on_quota_restored(&self, _tenant_id: Uuid, _metric: &str) {}
    async fn on_period_reset(&self, _tenant_id: Uuid, _period: &str) {}
}
