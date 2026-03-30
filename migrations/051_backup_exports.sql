-- P14: Backup / Restore — audit log for tenant export history
CREATE TABLE IF NOT EXISTS backup_export_log (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID        NOT NULL,
    created_time  BIGINT      NOT NULL,
    -- Summary counts
    device_count  INT         NOT NULL DEFAULT 0,
    asset_count   INT         NOT NULL DEFAULT 0,
    customer_count INT        NOT NULL DEFAULT 0,
    dashboard_count INT       NOT NULL DEFAULT 0,
    rule_chain_count INT      NOT NULL DEFAULT 0,
    user_count    INT         NOT NULL DEFAULT 0,
    -- Options snapshot
    include_telemetry BOOLEAN NOT NULL DEFAULT false,
    -- Status: COMPLETED | FAILED
    status        VARCHAR(32) NOT NULL DEFAULT 'COMPLETED',
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_backup_export_log_tenant_time
    ON backup_export_log (tenant_id, created_time DESC);
