-- VieLang Phase 63 — Missing junction and settings tables
-- Matches ThingsBoard Java schema-entities.sql

-- ── Entity Alarm (junction: entity ↔ alarm) ───────────────────────────────────
CREATE TABLE IF NOT EXISTS entity_alarm (
    tenant_id        UUID        NOT NULL,
    entity_type      VARCHAR(32),
    entity_id        UUID        NOT NULL,
    created_time     BIGINT      NOT NULL,
    alarm_type       VARCHAR(255) NOT NULL,
    customer_id      UUID,
    alarm_id         UUID        REFERENCES alarm(id) ON DELETE CASCADE,
    CONSTRAINT entity_alarm_pkey PRIMARY KEY (entity_id, alarm_id)
);

CREATE INDEX IF NOT EXISTS idx_entity_alarm_tenant_entity
    ON entity_alarm (tenant_id, entity_type, entity_id);

CREATE INDEX IF NOT EXISTS idx_entity_alarm_alarm_type
    ON entity_alarm (tenant_id, alarm_type);

CREATE INDEX IF NOT EXISTS idx_entity_alarm_created
    ON entity_alarm (tenant_id, created_time DESC);

-- ── Alarm Types (per-tenant alarm type registry) ──────────────────────────────
CREATE TABLE IF NOT EXISTS alarm_types (
    tenant_id        UUID        NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    type             VARCHAR(255) NOT NULL,
    CONSTRAINT alarm_types_unq_key UNIQUE (tenant_id, type)
);

-- ── User Auth Settings (2FA configuration per user) ───────────────────────────
CREATE TABLE IF NOT EXISTS user_auth_settings (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    user_id          UUID        UNIQUE NOT NULL REFERENCES tb_user(id) ON DELETE CASCADE,
    two_fa_settings  VARCHAR
);

-- ── User Settings (general per-user settings by type) ─────────────────────────
CREATE TABLE IF NOT EXISTS user_settings (
    user_id          UUID        NOT NULL REFERENCES tb_user(id) ON DELETE CASCADE,
    type             VARCHAR(50) NOT NULL,
    settings         JSONB,
    CONSTRAINT user_settings_pkey PRIMARY KEY (user_id, type)
);

-- ── Schema Settings (database schema version tracking) ────────────────────────
CREATE TABLE IF NOT EXISTS tb_schema_settings (
    schema_version   BIGINT      NOT NULL,
    product          VARCHAR(2)  NOT NULL,
    CONSTRAINT tb_schema_settings_pkey PRIMARY KEY (schema_version)
);

-- Seed current schema version
INSERT INTO tb_schema_settings (schema_version, product)
VALUES (63, 'VL')
ON CONFLICT (schema_version) DO NOTHING;

-- ── Mobile App Bundle ↔ OAuth2 Client (junction) ──────────────────────────────
CREATE TABLE IF NOT EXISTS mobile_app_bundle_oauth2_client (
    mobile_app_bundle_id UUID NOT NULL REFERENCES mobile_app_bundle(id) ON DELETE CASCADE,
    oauth2_client_id     UUID NOT NULL,
    CONSTRAINT mobile_app_bundle_oauth2_pkey PRIMARY KEY (mobile_app_bundle_id, oauth2_client_id)
);

CREATE INDEX IF NOT EXISTS idx_mobile_bundle_oauth2_bundle
    ON mobile_app_bundle_oauth2_client (mobile_app_bundle_id);
