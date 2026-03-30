-- Phase 22: Resource & Image Management

CREATE TABLE IF NOT EXISTS resource (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time        BIGINT NOT NULL,
    tenant_id           UUID,                           -- NULL = system resource
    title               VARCHAR(255) NOT NULL,
    resource_type       VARCHAR(32) NOT NULL,           -- IMAGE | JS_MODULE | JKS | PKCS_12 | LWM2M_MODEL | DASHBOARD | GENERAL
    resource_sub_type   VARCHAR(32),
    resource_key        VARCHAR(255) NOT NULL,
    file_name           VARCHAR(255) NOT NULL,
    is_public           BOOLEAN NOT NULL DEFAULT FALSE,
    public_resource_key VARCHAR(255),
    etag                VARCHAR(64),
    descriptor          JSONB,
    data                BYTEA,
    preview             BYTEA,
    external_id         UUID,
    version             BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS resource_tenant_type_key_unq
    ON resource (COALESCE(tenant_id::text, ''), resource_type, resource_key);

CREATE INDEX IF NOT EXISTS resource_tenant_type_idx
    ON resource (tenant_id, resource_type);
