-- Phase 35: Job Scheduler
-- Scheduled jobs and execution history

CREATE TABLE IF NOT EXISTS scheduled_job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    job_type VARCHAR(64) NOT NULL,       -- 'RULE_CHAIN', 'EXPORT', 'CLEANUP', 'CUSTOM'
    schedule_type VARCHAR(32) NOT NULL DEFAULT 'INTERVAL',  -- 'INTERVAL', 'CRON'
    interval_ms BIGINT,                  -- for INTERVAL type
    cron_expression VARCHAR(128),        -- for CRON type
    configuration JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    last_run_at BIGINT,
    next_run_at BIGINT NOT NULL DEFAULT 0,
    created_time BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000,
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_scheduled_job_tenant ON scheduled_job(tenant_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_job_next_run ON scheduled_job(next_run_at) WHERE enabled = true;

CREATE TABLE IF NOT EXISTS job_execution (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES scheduled_job(id) ON DELETE CASCADE,
    started_at BIGINT NOT NULL,
    finished_at BIGINT,
    status VARCHAR(32) NOT NULL DEFAULT 'RUNNING',  -- 'RUNNING', 'SUCCESS', 'FAILED'
    error_message TEXT,
    result JSONB
);

CREATE INDEX IF NOT EXISTS idx_job_execution_job ON job_execution(job_id);
