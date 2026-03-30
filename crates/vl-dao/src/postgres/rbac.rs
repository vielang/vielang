use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{EntityGroup, RolePermissions, RoleType, TbRole};

use crate::DaoError;

pub struct RbacDao {
    pool: PgPool,
}

impl RbacDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Role CRUD ──────────────────────────────────────────────────────────────

    #[instrument(skip(self, role))]
    pub async fn save_role(&self, role: &TbRole) -> Result<TbRole, DaoError> {
        let perms = serde_json::to_value(&role.permissions)?;
        sqlx::query!(
            r#"INSERT INTO tb_role (id, tenant_id, name, role_type, permissions, created_time)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (id) DO UPDATE SET
                   name        = EXCLUDED.name,
                   role_type   = EXCLUDED.role_type,
                   permissions = EXCLUDED.permissions"#,
            role.id,
            role.tenant_id,
            role.name,
            role.role_type.as_str(),
            perms,
            role.created_time,
        )
        .execute(&self.pool)
        .await?;

        self.find_role_by_id(role.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn find_role_by_id(&self, id: Uuid) -> Result<Option<TbRole>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, tenant_id, name, role_type, permissions, created_time
             FROM tb_role WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| TbRole {
            id:           r.id,
            tenant_id:    r.tenant_id,
            name:         r.name,
            role_type:    RoleType::from_str(&r.role_type),
            permissions:  serde_json::from_value(r.permissions).unwrap_or_default(),
            created_time: r.created_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_roles_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<TbRole>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id, tenant_id, name, role_type, permissions, created_time
             FROM tb_role WHERE tenant_id = $1 ORDER BY name",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TbRole {
            id:           r.id,
            tenant_id:    r.tenant_id,
            name:         r.name,
            role_type:    RoleType::from_str(&r.role_type),
            permissions:  serde_json::from_value(r.permissions).unwrap_or_default(),
            created_time: r.created_time,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn delete_role(&self, id: Uuid) -> Result<(), DaoError> {
        let r = sqlx::query!("DELETE FROM tb_role WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    // ── Entity Group CRUD ──────────────────────────────────────────────────────

    #[instrument(skip(self, group))]
    pub async fn save_group(&self, group: &EntityGroup) -> Result<EntityGroup, DaoError> {
        sqlx::query!(
            r#"INSERT INTO entity_group
                   (id, tenant_id, customer_id, name, entity_type, additional_info, created_time)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                   name            = EXCLUDED.name,
                   customer_id     = EXCLUDED.customer_id,
                   additional_info = EXCLUDED.additional_info"#,
            group.id,
            group.tenant_id,
            group.customer_id,
            group.name,
            group.entity_type,
            group.additional_info,
            group.created_time,
        )
        .execute(&self.pool)
        .await?;

        self.find_group_by_id(group.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn find_group_by_id(&self, id: Uuid) -> Result<Option<EntityGroup>, DaoError> {
        let row = sqlx::query!(
            "SELECT id, tenant_id, customer_id, name, entity_type, additional_info, created_time
             FROM entity_group WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntityGroup {
            id:              r.id,
            tenant_id:       r.tenant_id,
            customer_id:     r.customer_id,
            name:            r.name,
            entity_type:     r.entity_type,
            additional_info: r.additional_info,
            created_time:    r.created_time,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_groups_by_tenant(
        &self,
        tenant_id:   Uuid,
        entity_type: &str,
    ) -> Result<Vec<EntityGroup>, DaoError> {
        let rows = sqlx::query!(
            "SELECT id, tenant_id, customer_id, name, entity_type, additional_info, created_time
             FROM entity_group
             WHERE tenant_id = $1 AND entity_type = $2
             ORDER BY name",
            tenant_id,
            entity_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityGroup {
            id:              r.id,
            tenant_id:       r.tenant_id,
            customer_id:     r.customer_id,
            name:            r.name,
            entity_type:     r.entity_type,
            additional_info: r.additional_info,
            created_time:    r.created_time,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn delete_group(&self, id: Uuid) -> Result<(), DaoError> {
        let r = sqlx::query!("DELETE FROM entity_group WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    // ── Group membership ───────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn add_to_group(&self, group_id: Uuid, entity_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "INSERT INTO entity_group_member (group_id, entity_id) VALUES ($1, $2)
             ON CONFLICT DO NOTHING",
            group_id,
            entity_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn remove_from_group(&self, group_id: Uuid, entity_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM entity_group_member WHERE group_id = $1 AND entity_id = $2",
            group_id,
            entity_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<Uuid>, DaoError> {
        let rows = sqlx::query_scalar!(
            "SELECT entity_id FROM entity_group_member WHERE group_id = $1",
            group_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    #[instrument(skip(self))]
    pub async fn get_groups_for_entity(&self, entity_id: Uuid) -> Result<Vec<EntityGroup>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT g.id, g.tenant_id, g.customer_id, g.name, g.entity_type,
                      g.additional_info, g.created_time
               FROM entity_group g
               JOIN entity_group_member m ON m.group_id = g.id
               WHERE m.entity_id = $1"#,
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityGroup {
            id:              r.id,
            tenant_id:       r.tenant_id,
            customer_id:     r.customer_id,
            name:            r.name,
            entity_type:     r.entity_type,
            additional_info: r.additional_info,
            created_time:    r.created_time,
        }).collect())
    }

    // ── User ↔ Role assignment ─────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "INSERT INTO user_role (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            user_id,
            role_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn remove_role_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            "DELETE FROM user_role WHERE user_id = $1 AND role_id = $2",
            user_id,
            role_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<TbRole>, DaoError> {
        let rows = sqlx::query!(
            r#"SELECT r.id, r.tenant_id, r.name, r.role_type, r.permissions, r.created_time
               FROM tb_role r
               JOIN user_role ur ON ur.role_id = r.id
               WHERE ur.user_id = $1
               ORDER BY r.name"#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TbRole {
            id:           r.id,
            tenant_id:    r.tenant_id,
            name:         r.name,
            role_type:    RoleType::from_str(&r.role_type),
            permissions:  serde_json::from_value(r.permissions).unwrap_or_default(),
            created_time: r.created_time,
        }).collect())
    }

    // ── Merged permissions ─────────────────────────────────────────────────────

    /// Merge tất cả RolePermissions của user thành một map duy nhất.
    ///
    /// Dùng để build `RolePermissionsChecker` cho CUSTOMER_USER tại auth middleware.
    /// Kết quả nên được cache (e.g., 60s trong Redis) để tránh DB query mỗi request.
    #[instrument(skip(self))]
    pub async fn get_merged_permissions(&self, user_id: Uuid) -> Result<RolePermissions, DaoError> {
        let roles = self.get_user_roles(user_id).await?;
        let mut merged: HashMap<String, HashSet<String>> = HashMap::new();
        for role in roles {
            for (resource, ops) in role.permissions.0 {
                merged.entry(resource).or_default().extend(ops);
            }
        }
        let map = merged
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect::<Vec<_>>()))
            .collect();
        Ok(RolePermissions(map))
    }

    // ── Permission check ───────────────────────────────────────────────────────

    /// Kiểm tra user có quyền thực hiện operation trên một entity không.
    /// SYS_ADMIN và TENANT_ADMIN nên bypass check này ở middleware level.
    /// Method này dành cho CUSTOMER_USER với custom roles.
    #[instrument(skip(self))]
    pub async fn check_permission(
        &self,
        user_id:     Uuid,
        entity_id:   Uuid,
        entity_type: &str,
        operation:   &str,
    ) -> Result<bool, DaoError> {
        // Lấy tất cả roles của user
        let roles = self.get_user_roles(user_id).await?;
        if roles.is_empty() {
            return Ok(false);
        }

        for role in &roles {
            if role.permissions.can(entity_type, operation) {
                // GENERIC role: có permission → allowed
                if role.role_type == RoleType::Generic {
                    return Ok(true);
                }

                // GROUP role: phải check entity thuộc group không
                if role.role_type == RoleType::Group {
                    let groups = self.get_groups_for_entity(entity_id).await?;
                    // Kiểm tra có group nào của tenant này có entity không
                    if groups.iter().any(|g| g.entity_type == entity_type) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}
