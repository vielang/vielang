-- Mobile sessions (FCM token per user per device)
CREATE TABLE IF NOT EXISTS mobile_session (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL,
    fcm_token    TEXT NOT NULL,
    platform     VARCHAR(20) NOT NULL DEFAULT 'ANDROID',
    app_version  VARCHAR(50),
    created_time BIGINT NOT NULL,
    UNIQUE (user_id, fcm_token)
);
CREATE INDEX IF NOT EXISTS idx_mobile_session_user ON mobile_session(user_id);

-- Notification delivery inbox (per-user notification records)
CREATE TABLE IF NOT EXISTS notification_inbox (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,
    recipient_user_id UUID NOT NULL,
    subject           TEXT,
    body              TEXT NOT NULL,
    notification_type VARCHAR(100),
    severity          VARCHAR(20) NOT NULL DEFAULT 'INFO',
    status            VARCHAR(20) NOT NULL DEFAULT 'SENT',
    sent_time         BIGINT NOT NULL,
    read_time         BIGINT,
    additional_config JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX IF NOT EXISTS idx_notif_inbox_user_time ON notification_inbox(recipient_user_id, sent_time DESC);
CREATE INDEX IF NOT EXISTS idx_notif_inbox_unread    ON notification_inbox(recipient_user_id) WHERE status = 'SENT';
