-- Phase 32: Housekeeper execution tracking

CREATE TABLE IF NOT EXISTS housekeeper_execution (
    id               UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at       BIGINT  NOT NULL,
    finished_at      BIGINT,
    cleaned_telemetry BIGINT NOT NULL DEFAULT 0,
    cleaned_events   BIGINT  NOT NULL DEFAULT 0,
    cleaned_alarms   BIGINT  NOT NULL DEFAULT 0,
    cleaned_rpc      BIGINT  NOT NULL DEFAULT 0,
    status           VARCHAR(32) NOT NULL DEFAULT 'RUNNING'
);

CREATE INDEX IF NOT EXISTS idx_housekeeper_started ON housekeeper_execution (started_at DESC);
