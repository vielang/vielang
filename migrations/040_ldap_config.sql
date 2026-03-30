-- Migration 040: LDAP / Active Directory configuration per tenant
-- Allows each tenant to configure their own LDAP server for SSO.

CREATE TABLE IF NOT EXISTS tenant_ldap_config (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL UNIQUE,
    enabled          BOOLEAN     NOT NULL DEFAULT false,
    server_url       TEXT        NOT NULL,                 -- ldap://host:389 or ldaps://host:636
    use_tls          BOOLEAN     NOT NULL DEFAULT false,
    base_dn          TEXT        NOT NULL,                 -- DC=company,DC=com
    search_filter    TEXT        NOT NULL DEFAULT '(sAMAccountName={username})',
    bind_dn          TEXT        NOT NULL,                 -- service account DN
    bind_password    TEXT        NOT NULL,
    username_attr    TEXT        NOT NULL DEFAULT 'sAMAccountName',
    first_name_attr  TEXT        NOT NULL DEFAULT 'givenName',
    last_name_attr   TEXT        NOT NULL DEFAULT 'sn',
    email_attr       TEXT        NOT NULL DEFAULT 'mail',
    default_authority TEXT       NOT NULL DEFAULT 'TENANT_ADMIN',
    created_time     BIGINT      NOT NULL,
    updated_time     BIGINT      NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_ldap_config_tenant ON tenant_ldap_config(tenant_id);
