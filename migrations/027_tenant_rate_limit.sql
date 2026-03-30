-- Phase 58.4: Per-tenant configurable rate limiting
-- Adds rate_limit_per_second column to tenant_profile.
-- NULL means "use server default" (currently 1000 req/s).
ALTER TABLE tenant_profile
    ADD COLUMN IF NOT EXISTS rate_limit_per_second INTEGER DEFAULT NULL;
