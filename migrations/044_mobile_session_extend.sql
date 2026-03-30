-- P9: Mobile session — extend with device info for push targeting + analytics.
ALTER TABLE mobile_session
    ADD COLUMN IF NOT EXISTS os           VARCHAR(20),
    ADD COLUMN IF NOT EXISTS os_version   VARCHAR(50),
    ADD COLUMN IF NOT EXISTS device_model VARCHAR(100),
    ADD COLUMN IF NOT EXISTS last_active  BIGINT;

-- Index for last-active queries (e.g. prune stale sessions)
CREATE INDEX IF NOT EXISTS idx_mobile_session_last_active
    ON mobile_session(last_active) WHERE last_active IS NOT NULL;
