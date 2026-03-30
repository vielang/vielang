use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::OAuth2ClientRegistrationTemplate;

use crate::DaoError;

pub struct OAuth2TemplateDao {
    pool: PgPool,
}

impl OAuth2TemplateDao {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    fn map(r: Oauth2TemplateRow) -> OAuth2ClientRegistrationTemplate {
        OAuth2ClientRegistrationTemplate {
            id:                           r.id,
            created_time:                 r.created_time,
            additional_info:              r.additional_info,
            provider_id:                  r.provider_id,
            name:                         r.name,
            authorization_uri:            r.authorization_uri,
            token_uri:                    r.token_uri,
            scope:                        r.scope,
            user_info_uri:                r.user_info_uri,
            user_name_attribute_name:     r.user_name_attribute_name,
            jwk_set_uri:                  r.jwk_set_uri,
            client_authentication_method: r.client_authentication_method,
            type_:                        r.type_,
            comment:                      r.comment,
            login_button_icon:            r.login_button_icon,
            login_button_label:           r.login_button_label,
            help_link:                    r.help_link,
            platforms:                    r.platforms,
        }
    }

    #[instrument(skip(self))]
    pub async fn find_all(&self) -> Result<Vec<OAuth2ClientRegistrationTemplate>, DaoError> {
        let rows = sqlx::query_as!(
            Oauth2TemplateRow,
            r#"SELECT id, created_time, additional_info, provider_id, name,
                      authorization_uri, token_uri, scope, user_info_uri,
                      user_name_attribute_name, jwk_set_uri, client_authentication_method,
                      type AS "type_", comment, login_button_icon, login_button_label,
                      help_link, platforms
               FROM oauth2_client_registration_template
               ORDER BY created_time DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Self::map).collect())
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuth2ClientRegistrationTemplate>, DaoError> {
        let row = sqlx::query_as!(
            Oauth2TemplateRow,
            r#"SELECT id, created_time, additional_info, provider_id, name,
                      authorization_uri, token_uri, scope, user_info_uri,
                      user_name_attribute_name, jwk_set_uri, client_authentication_method,
                      type AS "type_", comment, login_button_icon, login_button_label,
                      help_link, platforms
               FROM oauth2_client_registration_template
               WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Self::map))
    }

    #[instrument(skip(self, t))]
    pub async fn save(&self, t: &OAuth2ClientRegistrationTemplate) -> Result<OAuth2ClientRegistrationTemplate, DaoError> {
        sqlx::query!(
            r#"INSERT INTO oauth2_client_registration_template
               (id, created_time, additional_info, provider_id, name,
                authorization_uri, token_uri, scope, user_info_uri,
                user_name_attribute_name, jwk_set_uri, client_authentication_method,
                type, comment, login_button_icon, login_button_label, help_link, platforms)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
               ON CONFLICT (id) DO UPDATE SET
                   additional_info               = EXCLUDED.additional_info,
                   provider_id                   = EXCLUDED.provider_id,
                   name                          = EXCLUDED.name,
                   authorization_uri             = EXCLUDED.authorization_uri,
                   token_uri                     = EXCLUDED.token_uri,
                   scope                         = EXCLUDED.scope,
                   user_info_uri                 = EXCLUDED.user_info_uri,
                   user_name_attribute_name      = EXCLUDED.user_name_attribute_name,
                   jwk_set_uri                   = EXCLUDED.jwk_set_uri,
                   client_authentication_method  = EXCLUDED.client_authentication_method,
                   type                          = EXCLUDED.type,
                   comment                       = EXCLUDED.comment,
                   login_button_icon             = EXCLUDED.login_button_icon,
                   login_button_label            = EXCLUDED.login_button_label,
                   help_link                     = EXCLUDED.help_link,
                   platforms                     = EXCLUDED.platforms"#,
            t.id,
            t.created_time,
            t.additional_info.as_ref() as Option<&serde_json::Value>,
            t.provider_id.as_deref(),
            t.name.as_deref(),
            t.authorization_uri.as_deref(),
            t.token_uri.as_deref(),
            t.scope.as_deref(),
            t.user_info_uri.as_deref(),
            t.user_name_attribute_name.as_deref(),
            t.jwk_set_uri.as_deref(),
            t.client_authentication_method.as_deref(),
            t.type_.as_deref() as Option<&str>,
            t.comment.as_deref(),
            t.login_button_icon.as_deref(),
            t.login_button_label.as_deref(),
            t.help_link.as_deref(),
            t.platforms.as_deref(),
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(t.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let r = sqlx::query!(
            "DELETE FROM oauth2_client_registration_template WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;
        if r.rows_affected() == 0 { return Err(DaoError::NotFound); }
        Ok(())
    }
}

struct Oauth2TemplateRow {
    id:                           Uuid,
    created_time:                 i64,
    additional_info:              Option<serde_json::Value>,
    provider_id:                  Option<String>,
    name:                         Option<String>,
    authorization_uri:            Option<String>,
    token_uri:                    Option<String>,
    scope:                        Option<String>,
    user_info_uri:                Option<String>,
    user_name_attribute_name:     Option<String>,
    jwk_set_uri:                  Option<String>,
    client_authentication_method: Option<String>,
    type_:                        Option<String>,
    comment:                      Option<String>,
    login_button_icon:            Option<String>,
    login_button_label:           Option<String>,
    help_link:                    Option<String>,
    platforms:                    Option<String>,
}
