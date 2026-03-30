-- Phase 54: AI Model table
CREATE TABLE IF NOT EXISTS ai_model (
    id              UUID        PRIMARY KEY,
    created_time    BIGINT      NOT NULL,
    tenant_id       UUID,                       -- NULL = system-level
    name            VARCHAR(255) NOT NULL,
    configuration   JSONB,
    additional_info JSONB
);

CREATE INDEX IF NOT EXISTS idx_ai_model_tenant
    ON ai_model(tenant_id);
