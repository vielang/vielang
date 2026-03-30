-- Phase 60.2: Device claiming — pre-provision devices for customer self-service activation
ALTER TABLE device
    ADD COLUMN IF NOT EXISTS claiming_data  JSONB,
    ADD COLUMN IF NOT EXISTS claim_expiry_ts BIGINT;

CREATE INDEX IF NOT EXISTS idx_device_claiming
    ON device(tenant_id) WHERE claiming_data IS NOT NULL;
