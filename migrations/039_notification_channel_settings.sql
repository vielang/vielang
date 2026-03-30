-- P3: Notification Channel Settings per tenant
-- Lưu channel-specific config (Slack webhook URL, Telegram bot token, etc.)

CREATE TABLE IF NOT EXISTS notification_channel_settings (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID NOT NULL,
    channel      TEXT NOT NULL,   -- 'SLACK'|'MICROSOFT_TEAMS'|'FCM'|'WEBHOOK'|'TELEGRAM'
    config       JSONB NOT NULL,  -- channel-specific config (webhookUrl, botToken, etc.)
    enabled      BOOLEAN NOT NULL DEFAULT true,
    created_time BIGINT NOT NULL,
    UNIQUE (tenant_id, channel)
);

CREATE INDEX idx_notif_channel_settings_tenant ON notification_channel_settings(tenant_id);
