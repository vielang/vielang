-- VieLang Phase 61 — Rule Node tables (matching ThingsBoard Java schema)
-- Stores individual rule nodes separately from rule_chain.configuration JSON

-- ── Rule Node ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS rule_node (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time          BIGINT      NOT NULL,
    rule_chain_id         UUID        REFERENCES rule_chain(id) ON DELETE CASCADE,
    additional_info       VARCHAR,
    configuration_version INTEGER     NOT NULL DEFAULT 0,
    configuration         VARCHAR(10000000),
    type                  VARCHAR(255),
    name                  VARCHAR(255),
    debug_settings        VARCHAR(1024),
    singleton_mode        BOOLEAN     NOT NULL DEFAULT FALSE,
    queue_name            VARCHAR(255),
    external_id           UUID
);

CREATE INDEX IF NOT EXISTS idx_rule_node_chain_id
    ON rule_node (rule_chain_id);

CREATE INDEX IF NOT EXISTS idx_rule_node_type
    ON rule_node (type);

CREATE INDEX IF NOT EXISTS idx_rule_node_external_id
    ON rule_node (external_id) WHERE external_id IS NOT NULL;

-- ── Rule Node State ────────────────────────────────────────────────────────────
-- Persists state data for stateful rule engine nodes (e.g., aggregation windows)
CREATE TABLE IF NOT EXISTS rule_node_state (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time      BIGINT      NOT NULL,
    rule_node_id      UUID        NOT NULL REFERENCES rule_node(id) ON DELETE CASCADE,
    entity_type       VARCHAR(32) NOT NULL,
    entity_id         UUID        NOT NULL,
    state_data        VARCHAR(16384) NOT NULL,
    CONSTRAINT rule_node_state_unq_key UNIQUE (rule_node_id, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_rule_node_state_node_id
    ON rule_node_state (rule_node_id);
