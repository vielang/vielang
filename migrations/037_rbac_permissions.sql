-- Phase S10: Fine-grained RBAC — queryable permission view
-- Expands the JSONB `permissions` column in `tb_role` into flat (role_id, resource, operation) rows.
-- This does NOT change the storage format — tb_role.permissions remains authoritative.

-- View: normalized role permissions for easy SQL querying
CREATE OR REPLACE VIEW v_role_permission AS
SELECT
    r.id          AS role_id,
    r.tenant_id,
    r.name        AS role_name,
    r.role_type,
    kv.key        AS resource,
    jsonb_array_elements_text(kv.value) AS operation
FROM tb_role r,
LATERAL jsonb_each(r.permissions) kv
WHERE jsonb_typeof(kv.value) = 'array';

-- View: merged permissions per user (union of all role grants)
CREATE OR REPLACE VIEW v_user_merged_permissions AS
SELECT DISTINCT
    ur.user_id,
    rp.resource,
    rp.operation
FROM user_role ur
JOIN v_role_permission rp ON rp.role_id = ur.role_id;

-- Index: fast lookup of user's permissions
CREATE INDEX IF NOT EXISTS idx_user_role_role ON user_role(role_id);

COMMENT ON VIEW v_role_permission IS
    'Normalized role permissions — each row is one (role, resource, operation) grant. '
    'Derived from tb_role.permissions JSONB. Read-only.';

COMMENT ON VIEW v_user_merged_permissions IS
    'Merged permissions for each user across all assigned roles. Read-only.';
