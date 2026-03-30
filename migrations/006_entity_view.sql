-- VieLang Phase 20 — EntityView
-- Java: org.thingsboard.server.common.data.EntityView

CREATE TABLE IF NOT EXISTS entity_view (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    customer_id      UUID,
    entity_id        UUID        NOT NULL,
    entity_type      VARCHAR(255) NOT NULL,   -- DEVICE | ASSET | etc.
    name             VARCHAR(255) NOT NULL,
    type             VARCHAR(255) NOT NULL DEFAULT 'DEFAULT',
    -- keys: { "timeseries": [...], "attributes": {"cs": [...], "ss": [...], "sh": [...]} }
    keys             JSONB,
    start_ts         BIGINT      NOT NULL DEFAULT 0,
    end_ts           BIGINT      NOT NULL DEFAULT 0,
    additional_info  JSONB,
    external_id      UUID,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT entity_view_name_unq UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_entity_view_tenant    ON entity_view (tenant_id, created_time DESC);
CREATE INDEX IF NOT EXISTS idx_entity_view_customer  ON entity_view (customer_id, created_time DESC) WHERE customer_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_entity_view_entity    ON entity_view (entity_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_entity_view_type      ON entity_view (tenant_id, type);
