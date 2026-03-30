-- Phase 29: Admin Settings table
-- Mirrors ThingsBoard admin_settings entity (key-value JSON storage)

CREATE TABLE IF NOT EXISTS admin_settings (
    id           UUID        PRIMARY KEY,
    created_time BIGINT      NOT NULL,
    tenant_id    UUID        NOT NULL,          -- SYS_TENANT_ID (nil UUID) for system-wide settings
    key          VARCHAR(64) NOT NULL,
    json_value   JSONB       NOT NULL DEFAULT '{}'
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_admin_settings_tenant_key
    ON admin_settings(tenant_id, key);
