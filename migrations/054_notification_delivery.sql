-- P3: Notification delivery status tracking
-- Tracks per-channel delivery attempts for each notification request

CREATE TABLE IF NOT EXISTS notification_delivery (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id  UUID NOT NULL,
    channel_type     TEXT NOT NULL,
    recipient        TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'PENDING',
    error            TEXT,
    sent_at          BIGINT,
    created_time     BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notif_delivery_notif   ON notification_delivery(notification_id);
CREATE INDEX IF NOT EXISTS idx_notif_delivery_pending ON notification_delivery(status) WHERE status = 'PENDING';
