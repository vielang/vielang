-- Phase 16: Advanced Security
-- oauth2_client_registration, two_factor_auth_settings, api_key, audit_log

CREATE TABLE IF NOT EXISTS oauth2_client_registration (
    id                  UUID PRIMARY KEY,
    created_time        BIGINT NOT NULL,
    tenant_id           UUID NOT NULL,
    provider_name       VARCHAR(50) NOT NULL,
    client_id           VARCHAR(512) NOT NULL,
    client_secret       VARCHAR(1024) NOT NULL,
    authorization_uri   VARCHAR(1024) NOT NULL,
    token_uri           VARCHAR(1024) NOT NULL,
    user_info_uri       VARCHAR(1024) NOT NULL,
    scope               JSONB NOT NULL DEFAULT '[]',
    user_name_attribute VARCHAR(100) NOT NULL DEFAULT 'email',
    mapper_config       JSONB NOT NULL DEFAULT '{}',
    enabled             BOOLEAN NOT NULL DEFAULT true
);
CREATE INDEX IF NOT EXISTS idx_oauth2_tenant ON oauth2_client_registration(tenant_id);

CREATE TABLE IF NOT EXISTS two_factor_auth_settings (
    user_id      UUID PRIMARY KEY,
    provider     VARCHAR(20) NOT NULL DEFAULT 'TOTP',
    enabled      BOOLEAN NOT NULL DEFAULT false,
    secret       VARCHAR(512) NOT NULL DEFAULT '',
    backup_codes JSONB NOT NULL DEFAULT '[]',
    verified     BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS api_key (
    id           UUID PRIMARY KEY,
    created_time BIGINT NOT NULL,
    tenant_id    UUID NOT NULL,
    user_id      UUID NOT NULL,
    name         VARCHAR(255) NOT NULL,
    key_hash     VARCHAR(64) NOT NULL,
    key_prefix   VARCHAR(30) NOT NULL,
    scopes       JSONB NOT NULL DEFAULT '[]',
    expires_at   BIGINT,
    last_used_at BIGINT,
    enabled      BOOLEAN NOT NULL DEFAULT true,
    CONSTRAINT api_key_hash_unique UNIQUE (key_hash)
);
CREATE INDEX IF NOT EXISTS idx_api_key_tenant ON api_key(tenant_id);
CREATE INDEX IF NOT EXISTS idx_api_key_user ON api_key(user_id);

CREATE TABLE IF NOT EXISTS audit_log (
    id                     UUID PRIMARY KEY,
    created_time           BIGINT NOT NULL,
    tenant_id              UUID NOT NULL,
    user_id                UUID,
    user_name              VARCHAR(255),
    action_type            VARCHAR(50) NOT NULL,
    action_data            JSONB NOT NULL DEFAULT '{}',
    action_status          VARCHAR(20) NOT NULL DEFAULT 'SUCCESS',
    action_failure_details TEXT,
    entity_type            VARCHAR(50),
    entity_id              UUID,
    entity_name            VARCHAR(255)
);
CREATE INDEX IF NOT EXISTS idx_audit_tenant_time ON audit_log(tenant_id, created_time DESC);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);
