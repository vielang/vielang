-- IoT Simulator — virtual device telemetry generation
CREATE TABLE IF NOT EXISTS simulator_config (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL,
    device_id        UUID NOT NULL REFERENCES device(id) ON DELETE CASCADE,
    name             VARCHAR(255) NOT NULL,
    enabled          BOOLEAN NOT NULL DEFAULT false,
    interval_ms      BIGINT NOT NULL DEFAULT 5000,
    telemetry_schema JSONB NOT NULL DEFAULT '[]',
    script           TEXT,
    created_time     BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    updated_time     BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_simulator_config_tenant ON simulator_config(tenant_id);
CREATE INDEX IF NOT EXISTS idx_simulator_config_device ON simulator_config(device_id);
CREATE INDEX IF NOT EXISTS idx_simulator_config_enabled ON simulator_config(enabled) WHERE enabled = true;
