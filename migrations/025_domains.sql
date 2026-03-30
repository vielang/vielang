-- Phase 54: Domain and OAuth2 template tables

CREATE TABLE IF NOT EXISTS domain (
    id                  UUID        PRIMARY KEY,
    created_time        BIGINT      NOT NULL,
    tenant_id           UUID        NOT NULL,
    name                VARCHAR(255) NOT NULL,
    oauth2_enabled      BOOLEAN     NOT NULL DEFAULT false,
    propagate_to_edge   BOOLEAN     NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_domain_tenant
    ON domain(tenant_id);
