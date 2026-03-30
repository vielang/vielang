CREATE TABLE IF NOT EXISTS calculated_field (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type VARCHAR(32) NOT NULL,
    name VARCHAR(255) NOT NULL,
    expression TEXT NOT NULL,
    output_key VARCHAR(255) NOT NULL,
    input_keys TEXT[] NOT NULL,
    trigger_mode VARCHAR(32) NOT NULL DEFAULT 'ANY_CHANGE',
    output_ttl_ms BIGINT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_time BIGINT NOT NULL,
    UNIQUE(tenant_id, entity_id, name)
);
CREATE INDEX IF NOT EXISTS idx_calc_field_entity ON calculated_field(entity_id);
CREATE INDEX IF NOT EXISTS idx_calc_field_tenant ON calculated_field(tenant_id);
