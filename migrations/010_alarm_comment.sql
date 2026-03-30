-- Phase 28: Alarm comments
-- Khớp Java: AlarmCommentEntity

CREATE TABLE IF NOT EXISTS alarm_comment (
    id              UUID        PRIMARY KEY,
    created_time    BIGINT      NOT NULL,
    alarm_id        UUID        NOT NULL REFERENCES alarm(id) ON DELETE CASCADE,
    user_id         UUID,
    type            VARCHAR(32) NOT NULL DEFAULT 'OTHER',  -- SYSTEM | OTHER
    comment         TEXT        NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_alarm_comment_alarm_id
    ON alarm_comment(alarm_id, created_time DESC);
