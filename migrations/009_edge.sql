-- Phase 23: Edge Gateway

CREATE TABLE IF NOT EXISTS edge (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time        BIGINT NOT NULL,
    tenant_id           UUID NOT NULL,
    customer_id         UUID,
    root_rule_chain_id  UUID,
    name                VARCHAR(255) NOT NULL,
    type                VARCHAR(255) NOT NULL DEFAULT 'DEFAULT',
    label               VARCHAR(255),
    routing_key         VARCHAR(255) NOT NULL,
    secret              VARCHAR(255) NOT NULL,
    additional_info     JSONB,
    external_id         UUID,
    version             BIGINT NOT NULL DEFAULT 1,
    CONSTRAINT edge_routing_key_unq UNIQUE (routing_key),
    CONSTRAINT edge_tenant_name_unq UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS edge_tenant_id_idx ON edge (tenant_id);
CREATE INDEX IF NOT EXISTS edge_customer_id_idx ON edge (customer_id);

CREATE TABLE IF NOT EXISTS edge_event (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time BIGINT NOT NULL,
    seq_id       BIGSERIAL,
    tenant_id   UUID NOT NULL,
    edge_id     UUID NOT NULL REFERENCES edge(id) ON DELETE CASCADE,
    edge_event_type  VARCHAR(255) NOT NULL,
    edge_event_action VARCHAR(255) NOT NULL,
    entity_id   UUID,
    body        JSONB,
    uid         VARCHAR(255)
);

CREATE INDEX IF NOT EXISTS edge_event_edge_id_idx ON edge_event (edge_id, seq_id DESC);
CREATE INDEX IF NOT EXISTS edge_event_tenant_id_idx ON edge_event (tenant_id);
