-- Phase 71: API usage tracking per tenant per billing period

CREATE TABLE IF NOT EXISTS api_usage_state (
    id               UUID       PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID       NOT NULL,
    billing_period   VARCHAR(7) NOT NULL,  -- 'YYYY-MM'
    transport_msg_count   BIGINT   NOT NULL DEFAULT 0,
    transport_dp_count    BIGINT   NOT NULL DEFAULT 0,
    re_exec_count         BIGINT   NOT NULL DEFAULT 0,
    js_exec_count         BIGINT   NOT NULL DEFAULT 0,
    email_count           INTEGER  NOT NULL DEFAULT 0,
    sms_count             INTEGER  NOT NULL DEFAULT 0,
    alarm_count           INTEGER  NOT NULL DEFAULT 0,
    active_device_count   INTEGER  NOT NULL DEFAULT 0,
    created_time     BIGINT     NOT NULL,
    updated_time     BIGINT     NOT NULL,
    CONSTRAINT api_usage_tenant_period_unq UNIQUE (tenant_id, billing_period)
);
CREATE INDEX IF NOT EXISTS idx_api_usage_tenant ON api_usage_state(tenant_id, billing_period);
