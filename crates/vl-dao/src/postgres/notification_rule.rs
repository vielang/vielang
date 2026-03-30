use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{NotificationRule, TriggerType};
use crate::{DaoError, PageData, PageLink};

pub struct NotificationRuleDao {
    pool: PgPool,
}

impl NotificationRuleDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationRule>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, template_id,
                   trigger_type, trigger_config, recipients_config,
                   additional_config, enabled, version
            FROM notification_rule WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| NotificationRule {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            template_id:       r.template_id,
            trigger_type:      TriggerType::from_str(&r.trigger_type)
                .unwrap_or(TriggerType::RuleEngine),
            trigger_config:    r.trigger_config,
            recipients_config: r.recipients_config,
            additional_config: r.additional_config,
            enabled:           r.enabled,
            version:           r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, rule: &NotificationRule) -> Result<NotificationRule, DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO notification_rule (
                id, created_time, tenant_id, name, template_id,
                trigger_type, trigger_config, recipients_config,
                additional_config, enabled, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (id) DO UPDATE SET
                name              = EXCLUDED.name,
                template_id       = EXCLUDED.template_id,
                trigger_type      = EXCLUDED.trigger_type,
                trigger_config    = EXCLUDED.trigger_config,
                recipients_config = EXCLUDED.recipients_config,
                additional_config = EXCLUDED.additional_config,
                enabled           = EXCLUDED.enabled,
                version           = notification_rule.version + 1
            "#,
            rule.id,
            rule.created_time,
            rule.tenant_id,
            rule.name,
            rule.template_id,
            rule.trigger_type.as_str(),
            rule.trigger_config,
            rule.recipients_config,
            rule.additional_config,
            rule.enabled,
            rule.version,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(rule.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!(
            "DELETE FROM notification_rule WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            Err(DaoError::NotFound)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<NotificationRule>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM notification_rule
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, template_id,
                   trigger_type, trigger_config, recipients_config,
                   additional_config, enabled, version
            FROM notification_rule
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(name) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| NotificationRule {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            template_id:       r.template_id,
            trigger_type:      TriggerType::from_str(&r.trigger_type)
                .unwrap_or(TriggerType::RuleEngine),
            trigger_config:    r.trigger_config,
            recipients_config: r.recipients_config,
            additional_config: r.additional_config,
            enabled:           r.enabled,
            version:           r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Tìm notification rules theo trigger type — dùng cho trigger matching
    #[instrument(skip(self))]
    pub async fn find_enabled_by_trigger(
        &self,
        tenant_id: Uuid,
        trigger_type: TriggerType,
    ) -> Result<Vec<NotificationRule>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, name, template_id,
                   trigger_type, trigger_config, recipients_config,
                   additional_config, enabled, version
            FROM notification_rule
            WHERE tenant_id = $1 AND trigger_type = $2 AND enabled = TRUE
            "#,
            tenant_id,
            trigger_type.as_str(),
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| NotificationRule {
            id:                r.id,
            created_time:      r.created_time,
            tenant_id:         r.tenant_id,
            name:              r.name,
            template_id:       r.template_id,
            trigger_type:      TriggerType::from_str(&r.trigger_type)
                .unwrap_or(TriggerType::RuleEngine),
            trigger_config:    r.trigger_config,
            recipients_config: r.recipients_config,
            additional_config: r.additional_config,
            enabled:           r.enabled,
            version:           r.version,
        }).collect())
    }
}
