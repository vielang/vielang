-- Phase 30: Mobile App Framework tables

CREATE TABLE IF NOT EXISTS mobile_app (
    id               UUID        PRIMARY KEY,
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    pkg_name         VARCHAR(255) NOT NULL,
    title            VARCHAR(255),
    app_secret       VARCHAR(2048) NOT NULL,
    platform_type    VARCHAR(16)  NOT NULL,  -- ANDROID | IOS
    status           VARCHAR(32)  NOT NULL DEFAULT 'DRAFT',
    version_info     JSONB,
    store_info       JSONB
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mobile_app_tenant_pkg_platform
    ON mobile_app(tenant_id, pkg_name, platform_type);

CREATE TABLE IF NOT EXISTS mobile_app_bundle (
    id                UUID        PRIMARY KEY,
    created_time      BIGINT      NOT NULL,
    tenant_id         UUID        NOT NULL,
    title             VARCHAR(255),
    android_app_id    UUID,
    ios_app_id        UUID,
    layout_config     JSONB,
    oauth2_client_ids JSONB       NOT NULL DEFAULT '[]'  -- array of UUIDs
);

CREATE INDEX IF NOT EXISTS idx_mobile_app_bundle_tenant ON mobile_app_bundle(tenant_id);

CREATE TABLE IF NOT EXISTS qr_code_settings (
    id                  UUID        PRIMARY KEY,
    created_time        BIGINT      NOT NULL,
    tenant_id           UUID        NOT NULL UNIQUE,
    use_system_settings BOOLEAN     NOT NULL DEFAULT false,
    use_default_app     BOOLEAN     NOT NULL DEFAULT true,
    mobile_app_bundle_id UUID,
    qr_code_config      JSONB       NOT NULL DEFAULT '{}',
    android_enabled     BOOLEAN     NOT NULL DEFAULT false,
    ios_enabled         BOOLEAN     NOT NULL DEFAULT false
);
