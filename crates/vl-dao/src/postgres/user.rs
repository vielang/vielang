use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{Authority, User, UserCredentials};
use crate::{DaoError, PageData, PageLink};

pub struct UserDao {
    pool: PgPool,
}

impl UserDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, email,
                   authority, first_name, last_name, phone,
                   additional_info, version
            FROM tb_user WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id.unwrap_or_default(),
            customer_id: r.customer_id,
            email: r.email,
            authority: parse_authority(&r.authority),
            first_name: r.first_name,
            last_name: r.last_name,
            phone: r.phone,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, email,
                   authority, first_name, last_name, phone,
                   additional_info, version
            FROM tb_user WHERE LOWER(email) = LOWER($1)
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id.unwrap_or_default(),
            customer_id: r.customer_id,
            email: r.email,
            authority: parse_authority(&r.authority),
            first_name: r.first_name,
            last_name: r.last_name,
            phone: r.phone,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<User>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tb_user
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(email) LIKE LOWER($2)
                    OR LOWER(first_name) LIKE LOWER($2)
                    OR LOWER(last_name) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, email,
                   authority, first_name, last_name, phone, additional_info, version
            FROM tb_user
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(email) LIKE LOWER($2)
                 OR LOWER(first_name) LIKE LOWER($2)
                 OR LOWER(last_name) LIKE LOWER($2))
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

        let data = rows.into_iter().map(|r| User {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id.unwrap_or_default(),
            customer_id: r.customer_id,
            email: r.email,
            authority: parse_authority(&r.authority),
            first_name: r.first_name,
            last_name: r.last_name,
            phone: r.phone,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Export all users for a tenant (used by backup service).
    #[instrument(skip(self))]
    pub async fn find_all_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<User>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, customer_id, email,
                   authority, first_name, last_name, phone, additional_info, version
            FROM tb_user WHERE tenant_id = $1 ORDER BY created_time
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| User {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id.unwrap_or_default(),
            customer_id: r.customer_id,
            email: r.email,
            authority: parse_authority(&r.authority),
            first_name: r.first_name,
            last_name: r.last_name,
            phone: r.phone,
            additional_info: r.additional_info.and_then(|s| serde_json::from_str(&s).ok()),
            version: r.version,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn save(&self, user: &User) -> Result<User, DaoError> {
        let additional_info = user.additional_info.as_ref().map(|v| v.to_string());
        let authority = authority_to_db_str(&user.authority);

        sqlx::query!(
            r#"
            INSERT INTO tb_user (
                id, created_time, tenant_id, customer_id,
                email, authority, first_name, last_name, phone,
                additional_info, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (id) DO UPDATE SET
                email           = EXCLUDED.email,
                authority       = EXCLUDED.authority,
                first_name      = EXCLUDED.first_name,
                last_name       = EXCLUDED.last_name,
                phone           = EXCLUDED.phone,
                customer_id     = EXCLUDED.customer_id,
                additional_info = EXCLUDED.additional_info,
                version         = tb_user.version + 1
            "#,
            user.id,
            user.created_time,
            user.tenant_id,
            user.customer_id,
            user.email,
            authority,
            user.first_name,
            user.last_name,
            user.phone,
            additional_info,
            user.version,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(user.id).await?.ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn find_credentials(&self, user_id: Uuid) -> Result<Option<UserCredentials>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, user_id, enabled, password,
                   activate_token, reset_token, additional_info
            FROM user_credentials WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserCredentials {
            id: r.id,
            created_time: r.created_time,
            user_id: r.user_id,
            enabled: r.enabled,
            password: r.password,
            activate_token: r.activate_token,
            reset_token: r.reset_token,
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
        }))
    }

    /// Kích hoạt tài khoản user bằng activate_token — set password, clear token, enable
    #[instrument(skip(self, hashed_password))]
    pub async fn activate_user(
        &self,
        activate_token: &str,
        hashed_password: &str,
    ) -> Result<User, DaoError> {
        let row = sqlx::query!(
            r#"
            UPDATE user_credentials
            SET password       = $1,
                activate_token = NULL,
                enabled        = true
            WHERE activate_token = $2
            RETURNING user_id
            "#,
            hashed_password,
            activate_token,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DaoError::NotFound)?;

        self.find_by_id(row.user_id).await?.ok_or(DaoError::NotFound)
    }

    /// Set reset token cho user theo email (sau đó gửi email với token này)
    #[instrument(skip(self, reset_token))]
    pub async fn reset_password_token(
        &self,
        email: &str,
        reset_token: &str,
    ) -> Result<(), DaoError> {
        let user = self.find_by_email(email).await?.ok_or(DaoError::NotFound)?;

        sqlx::query!(
            r#"
            UPDATE user_credentials
            SET reset_token = $1
            WHERE user_id = $2
            "#,
            reset_token,
            user.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, user_id: Uuid) -> Result<(), DaoError> {
        sqlx::query!(
            r#"DELETE FROM user_credentials WHERE user_id = $1"#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"DELETE FROM tb_user WHERE id = $1"#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, DaoError> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tb_user WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    #[instrument(skip(self))]
    pub async fn count_all(&self) -> Result<i64, DaoError> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM tb_user"#
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    /// Phase 69: store base64-encoded avatar in additional_info JSONB
    #[instrument(skip(self, b64_image))]
    pub async fn set_avatar(&self, user_id: Uuid, b64_image: &str) -> Result<(), DaoError> {
        sqlx::query!(
            r#"UPDATE tb_user
               SET additional_info = jsonb_set(
                   COALESCE(additional_info::jsonb, '{}'::jsonb),
                   '{avatarBase64}',
                   to_jsonb($2::text)
               )
               WHERE id = $1"#,
            user_id,
            b64_image
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Phase 69: retrieve avatar from additional_info JSONB
    #[instrument(skip(self))]
    pub async fn get_avatar(&self, user_id: Uuid) -> Result<Option<String>, DaoError> {
        let row = sqlx::query!(
            "SELECT additional_info FROM tb_user WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| {
            r.additional_info
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|info| info["avatarBase64"].as_str().map(str::to_owned))
                .filter(|s| !s.is_empty())
        }))
    }

    /// Phase 69: get notification settings from additional_info JSONB
    #[instrument(skip(self))]
    pub async fn get_notification_settings(&self, user_id: Uuid) -> Result<Option<serde_json::Value>, DaoError> {
        let row = sqlx::query!(
            "SELECT additional_info FROM tb_user WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| {
            r.additional_info
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|info| info.get("notificationSettings").cloned())
        }))
    }

    /// Phase 69: save notification settings into additional_info JSONB
    #[instrument(skip(self, settings))]
    pub async fn set_notification_settings(&self, user_id: Uuid, settings: &serde_json::Value) -> Result<(), DaoError> {
        sqlx::query!(
            r#"UPDATE tb_user
               SET additional_info = jsonb_set(
                   COALESCE(additional_info::jsonb, '{}'::jsonb),
                   '{notificationSettings}',
                   $2
               )
               WHERE id = $1"#,
            user_id,
            settings
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn save_credentials(&self, creds: &UserCredentials) -> Result<(), DaoError> {
        let additional_info = creds.additional_info.as_ref().map(|v| v.to_string());

        sqlx::query!(
            r#"
            INSERT INTO user_credentials (
                id, created_time, user_id, enabled, password,
                activate_token, reset_token, additional_info
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (user_id) DO UPDATE SET
                enabled        = EXCLUDED.enabled,
                password       = EXCLUDED.password,
                activate_token = EXCLUDED.activate_token,
                reset_token    = EXCLUDED.reset_token,
                additional_info= EXCLUDED.additional_info
            "#,
            creds.id,
            creds.created_time,
            creds.user_id,
            creds.enabled,
            creds.password,
            creds.activate_token,
            creds.reset_token,
            additional_info,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        Ok(())
    }
}

fn parse_authority(s: &str) -> Authority {
    match s {
        "SYS_ADMIN"             => Authority::SysAdmin,
        "TENANT_ADMIN"          => Authority::TenantAdmin,
        "CUSTOMER_USER"         => Authority::CustomerUser,
        "REFRESH_TOKEN"         => Authority::RefreshToken,
        "PRE_VERIFICATION_TOKEN"=> Authority::PreVerificationToken,
        _                       => Authority::CustomerUser,
    }
}

fn authority_to_db_str(auth: &Authority) -> &'static str {
    match auth {
        Authority::SysAdmin             => "SYS_ADMIN",
        Authority::TenantAdmin          => "TENANT_ADMIN",
        Authority::CustomerUser         => "CUSTOMER_USER",
        Authority::RefreshToken         => "REFRESH_TOKEN",
        Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
    }
}
