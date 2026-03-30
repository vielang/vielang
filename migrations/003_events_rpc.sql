-- VieLang Phase 13 — Events and RPC tables

-- ── Event table ─────────────────────────────────────────────────────────────
-- Store lifecycle events, debug events, statistics
CREATE TABLE IF NOT EXISTS event (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    entity_id        UUID        NOT NULL,
    entity_type      VARCHAR(255) NOT NULL,
    event_type       VARCHAR(255) NOT NULL,
    event_uid        VARCHAR(255) NOT NULL,
    body             JSONB       NOT NULL,
    CONSTRAINT event_uid_unq UNIQUE (tenant_id, entity_id, entity_type, event_uid)
);

CREATE INDEX IF NOT EXISTS idx_event_entity ON event (tenant_id, entity_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_event_created ON event (tenant_id, created_time DESC);
CREATE INDEX IF NOT EXISTS idx_event_type ON event (tenant_id, event_type);

-- ── RPC table ───────────────────────────────────────────────────────────────
-- Store persistent RPC requests
CREATE TABLE IF NOT EXISTS rpc (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    device_id        UUID        NOT NULL,
    request_id       INTEGER     NOT NULL,
    expiration_time  BIGINT      NOT NULL,
    request          JSONB       NOT NULL,
    response         JSONB,
    status           VARCHAR(255) NOT NULL DEFAULT 'QUEUED',
    additional_info  JSONB
);

CREATE INDEX IF NOT EXISTS idx_rpc_device ON rpc (tenant_id, device_id);
CREATE INDEX IF NOT EXISTS idx_rpc_status ON rpc (device_id, status) WHERE status IN ('QUEUED', 'SENT', 'DELIVERED');
CREATE INDEX IF NOT EXISTS idx_rpc_expiration ON rpc (expiration_time) WHERE status IN ('QUEUED', 'SENT', 'DELIVERED');
