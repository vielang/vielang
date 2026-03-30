use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{EntityRelation, EntityType, RelationTypeGroup};
use crate::DaoError;

pub struct RelationDao {
    pool: PgPool,
}

impl RelationDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    #[instrument(skip(self))]
    pub async fn save(&self, r: &EntityRelation) -> Result<(), DaoError> {
        let from_type = entity_type_str(&r.from_type);
        let to_type = entity_type_str(&r.to_type);
        let group = relation_group_str(&r.relation_type_group);
        let additional_info = r.additional_info.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO relation (
                from_id, from_type, to_id, to_type,
                relation_type, relation_type_group, additional_info
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            ON CONFLICT (from_id, from_type, relation_type_group, relation_type, to_id, to_type)
            DO UPDATE SET additional_info = EXCLUDED.additional_info
            "#,
            r.from_id,
            from_type,
            r.to_id,
            to_type,
            r.relation_type,
            group,
            additional_info,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(
        &self,
        from_id: Uuid,
        from_type: &str,
        to_id: Uuid,
        to_type: &str,
        relation_type: &str,
        group: &str,
    ) -> Result<(), DaoError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM relation
            WHERE from_id = $1 AND from_type = $2 AND to_id = $3
              AND to_type = $4 AND relation_type = $5 AND relation_type_group = $6
            "#,
            from_id, from_type, to_id, to_type, relation_type, group,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn find_by_from(
        &self,
        from_id: Uuid,
        from_type: &str,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE from_id = $1 AND from_type = $2
            ORDER BY relation_type, to_type
            "#,
            from_id, from_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn find_by_to(
        &self,
        to_id: Uuid,
        to_type: &str,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE to_id = $1 AND to_type = $2
            ORDER BY relation_type, from_type
            "#,
            to_id, to_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// Lọc theo relation_type và to_type (dùng cho GET /api/relations)
    #[instrument(skip(self))]
    pub async fn find_by_from_filtered(
        &self,
        from_id: Uuid,
        from_type: &str,
        relation_type: Option<&str>,
        to_type: Option<&str>,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE from_id = $1 AND from_type = $2
            AND ($3::text IS NULL OR relation_type = $3)
            AND ($4::text IS NULL OR to_type = $4)
            ORDER BY relation_type, to_type
            "#,
            from_id, from_type, relation_type, to_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// Find by from entity filtered by relation_type_group
    #[instrument(skip(self))]
    pub async fn find_by_from_with_group(
        &self,
        from_id: Uuid,
        from_type: &str,
        relation_type_group: &str,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE from_id = $1 AND from_type = $2 AND relation_type_group = $3
            ORDER BY relation_type, to_type
            "#,
            from_id, from_type, relation_type_group,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// Find by to entity with optional relation_type and from_type filters
    #[instrument(skip(self))]
    pub async fn find_by_to_filtered(
        &self,
        to_id: Uuid,
        to_type: &str,
        relation_type: Option<&str>,
        from_type: Option<&str>,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE to_id = $1 AND to_type = $2
            AND ($3::text IS NULL OR relation_type = $3)
            AND ($4::text IS NULL OR from_type = $4)
            ORDER BY relation_type, from_type
            "#,
            to_id, to_type, relation_type, from_type,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// Find by to entity filtered by relation_type_group
    #[instrument(skip(self))]
    pub async fn find_by_to_with_group(
        &self,
        to_id: Uuid,
        to_type: &str,
        relation_type_group: &str,
    ) -> Result<Vec<EntityRelation>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE to_id = $1 AND to_type = $2 AND relation_type_group = $3
            ORDER BY relation_type, from_type
            "#,
            to_id, to_type, relation_type_group,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }).collect())
    }

    /// Delete all COMMON relations for entity (both from and to direction)
    /// Java: deleteCommonRelations
    #[instrument(skip(self))]
    pub async fn delete_all_by_entity(
        &self,
        entity_id: Uuid,
        entity_type: &str,
    ) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            DELETE FROM relation
            WHERE (from_id = $1 AND from_type = $2 AND relation_type_group = 'COMMON')
               OR (to_id = $1 AND to_type = $2 AND relation_type_group = 'COMMON')
            "#,
            entity_id, entity_type,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a single relation by full composite key
    #[instrument(skip(self))]
    pub async fn get_relation(
        &self,
        from_id: Uuid,
        from_type: &str,
        to_id: Uuid,
        to_type: &str,
        relation_type: &str,
        relation_type_group: &str,
    ) -> Result<Option<EntityRelation>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT from_id, from_type, to_id, to_type,
                   relation_type, relation_type_group, additional_info
            FROM relation
            WHERE from_id = $1 AND from_type = $2 AND to_id = $3
              AND to_type = $4 AND relation_type = $5 AND relation_type_group = $6
            "#,
            from_id, from_type, to_id, to_type, relation_type, relation_type_group,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| EntityRelation {
            from_id: r.from_id,
            from_type: parse_entity_type(&r.from_type),
            to_id: r.to_id,
            to_type: parse_entity_type(&r.to_type),
            relation_type: r.relation_type,
            relation_type_group: parse_relation_group(&r.relation_type_group),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }))
    }
}

fn entity_type_str(t: &EntityType) -> &'static str {
    match t {
        EntityType::Tenant         => "TENANT",
        EntityType::Customer       => "CUSTOMER",
        EntityType::User           => "USER",
        EntityType::Dashboard      => "DASHBOARD",
        EntityType::Asset          => "ASSET",
        EntityType::Device         => "DEVICE",
        EntityType::AlarmEntity    => "ALARM",
        EntityType::RuleChain      => "RULE_CHAIN",
        EntityType::RuleNode       => "RULE_NODE",
        EntityType::EntityView     => "ENTITY_VIEW",
        EntityType::TenantProfile  => "TENANT_PROFILE",
        EntityType::DeviceProfile  => "DEVICE_PROFILE",
        EntityType::AssetProfile   => "ASSET_PROFILE",
        EntityType::Edge           => "EDGE",
        EntityType::OtaPackage     => "OTA_PACKAGE",
        _                          => "DEVICE",
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s {
        "TENANT"         => EntityType::Tenant,
        "CUSTOMER"       => EntityType::Customer,
        "USER"           => EntityType::User,
        "DASHBOARD"      => EntityType::Dashboard,
        "ASSET"          => EntityType::Asset,
        "RULE_CHAIN"     => EntityType::RuleChain,
        "RULE_NODE"      => EntityType::RuleNode,
        "ENTITY_VIEW"    => EntityType::EntityView,
        "TENANT_PROFILE" => EntityType::TenantProfile,
        "DEVICE_PROFILE" => EntityType::DeviceProfile,
        "ASSET_PROFILE"  => EntityType::AssetProfile,
        "EDGE"           => EntityType::Edge,
        "OTA_PACKAGE"    => EntityType::OtaPackage,
        _                => EntityType::Device,
    }
}

fn relation_group_str(g: &RelationTypeGroup) -> &'static str {
    match g {
        RelationTypeGroup::Common                 => "COMMON",
        RelationTypeGroup::Alarm                  => "ALARM",
        RelationTypeGroup::DashboardLink          => "DASHBOARD_LINK",
        RelationTypeGroup::RuleChain              => "RULE_CHAIN",
        RelationTypeGroup::RuleNode               => "RULE_NODE",
        RelationTypeGroup::EdgeAutoAssignDefault  => "EDGE_AUTO_ASSIGN_DEFAULT",
    }
}

fn parse_relation_group(s: &str) -> RelationTypeGroup {
    match s {
        "ALARM"                   => RelationTypeGroup::Alarm,
        "DASHBOARD_LINK"          => RelationTypeGroup::DashboardLink,
        "RULE_CHAIN"              => RelationTypeGroup::RuleChain,
        "RULE_NODE"               => RelationTypeGroup::RuleNode,
        "EDGE_AUTO_ASSIGN_DEFAULT"=> RelationTypeGroup::EdgeAutoAssignDefault,
        _                         => RelationTypeGroup::Common,
    }
}
