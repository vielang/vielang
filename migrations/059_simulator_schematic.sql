-- IoT Simulator Schematic — visual wiring of simulated devices
CREATE TABLE IF NOT EXISTS simulator_schematic (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID NOT NULL,
    name         VARCHAR(255) NOT NULL,
    graph_data   JSONB NOT NULL DEFAULT '{}',
    created_time BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    updated_time BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_simulator_schematic_tenant ON simulator_schematic(tenant_id);

-- Links JointJS nodes to simulator configs
CREATE TABLE IF NOT EXISTS schematic_node_config (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schematic_id        UUID NOT NULL REFERENCES simulator_schematic(id) ON DELETE CASCADE,
    node_id             VARCHAR(128) NOT NULL,
    simulator_config_id UUID REFERENCES simulator_config(id) ON DELETE SET NULL,
    node_type           VARCHAR(64) NOT NULL,
    properties          JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_schematic_node_schematic ON schematic_node_config(schematic_id);
