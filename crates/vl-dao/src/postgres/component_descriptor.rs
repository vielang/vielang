use sqlx::PgPool;
use tracing::instrument;

use vl_core::entities::ComponentDescriptor;
use crate::DaoError;

pub struct ComponentDescriptorDao {
    pool: PgPool,
}

impl ComponentDescriptorDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all component descriptors of a given type (e.g. "FILTER", "ACTION")
    #[instrument(skip(self))]
    pub async fn find_by_type(&self, component_type: &str) -> Result<Vec<ComponentDescriptor>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, type, scope, clustering_mode, name, clazz,
                   configuration_descriptor, configuration_version, actions, has_queue_name
            FROM component_descriptor
            WHERE type = $1
            ORDER BY name
            "#,
            component_type
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ComponentDescriptor {
            id:                       r.id,
            created_time:             r.created_time,
            type_:                    r.r#type,
            scope:                    r.scope,
            clustering_mode:          r.clustering_mode,
            name:                     r.name,
            clazz:                    r.clazz,
            configuration_descriptor: r.configuration_descriptor
                .and_then(|s| serde_json::from_str(&s).ok()),
            configuration_version:    r.configuration_version,
            actions:                  r.actions,
            has_queue_name:           r.has_queue_name,
        }).collect())
    }

    /// Find all component descriptors matching any of the given types
    #[instrument(skip(self))]
    pub async fn find_by_types(&self, component_types: &[String]) -> Result<Vec<ComponentDescriptor>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, type, scope, clustering_mode, name, clazz,
                   configuration_descriptor, configuration_version, actions, has_queue_name
            FROM component_descriptor
            WHERE type = ANY($1::text[])
            ORDER BY name
            "#,
            component_types as &[String]
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ComponentDescriptor {
            id:                       r.id,
            created_time:             r.created_time,
            type_:                    r.r#type,
            scope:                    r.scope,
            clustering_mode:          r.clustering_mode,
            name:                     r.name,
            clazz:                    r.clazz,
            configuration_descriptor: r.configuration_descriptor
                .and_then(|s| serde_json::from_str(&s).ok()),
            configuration_version:    r.configuration_version,
            actions:                  r.actions,
            has_queue_name:           r.has_queue_name,
        }).collect())
    }

    /// Find all component descriptors
    #[instrument(skip(self))]
    pub async fn find_all(&self) -> Result<Vec<ComponentDescriptor>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, type, scope, clustering_mode, name, clazz,
                   configuration_descriptor, configuration_version, actions, has_queue_name
            FROM component_descriptor
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ComponentDescriptor {
            id:                       r.id,
            created_time:             r.created_time,
            type_:                    r.r#type,
            scope:                    r.scope,
            clustering_mode:          r.clustering_mode,
            name:                     r.name,
            clazz:                    r.clazz,
            configuration_descriptor: r.configuration_descriptor
                .and_then(|s| serde_json::from_str(&s).ok()),
            configuration_version:    r.configuration_version,
            actions:                  r.actions,
            has_queue_name:           r.has_queue_name,
        }).collect())
    }

    /// Find a single component descriptor by its class name
    #[instrument(skip(self))]
    pub async fn find_by_clazz(&self, clazz: &str) -> Result<Option<ComponentDescriptor>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, type, scope, clustering_mode, name, clazz,
                   configuration_descriptor, configuration_version, actions, has_queue_name
            FROM component_descriptor
            WHERE clazz = $1
            "#,
            clazz
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ComponentDescriptor {
            id:                       r.id,
            created_time:             r.created_time,
            type_:                    r.r#type,
            scope:                    r.scope,
            clustering_mode:          r.clustering_mode,
            name:                     r.name,
            clazz:                    r.clazz,
            configuration_descriptor: r.configuration_descriptor
                .and_then(|s| serde_json::from_str(&s).ok()),
            configuration_version:    r.configuration_version,
            actions:                  r.actions,
            has_queue_name:           r.has_queue_name,
        }))
    }
}
