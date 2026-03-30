-- Phase 63: Fine-Grained RBAC — custom roles, entity groups, user-role assignment
-- Backward compatible: existing SYS_ADMIN/TENANT_ADMIN/CUSTOMER_USER không bị ảnh hưởng

-- Custom roles (per-tenant)
CREATE TABLE IF NOT EXISTS tb_role (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    UUID         NOT NULL,
    name         VARCHAR(255) NOT NULL,
    role_type    VARCHAR(50)  NOT NULL DEFAULT 'GENERIC',  -- GENERIC | GROUP
    permissions  JSONB        NOT NULL DEFAULT '{}',
    created_time BIGINT       NOT NULL,
    UNIQUE (tenant_id, name)
);

-- Entity groups (nhóm devices/assets/dashboards để gán role)
CREATE TABLE IF NOT EXISTS entity_group (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL,
    customer_id     UUID,
    name            VARCHAR(255) NOT NULL,
    entity_type     VARCHAR(50)  NOT NULL,  -- DEVICE | ASSET | DASHBOARD | USER | ...
    additional_info JSONB,
    created_time    BIGINT       NOT NULL,
    UNIQUE (tenant_id, name, entity_type)
);

-- Entity group membership
CREATE TABLE IF NOT EXISTS entity_group_member (
    group_id  UUID NOT NULL REFERENCES entity_group(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL,
    PRIMARY KEY (group_id, entity_id)
);

-- User ↔ Role assignment
CREATE TABLE IF NOT EXISTS user_role (
    user_id  UUID NOT NULL,
    role_id  UUID NOT NULL REFERENCES tb_role(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_role_tenant     ON tb_role(tenant_id);
CREATE INDEX IF NOT EXISTS idx_eg_tenant_type  ON entity_group(tenant_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_egm_entity      ON entity_group_member(entity_id);
CREATE INDEX IF NOT EXISTS idx_user_role_user  ON user_role(user_id);
