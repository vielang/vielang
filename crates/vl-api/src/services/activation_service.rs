use std::sync::Arc;
use uuid::Uuid;
use rand::{Rng, distr::Alphanumeric};
use tracing::warn;

use vl_dao::postgres::{
    activation_token::ActivationTokenDao,
    user::UserDao,
};
use crate::{
    error::ApiError,
    notification::channels::email::{EmailChannel, EmailMessage},
};

pub struct ActivationService {
    pub token_dao:      Arc<ActivationTokenDao>,
    pub user_dao:       Arc<UserDao>,
    email:              EmailChannel,
    token_ttl_secs:     i64,
    base_url:           String,
}

impl ActivationService {
    pub fn new(
        token_dao:   Arc<ActivationTokenDao>,
        user_dao:    Arc<UserDao>,
        email:       EmailChannel,
        ttl_hours:   u64,
        base_url:    String,
    ) -> Self {
        Self {
            token_dao,
            user_dao,
            email,
            token_ttl_secs: (ttl_hours * 3600) as i64,
            base_url,
        }
    }

    fn gen_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect()
    }

    /// Send activation email to new user
    pub async fn send_activation_email(&self, user_id: Uuid, email: &str) -> Result<(), ApiError> {
        let token = Self::gen_token();
        self.token_dao
            .store(user_id, &token, self.token_ttl_secs)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let activation_url = format!(
            "{}/api/noauth/activateByEmailCode?activateToken={}",
            self.base_url, token
        );
        let msg = EmailMessage {
            from:    "noreply@vielang.local".into(),
            to:      email.to_string(),
            subject: "Activate your VieLang account".into(),
            body:    format!(
                "Click the link to activate your account (valid 24h):\n\n{}\n\nIf you did not request this, ignore this email.",
                activation_url
            ),
            is_html: false,
        };
        if let Err(e) = self.email.send_message(&msg).await {
            warn!(email, "Failed to send activation email: {e}");
        }
        Ok(())
    }

    /// Verify token and return the user_id if valid
    pub async fn verify_activation_token(&self, token: &str) -> Result<Uuid, ApiError> {
        let record = self.token_dao
            .consume(token)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::BadRequest("Invalid or expired activation token".into()))?;
        Ok(record.user_id)
    }
}
