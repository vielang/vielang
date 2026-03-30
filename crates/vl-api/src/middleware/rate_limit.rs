use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use uuid::Uuid;

use crate::{error::ApiError, middleware::auth::SecurityContext};

type DirectLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// In-memory rate limiter with per-tenant configurable limits.
///
/// On first request per tenant, the rate is loaded from the tenant's profile (via DB).
/// Limiters are kept in memory for the server lifetime — rate changes require restart.
pub struct TenantRateLimiter {
    /// Default requests/second when no per-tenant config is found
    pub default_limit: u32,
    /// Per-tenant limiters: tenant_id → (rate_per_sec, limiter)
    per_tenant: DashMap<Uuid, (u32, Arc<DirectLimiter>)>,
}

impl TenantRateLimiter {
    pub fn new(default_rps: u32) -> Self {
        Self {
            default_limit: default_rps,
            per_tenant: DashMap::new(),
        }
    }

    /// Get (or create) a per-tenant limiter with the given rate.
    /// If a limiter already exists for this tenant, the `rate` parameter is ignored
    /// (rate changes take effect only after a restart).
    fn get_or_create(&self, tenant_id: Uuid, rate: u32) -> (u32, Arc<DirectLimiter>) {
        if let Some(entry) = self.per_tenant.get(&tenant_id) {
            return (entry.0, entry.1.clone());
        }
        let rps = NonZeroU32::new(rate.max(1)).unwrap();
        let limiter = Arc::new(RateLimiter::direct(Quota::per_second(rps)));
        self.per_tenant.insert(tenant_id, (rate, limiter.clone()));
        (rate, limiter)
    }

    /// Check rate limit for tenant.  Returns `Ok(remaining)` or `Err(())` if limited.
    pub fn check_with_remaining(&self, tenant_id: Uuid, rate: u32) -> Result<u32, ()> {
        let (actual_rate, limiter) = self.get_or_create(tenant_id, rate);
        match limiter.check() {
            Ok(_)  => Ok(actual_rate.saturating_sub(1)),
            Err(_) => Err(()),
        }
    }

    /// True if this tenant already has a cached limiter (avoids async DB call).
    pub fn has_limiter(&self, tenant_id: &Uuid) -> bool {
        self.per_tenant.contains_key(tenant_id)
    }
}

/// Axum middleware — reads SecurityContext (set by auth_middleware) and enforces rate limit.
/// On first request per tenant, loads the tenant's configured rate from DB.
/// Adds X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset headers to all responses.
pub async fn rate_limit_middleware(
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Determine effective rate for this tenant
    let rate = if state.rate_limiter.has_limiter(&ctx.tenant_id) {
        // Already cached — use whatever rate was initially loaded
        state.rate_limiter.default_limit
    } else {
        // First request for this tenant — try to load custom rate from DB
        match state.tenant_profile_dao.find_rate_limit(ctx.tenant_id).await {
            Ok(Some(custom)) if custom > 0 => custom as u32,
            _ => state.rate_limiter.default_limit,
        }
    };

    match state.rate_limiter.check_with_remaining(ctx.tenant_id, rate) {
        Err(_) => {
            Err(ApiError::TooManyRequests(
                "Too many requests — please slow down".into(),
            ))
        }
        Ok(remaining) => {
            let mut response = next.run(request).await;
            let headers = response.headers_mut();

            if let Ok(v) = HeaderValue::from_str(&rate.to_string()) {
                headers.insert("x-ratelimit-limit", v);
            }
            if let Ok(v) = HeaderValue::from_str(&remaining.to_string()) {
                headers.insert("x-ratelimit-remaining", v);
            }
            if let Ok(v) = HeaderValue::from_str("1") {
                headers.insert("x-ratelimit-reset", v);
            }

            Ok(response)
        }
    }
}

/// Wrap in Arc for cheap cloning across threads.
pub type ArcRateLimiter = Arc<TenantRateLimiter>;
