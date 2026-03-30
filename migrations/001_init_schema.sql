-- VieLang Phase 1 — Initial Schema
-- Chuyển đổi từ ThingsBoard Java Hibernate schema
-- Source: thingsboard/dao/src/main/resources/sql/schema-entities.sql

-- ── Extensions ────────────────────────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ── Enums (thay vì varchar để type-safe hơn Java) ────────────────────────────
-- Giữ varchar để tương thích với TB Java data nếu cần migrate data

-- ── Tenant Profile (phụ thuộc bởi tenant) ────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenant_profile (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    name             VARCHAR(255) NOT NULL,
    description      VARCHAR,
    is_default       BOOLEAN     NOT NULL DEFAULT FALSE,
    isolated_vl_rule_engine BOOLEAN NOT NULL DEFAULT FALSE,
    profile_data     JSONB,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT tenant_profile_name_unq UNIQUE (name)
);

-- ── Tenant ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenant (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_profile_id UUID       NOT NULL,
    title            VARCHAR(255) NOT NULL,
    region           VARCHAR(255),
    country          VARCHAR(255),
    state            VARCHAR(255),
    city             VARCHAR(255),
    address          VARCHAR,
    address2         VARCHAR,
    zip              VARCHAR(255),
    phone            VARCHAR(255),
    email            VARCHAR(255),
    additional_info  VARCHAR,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT fk_tenant_profile FOREIGN KEY (tenant_profile_id) REFERENCES tenant_profile(id)
);

-- ── Customer ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS customer (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    title            VARCHAR(255) NOT NULL,
    country          VARCHAR(255),
    state            VARCHAR(255),
    city             VARCHAR(255),
    address          VARCHAR,
    address2         VARCHAR,
    zip              VARCHAR(255),
    phone            VARCHAR(255),
    email            VARCHAR(255),
    is_public        BOOLEAN     NOT NULL DEFAULT FALSE,
    external_id      UUID,
    additional_info  VARCHAR,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT customer_title_unq   UNIQUE (tenant_id, title),
    CONSTRAINT customer_ext_id_unq  UNIQUE (tenant_id, external_id),
    CONSTRAINT fk_customer_tenant   FOREIGN KEY (tenant_id) REFERENCES tenant(id)
);

-- ── User ──────────────────────────────────────────────────────────────────────
-- tb_user (tránh conflict với reserved word "user")
CREATE TABLE IF NOT EXISTS tb_user (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID,
    customer_id      UUID,
    email            VARCHAR(255) NOT NULL,
    authority        VARCHAR(255) NOT NULL,
    first_name       VARCHAR(255),
    last_name        VARCHAR(255),
    phone            VARCHAR(255),
    additional_info  VARCHAR,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT user_email_unq UNIQUE (email)
);

CREATE TABLE IF NOT EXISTS user_credentials (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    user_id          UUID        NOT NULL,
    enabled          BOOLEAN     NOT NULL DEFAULT FALSE,
    password         VARCHAR,
    activate_token   VARCHAR,
    reset_token      VARCHAR,
    additional_info  VARCHAR,
    CONSTRAINT user_credentials_user_unq UNIQUE (user_id),
    CONSTRAINT fk_user_credentials FOREIGN KEY (user_id) REFERENCES tb_user(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_credentials_activate_token
    ON user_credentials (activate_token) WHERE activate_token IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_credentials_reset_token
    ON user_credentials (reset_token) WHERE reset_token IS NOT NULL;

-- ── Rule Chain (phụ thuộc bởi device_profile, asset_profile) ─────────────────
CREATE TABLE IF NOT EXISTS rule_chain (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    name             VARCHAR(255) NOT NULL,
    type             VARCHAR(255) NOT NULL DEFAULT 'CORE',
    first_rule_node_id UUID,
    root             BOOLEAN     NOT NULL DEFAULT FALSE,
    debug_mode       BOOLEAN     NOT NULL DEFAULT FALSE,
    configuration    VARCHAR,
    additional_info  VARCHAR,
    external_id      UUID,
    version          BIGINT      NOT NULL DEFAULT 1
);

-- ── Dashboard ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS dashboard (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    title            VARCHAR(255),
    configuration    VARCHAR,
    external_id      UUID,
    mobile_hide      BOOLEAN     NOT NULL DEFAULT FALSE,
    mobile_order     INTEGER,
    image            VARCHAR(1000000),
    version          BIGINT      NOT NULL DEFAULT 1
);

-- ── OTA Package (phụ thuộc bởi device_profile, device) ───────────────────────
CREATE TABLE IF NOT EXISTS ota_package (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    device_profile_id UUID,
    type             VARCHAR(255) NOT NULL,
    title            VARCHAR(255) NOT NULL,
    version          VARCHAR(255) NOT NULL,
    tag              VARCHAR(255),
    url              VARCHAR(255),
    file_name        VARCHAR(255),
    content_type     VARCHAR(255),
    checksum_algorithm VARCHAR(32),
    checksum         VARCHAR(1020),
    data             BYTEA,
    data_size        BIGINT,
    additional_info  VARCHAR,
    ver              BIGINT      NOT NULL DEFAULT 1
);

-- ── Device Profile ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS device_profile (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time             BIGINT      NOT NULL,
    tenant_id                UUID        NOT NULL,
    name                     VARCHAR(255) NOT NULL,
    description              VARCHAR,
    image                    VARCHAR(1000000),
    is_default               BOOLEAN     NOT NULL DEFAULT FALSE,
    type                     VARCHAR(255) NOT NULL DEFAULT 'DEFAULT',
    transport_type           VARCHAR(255) NOT NULL DEFAULT 'DEFAULT',
    provision_type           VARCHAR(255) NOT NULL DEFAULT 'DISABLED',
    profile_data             JSONB,
    default_rule_chain_id    UUID,
    default_dashboard_id     UUID,
    default_queue_name       VARCHAR(255),
    default_edge_rule_chain_id UUID,
    provision_device_key     VARCHAR      UNIQUE,
    firmware_id              UUID,
    software_id              UUID,
    external_id              UUID,
    version                  BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT device_profile_name_unq    UNIQUE (tenant_id, name),
    CONSTRAINT device_profile_ext_id_unq  UNIQUE (tenant_id, external_id),
    CONSTRAINT fk_dp_rule_chain       FOREIGN KEY (default_rule_chain_id)      REFERENCES rule_chain(id),
    CONSTRAINT fk_dp_dashboard        FOREIGN KEY (default_dashboard_id)       REFERENCES dashboard(id),
    CONSTRAINT fk_dp_edge_rule_chain  FOREIGN KEY (default_edge_rule_chain_id) REFERENCES rule_chain(id),
    CONSTRAINT fk_dp_firmware         FOREIGN KEY (firmware_id)               REFERENCES ota_package(id),
    CONSTRAINT fk_dp_software         FOREIGN KEY (software_id)               REFERENCES ota_package(id)
);

-- ── Asset Profile ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS asset_profile (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time             BIGINT      NOT NULL,
    tenant_id                UUID        NOT NULL,
    name                     VARCHAR(255) NOT NULL,
    description              VARCHAR,
    image                    VARCHAR(1000000),
    is_default               BOOLEAN     NOT NULL DEFAULT FALSE,
    default_rule_chain_id    UUID,
    default_dashboard_id     UUID,
    default_queue_name       VARCHAR(255),
    default_edge_rule_chain_id UUID,
    external_id              UUID,
    version                  BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT asset_profile_name_unq    UNIQUE (tenant_id, name),
    CONSTRAINT asset_profile_ext_id_unq  UNIQUE (tenant_id, external_id),
    CONSTRAINT fk_ap_rule_chain      FOREIGN KEY (default_rule_chain_id)      REFERENCES rule_chain(id),
    CONSTRAINT fk_ap_dashboard       FOREIGN KEY (default_dashboard_id)       REFERENCES dashboard(id),
    CONSTRAINT fk_ap_edge_rule_chain FOREIGN KEY (default_edge_rule_chain_id) REFERENCES rule_chain(id)
);

-- ── Device ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS device (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time      BIGINT      NOT NULL,
    tenant_id         UUID        NOT NULL,
    customer_id       UUID,
    device_profile_id UUID        NOT NULL,
    name              VARCHAR(255) NOT NULL,
    type              VARCHAR(255),
    label             VARCHAR(255),
    device_data       JSONB,
    firmware_id       UUID,
    software_id       UUID,
    external_id       UUID,
    additional_info   VARCHAR,
    version           BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT device_name_unq     UNIQUE (tenant_id, name),
    CONSTRAINT device_ext_id_unq   UNIQUE (tenant_id, external_id),
    CONSTRAINT fk_device_profile   FOREIGN KEY (device_profile_id) REFERENCES device_profile(id),
    CONSTRAINT fk_device_firmware  FOREIGN KEY (firmware_id)       REFERENCES ota_package(id),
    CONSTRAINT fk_device_software  FOREIGN KEY (software_id)       REFERENCES ota_package(id)
);

CREATE INDEX IF NOT EXISTS idx_device_tenant    ON device (tenant_id);
CREATE INDEX IF NOT EXISTS idx_device_customer  ON device (customer_id);
CREATE INDEX IF NOT EXISTS idx_device_profile   ON device (device_profile_id);

CREATE TABLE IF NOT EXISTS device_credentials (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    device_id        UUID        NOT NULL,
    credentials_type VARCHAR(255) NOT NULL,
    credentials_id   VARCHAR(255) NOT NULL,
    credentials_value VARCHAR,
    CONSTRAINT device_credentials_device_unq UNIQUE (device_id),
    CONSTRAINT device_credentials_id_unq     UNIQUE (credentials_id),
    CONSTRAINT fk_device_credentials FOREIGN KEY (device_id) REFERENCES device(id) ON DELETE CASCADE
);

-- ── Asset ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS asset (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time     BIGINT      NOT NULL,
    tenant_id        UUID        NOT NULL,
    customer_id      UUID,
    asset_profile_id UUID        NOT NULL,
    name             VARCHAR(255) NOT NULL,
    type             VARCHAR(255),
    label            VARCHAR(255),
    external_id      UUID,
    additional_info  VARCHAR,
    version          BIGINT      NOT NULL DEFAULT 1,
    CONSTRAINT asset_name_unq    UNIQUE (tenant_id, name),
    CONSTRAINT asset_ext_id_unq  UNIQUE (tenant_id, external_id),
    CONSTRAINT fk_asset_profile  FOREIGN KEY (asset_profile_id) REFERENCES asset_profile(id)
);

CREATE INDEX IF NOT EXISTS idx_asset_tenant   ON asset (tenant_id);
CREATE INDEX IF NOT EXISTS idx_asset_customer ON asset (customer_id);

-- ── Alarm ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS alarm (
    id                      UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    created_time            BIGINT  NOT NULL,
    tenant_id               UUID    NOT NULL,
    customer_id             UUID,
    type                    VARCHAR(255) NOT NULL,
    originator_id           UUID    NOT NULL,
    originator_type         INTEGER NOT NULL,
    severity                VARCHAR(255) NOT NULL,
    acknowledged            BOOLEAN NOT NULL DEFAULT FALSE,
    cleared                 BOOLEAN NOT NULL DEFAULT FALSE,
    assignee_id             UUID,
    start_ts                BIGINT  NOT NULL,
    end_ts                  BIGINT  NOT NULL,
    ack_ts                  BIGINT,
    clear_ts                BIGINT,
    assign_ts               BIGINT  NOT NULL DEFAULT 0,
    propagate               BOOLEAN NOT NULL DEFAULT FALSE,
    propagate_to_owner      BOOLEAN NOT NULL DEFAULT FALSE,
    propagate_to_tenant     BOOLEAN NOT NULL DEFAULT FALSE,
    propagate_relation_types VARCHAR,
    details                 VARCHAR
);

CREATE INDEX IF NOT EXISTS idx_alarm_originator ON alarm (tenant_id, originator_id, originator_type);
CREATE INDEX IF NOT EXISTS idx_alarm_tenant_type ON alarm (tenant_id, type);
CREATE INDEX IF NOT EXISTS idx_alarm_start_ts    ON alarm (tenant_id, start_ts DESC);

-- ── Relation ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS relation (
    from_id             UUID        NOT NULL,
    from_type           VARCHAR(255) NOT NULL,
    to_id               UUID        NOT NULL,
    to_type             VARCHAR(255) NOT NULL,
    relation_type       VARCHAR(255) NOT NULL,
    relation_type_group VARCHAR(255) NOT NULL,
    additional_info     VARCHAR,
    CONSTRAINT relation_pkey PRIMARY KEY (from_id, from_type, relation_type_group, relation_type, to_id, to_type)
);

CREATE INDEX IF NOT EXISTS idx_relation_from ON relation (from_id, from_type, relation_type_group);
CREATE INDEX IF NOT EXISTS idx_relation_to   ON relation (to_id, to_type, relation_type_group);

-- ── Key Dictionary — ánh xạ string key → integer (tiết kiệm storage) ─────────
CREATE TABLE IF NOT EXISTS key_dictionary (
    key     VARCHAR(255) PRIMARY KEY,
    key_id  SERIAL       UNIQUE NOT NULL
);

-- ── Telemetry Latest Values ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ts_kv_latest (
    entity_id   UUID    NOT NULL,
    key         INTEGER NOT NULL,
    ts          BIGINT  NOT NULL,
    bool_v      BOOLEAN,
    str_v       VARCHAR(10000000),
    long_v      BIGINT,
    dbl_v       DOUBLE PRECISION,
    json_v      JSON,
    version     BIGINT  NOT NULL DEFAULT 0,
    CONSTRAINT ts_kv_latest_pkey PRIMARY KEY (entity_id, key),
    CONSTRAINT fk_ts_kv_latest_key FOREIGN KEY (key) REFERENCES key_dictionary(key_id)
);

-- ── Attribute Key-Value ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS attribute_kv (
    entity_id       UUID    NOT NULL,
    attribute_type  INTEGER NOT NULL,  -- 1=CLIENT, 2=SERVER, 3=SHARED
    attribute_key   INTEGER NOT NULL,
    bool_v          BOOLEAN,
    str_v           VARCHAR(10000000),
    long_v          BIGINT,
    dbl_v           DOUBLE PRECISION,
    json_v          JSON,
    last_update_ts  BIGINT  NOT NULL,
    version         BIGINT  NOT NULL DEFAULT 0,
    CONSTRAINT attribute_kv_pkey PRIMARY KEY (entity_id, attribute_type, attribute_key),
    CONSTRAINT fk_attribute_kv_key FOREIGN KEY (attribute_key) REFERENCES key_dictionary(key_id)
);
