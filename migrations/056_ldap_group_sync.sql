-- Migration 056: Add group sync fields to tenant_ldap_config (P4)
ALTER TABLE tenant_ldap_config
    ADD COLUMN IF NOT EXISTS group_search_base TEXT,
    ADD COLUMN IF NOT EXISTS group_filter       TEXT;
