CREATE TABLE IF NOT EXISTS entity_version (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type VARCHAR(64) NOT NULL,
    version_number BIGINT NOT NULL,
    commit_msg VARCHAR(512),
    snapshot JSONB NOT NULL,
    diff JSONB,
    created_by UUID,
    created_time BIGINT NOT NULL,
    UNIQUE(tenant_id, entity_id, version_number)
);
CREATE INDEX IF NOT EXISTS idx_ev_entity ON entity_version(entity_id, version_number DESC);
CREATE INDEX IF NOT EXISTS idx_ev_tenant ON entity_version(tenant_id, created_time DESC);
