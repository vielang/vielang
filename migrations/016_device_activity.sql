-- Phase 31: Device Activity & Connectivity Tracking

CREATE TABLE IF NOT EXISTS device_activity (
    device_id             UUID    NOT NULL PRIMARY KEY,
    last_connect_ts       BIGINT  NOT NULL DEFAULT 0,
    last_disconnect_ts    BIGINT  NOT NULL DEFAULT 0,
    last_activity_ts      BIGINT  NOT NULL DEFAULT 0,
    last_telemetry_ts     BIGINT  NOT NULL DEFAULT 0,
    last_rpc_ts           BIGINT  NOT NULL DEFAULT 0,
    active                BOOLEAN NOT NULL DEFAULT FALSE,
    -- Sau bao lâu không có activity → inactive (default 10 phút)
    inactivity_timeout_ms BIGINT  NOT NULL DEFAULT 600000
);

CREATE INDEX IF NOT EXISTS device_activity_active_idx
    ON device_activity (active)
    WHERE active = TRUE;
