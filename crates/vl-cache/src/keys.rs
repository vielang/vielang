use uuid::Uuid;

// ── Cache name constants (khớp ThingsBoard CacheConstants) ───────────────────

pub const DEVICE_CACHE:       &str = "devices";
pub const CREDENTIALS_CACHE:  &str = "deviceCredentials";
pub const TENANT_CACHE:       &str = "tenants";
pub const SESSIONS_CACHE:     &str = "sessions";

// ── Key builders ──────────────────────────────────────────────────────────────

/// Key for device by id: "devices:{uuid}"
pub fn device_key(id: &Uuid) -> String {
    format!("{}:{}", DEVICE_CACHE, id)
}

/// Key for credentials by token/username: "deviceCredentials:{token}"
pub fn credentials_key(token: &str) -> String {
    format!("{}:{}", CREDENTIALS_CACHE, token)
}

/// Key for tenant by id: "tenants:{uuid}"
pub fn tenant_key(id: &Uuid) -> String {
    format!("{}:{}", TENANT_CACHE, id)
}

/// Key for device session: "sessions:{uuid}"
pub fn session_key(device_id: &Uuid) -> String {
    format!("{}:{}", SESSIONS_CACHE, device_id)
}
