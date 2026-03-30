-- Phase 21: Widget Management

CREATE TABLE IF NOT EXISTS widgets_bundle (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time    BIGINT NOT NULL,
    tenant_id       UUID,                          -- NULL = system bundle
    alias           VARCHAR(255) NOT NULL,
    title           VARCHAR(255) NOT NULL,
    image           TEXT,
    scada           BOOLEAN NOT NULL DEFAULT FALSE,
    description     VARCHAR(1024),
    order_index     INT,
    external_id     UUID,
    version         BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS widgets_bundle_tenant_alias_unq
    ON widgets_bundle (COALESCE(tenant_id::text, ''), alias);

CREATE TABLE IF NOT EXISTS widget_type (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time    BIGINT NOT NULL,
    tenant_id       UUID,                          -- NULL = system widget
    fqn             VARCHAR(512) NOT NULL,
    name            VARCHAR(255) NOT NULL,
    descriptor      JSONB NOT NULL DEFAULT '{}',
    deprecated      BOOLEAN NOT NULL DEFAULT FALSE,
    scada           BOOLEAN NOT NULL DEFAULT FALSE,
    image           TEXT,
    description     VARCHAR(1024),
    tags            TEXT[],
    external_id     UUID,
    version         BIGINT NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS widget_type_fqn_unq
    ON widget_type (COALESCE(tenant_id::text, ''), fqn);

CREATE TABLE IF NOT EXISTS widgets_bundle_widget (
    widgets_bundle_id   UUID NOT NULL REFERENCES widgets_bundle(id) ON DELETE CASCADE,
    widget_type_id      UUID NOT NULL REFERENCES widget_type(id) ON DELETE CASCADE,
    widget_type_order   INT NOT NULL DEFAULT 0,
    PRIMARY KEY (widgets_bundle_id, widget_type_id)
);
