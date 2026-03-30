-- Phase 70: Subscription plan catalog and tenant billing

CREATE TABLE IF NOT EXISTS subscription_plan (
    id                         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time               BIGINT      NOT NULL,
    name                       VARCHAR(50) NOT NULL,
    display_name               VARCHAR(100) NOT NULL,
    description                TEXT,
    price_monthly              INTEGER     NOT NULL DEFAULT 0,
    price_annually             INTEGER     NOT NULL DEFAULT 0,
    stripe_price_id_monthly    VARCHAR(100),
    stripe_price_id_annually   VARCHAR(100),
    max_devices                INTEGER     NOT NULL DEFAULT 10,
    max_users                  INTEGER     NOT NULL DEFAULT 3,
    max_assets                 INTEGER     NOT NULL DEFAULT 10,
    max_dashboards             INTEGER     NOT NULL DEFAULT 5,
    max_rule_chains            INTEGER     NOT NULL DEFAULT 2,
    max_edges                  INTEGER     NOT NULL DEFAULT 0,
    max_transport_msgs_month   BIGINT      NOT NULL DEFAULT 100000,
    max_js_execs_month         BIGINT      NOT NULL DEFAULT 10000,
    max_emails_month           INTEGER     NOT NULL DEFAULT 100,
    max_sms_month              INTEGER     NOT NULL DEFAULT 0,
    max_alarms                 INTEGER     NOT NULL DEFAULT 100,
    max_api_keys               INTEGER     NOT NULL DEFAULT 1,
    feature_white_label        BOOLEAN     NOT NULL DEFAULT false,
    feature_edge_computing     BOOLEAN     NOT NULL DEFAULT false,
    feature_advanced_rbac      BOOLEAN     NOT NULL DEFAULT false,
    feature_audit_log          BOOLEAN     NOT NULL DEFAULT false,
    feature_sso                BOOLEAN     NOT NULL DEFAULT false,
    feature_api_export         BOOLEAN     NOT NULL DEFAULT false,
    sort_order                 INTEGER     NOT NULL DEFAULT 0,
    is_active                  BOOLEAN     NOT NULL DEFAULT true,
    CONSTRAINT subscription_plan_name_unq UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tenant_subscription (
    id                         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time               BIGINT      NOT NULL,
    updated_time               BIGINT      NOT NULL,
    tenant_id                  UUID        NOT NULL,
    plan_id                    UUID        NOT NULL REFERENCES subscription_plan(id),
    stripe_customer_id         VARCHAR(100),
    stripe_subscription_id     VARCHAR(100),
    billing_cycle              VARCHAR(10)  NOT NULL DEFAULT 'monthly',
    status                     VARCHAR(20)  NOT NULL DEFAULT 'free',
    current_period_start       BIGINT,
    current_period_end         BIGINT,
    trial_end                  BIGINT,
    cancel_at_period_end       BOOLEAN     NOT NULL DEFAULT false,
    canceled_at                BIGINT,
    CONSTRAINT tenant_subscription_tenant_unq UNIQUE (tenant_id),
    CONSTRAINT tenant_subscription_stripe_sub_unq UNIQUE (stripe_subscription_id)
);
CREATE INDEX IF NOT EXISTS idx_tsub_stripe_customer ON tenant_subscription(stripe_customer_id);
CREATE INDEX IF NOT EXISTS idx_tsub_status ON tenant_subscription(status);

CREATE TABLE IF NOT EXISTS stripe_event (
    stripe_event_id  VARCHAR(100) PRIMARY KEY,
    event_type       VARCHAR(100) NOT NULL,
    processed        BOOLEAN     NOT NULL DEFAULT false,
    received_time    BIGINT      NOT NULL,
    processed_time   BIGINT,
    payload          JSONB       NOT NULL DEFAULT '{}'::jsonb,
    error            TEXT
);
