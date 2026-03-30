-- Migration 041: SAML 2.0 SP configuration per tenant

CREATE TABLE IF NOT EXISTS tenant_saml_config (
    id                  UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID    NOT NULL UNIQUE,
    enabled             BOOLEAN NOT NULL DEFAULT false,
    entity_id           TEXT    NOT NULL,                 -- SP entity ID (VieLang URL)
    sso_url             TEXT    NOT NULL,                 -- IdP Single Sign-On URL
    slo_url             TEXT,                             -- IdP Single Logout URL (optional)
    idp_certificate     TEXT    NOT NULL,                 -- PEM public cert to verify IdP signature
    sp_private_key      TEXT,                             -- PEM private key (for encrypted assertions)
    sp_certificate      TEXT,                             -- PEM SP cert (sent in metadata)
    email_attr          TEXT    NOT NULL DEFAULT 'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress',
    first_name_attr     TEXT    NOT NULL DEFAULT 'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname',
    last_name_attr      TEXT    NOT NULL DEFAULT 'http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname',
    force_authn         BOOLEAN NOT NULL DEFAULT false,
    name_id_format      TEXT    NOT NULL DEFAULT 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
    default_authority   TEXT    NOT NULL DEFAULT 'TENANT_ADMIN',
    created_time        BIGINT  NOT NULL,
    updated_time        BIGINT  NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_saml_config_tenant ON tenant_saml_config(tenant_id);
