-- Phase P12: Usage Records & Billing DAO
-- Extends api_usage_state with limit columns and new counters.
-- Adds api_usage_history for monthly period archiving.

-- ── Extend existing api_usage_state ──────────────────────────────────────────

-- Limit columns for existing counters (-1 = unlimited)
ALTER TABLE api_usage_state
    ADD COLUMN IF NOT EXISTS transport_msg_limit     BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS transport_dp_limit      BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS re_exec_limit           BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS js_exec_limit           BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS email_limit             BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS sms_limit               BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS alarm_limit             BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS active_device_limit     BIGINT NOT NULL DEFAULT -1;

-- New counters (P12) with matching limits
ALTER TABLE api_usage_state
    ADD COLUMN IF NOT EXISTS storage_dp_count        BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS storage_dp_limit        BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS rpc_count               BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rpc_limit               BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN IF NOT EXISTS rule_engine_exec_count  BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rule_engine_exec_limit  BIGINT NOT NULL DEFAULT -1;

-- ── Monthly usage history ─────────────────────────────────────────────────────
-- One row per tenant per billing period, created on reset_period().
-- counters JSONB stores a snapshot of all counter values at archive time.

CREATE TABLE IF NOT EXISTS api_usage_history (
    id           UUID   PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID   NOT NULL,
    period_start BIGINT NOT NULL,
    period_end   BIGINT NOT NULL,
    counters     JSONB  NOT NULL,
    created_time BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_usage_history_tenant
    ON api_usage_history(tenant_id, period_start DESC);
