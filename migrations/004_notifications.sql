-- Phase 15: Notifications & Alerts
-- notification_template phải tạo trước vì notification_rule có FK tới nó

CREATE TABLE IF NOT EXISTS notification_template (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time      BIGINT       NOT NULL,
    tenant_id         UUID         NOT NULL,
    name              VARCHAR(255) NOT NULL,
    notification_type VARCHAR(50)  NOT NULL,         -- EMAIL, SLACK, WEBHOOK, SMS
    subject_template  VARCHAR(1000),                 -- for EMAIL only
    body_template     TEXT         NOT NULL,
    additional_config JSONB,
    enabled           BOOLEAN      NOT NULL DEFAULT TRUE,
    version           BIGINT       NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS notification_target (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time  BIGINT       NOT NULL,
    tenant_id     UUID         NOT NULL,
    name          VARCHAR(255) NOT NULL,
    target_type   VARCHAR(50)  NOT NULL,             -- PLATFORM_USERS, SLACK_CHANNEL, EMAIL_LIST, WEBHOOK
    target_config JSONB        NOT NULL DEFAULT '{}',
    version       BIGINT       NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS notification_rule (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time      BIGINT       NOT NULL,
    tenant_id         UUID         NOT NULL,
    name              VARCHAR(255) NOT NULL,
    template_id       UUID         NOT NULL REFERENCES notification_template(id) ON DELETE CASCADE,
    trigger_type      VARCHAR(50)  NOT NULL,         -- ALARM, DEVICE_ACTIVITY, ENTITY_ACTION, RULE_ENGINE, API_USAGE_LIMIT
    trigger_config    JSONB        NOT NULL DEFAULT '{}',
    recipients_config JSONB        NOT NULL DEFAULT '{}',
    additional_config JSONB,
    enabled           BOOLEAN      NOT NULL DEFAULT TRUE,
    version           BIGINT       NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS notification_request (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time BIGINT      NOT NULL,
    tenant_id    UUID        NOT NULL,
    rule_id      UUID,                               -- nullable: ad-hoc requests have no rule
    template_id  UUID        NOT NULL,               -- no FK — preserve history even if template deleted
    info         JSONB       NOT NULL DEFAULT '{}',  -- context variables for template rendering
    status       VARCHAR(50) NOT NULL DEFAULT 'SCHEDULED',
    error        TEXT,
    sent_time    BIGINT,
    version      BIGINT      NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_notif_template_tenant  ON notification_template(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notif_target_tenant    ON notification_target(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notif_rule_tenant      ON notification_rule(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notif_rule_template    ON notification_rule(template_id);
CREATE INDEX IF NOT EXISTS idx_notif_request_tenant   ON notification_request(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notif_request_rule     ON notification_request(rule_id);
CREATE INDEX IF NOT EXISTS idx_notif_request_status   ON notification_request(status);
