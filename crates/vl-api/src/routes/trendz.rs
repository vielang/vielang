use axum::{
    extract::{Extension, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use vl_core::entities::AdminSettings;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState, CoreState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trendz/settings",  get(get_trendz_settings).post(save_trendz_settings))
        .route("/trendz/token",     get(get_trendz_token))
        .route("/trendz/exec",      post(exec_trendz_query))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendzSettings {
    pub base_url:        String,
    pub enabled:         bool,
    pub additional_info: Value,
}

impl Default for TrendzSettings {
    fn default() -> Self {
        Self {
            base_url:        String::new(),
            enabled:         false,
            additional_info: Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendzToken {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendzQueryResult {
    pub data: Value,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/trendz/settings — get Trendz settings (SYS_ADMIN only)
async fn get_trendz_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<TrendzSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let settings = state.admin_settings_dao
        .find_by_key(Uuid::nil(), "trendz")
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .and_then(|s| serde_json::from_value(s.json_value).ok())
        .unwrap_or_default();
    Ok(Json(settings))
}

/// POST /api/trendz/settings — save Trendz settings (SYS_ADMIN only)
async fn save_trendz_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<TrendzSettings>,
) -> Result<Json<TrendzSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let json_value = serde_json::to_value(&body)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let now = chrono::Utc::now().timestamp_millis();
    let s = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    Uuid::nil(),
        key:          "trendz".into(),
        json_value,
    };
    state.admin_settings_dao.save(&s).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(body))
}

/// GET /api/trendz/token — issue JWT for Trendz Analytics (token exchange)
/// Trả về token nếu Trendz được cấu hình và enabled, 501 nếu không.
async fn get_trendz_token(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<TrendzToken>, ApiError> {
    let trendz = &state.config.trendz;
    if !trendz.enabled || trendz.base_url.is_empty() {
        return Err(ApiError::NotImplemented(
            "Trendz Analytics is not configured on this server".into()
        ));
    }

    use jsonwebtoken::{encode, Header, EncodingKey};

    #[derive(serde::Serialize)]
    struct TrendzClaims {
        sub:         String,
        tenant_id:   String,
        customer_id: Option<String>,
        exp:         usize,
    }

    let exp = (chrono::Utc::now() + chrono::TimeDelta::hours(1)).timestamp() as usize;
    let claims = TrendzClaims {
        sub:         ctx.user_id.to_string(),
        tenant_id:   ctx.tenant_id.to_string(),
        customer_id: ctx.customer_id.map(|id| id.to_string()),
        exp,
    };

    let token: String = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(trendz.secret_key.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(TrendzToken { token }))
}

/// POST /api/trendz/exec — proxy query tới Trendz Analytics service
async fn exec_trendz_query(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<Value>,
) -> Result<Json<TrendzQueryResult>, ApiError> {
    let trendz = &state.config.trendz;
    if !trendz.enabled || trendz.base_url.is_empty() {
        return Err(ApiError::NotImplemented(
            "Trendz Analytics is not configured on this server".into()
        ));
    }

    // Lấy Trendz JWT token
    let trendz_token = get_trendz_token(
        State(state.clone()),
        Extension(ctx),
    ).await?.0.token;

    // Proxy request tới Trendz
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/query/processRequest", trendz.base_url))
        .bearer_auth(trendz_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Trendz request failed: {e}")))?;

    let data: Value = resp.json().await
        .map_err(|e| ApiError::Internal(format!("Trendz parse error: {e}")))?;

    Ok(Json(TrendzQueryResult { data }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_creation_does_not_panic() {
        let _r: Router<AppState> = router();
    }

    #[test]
    fn trendz_settings_default_values() {
        let settings = TrendzSettings::default();
        assert_eq!(settings.base_url, "");
        assert!(!settings.enabled);
        assert!(settings.additional_info.is_object());
    }

    #[test]
    fn trendz_settings_serializes_camel_case() {
        let settings = TrendzSettings {
            base_url: "http://localhost:8888".into(),
            enabled: true,
            additional_info: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["baseUrl"], "http://localhost:8888");
        assert_eq!(json["enabled"], true);
        assert_eq!(json["additionalInfo"]["key"], "value");
    }

    #[test]
    fn trendz_settings_deserializes_camel_case() {
        let json = r#"{"baseUrl":"https://trendz.example.com","enabled":true,"additionalInfo":{}}"#;
        let settings: TrendzSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.base_url, "https://trendz.example.com");
        assert!(settings.enabled);
    }

    #[test]
    fn trendz_token_serializes_camel_case() {
        let token = TrendzToken {
            token: "abc123".into(),
        };
        let json = serde_json::to_value(&token).unwrap();
        assert_eq!(json["token"], "abc123");
    }

    #[test]
    fn trendz_query_result_serializes_camel_case() {
        let result = TrendzQueryResult {
            data: serde_json::json!({"rows": [1, 2, 3]}),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["data"]["rows"], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn trendz_settings_roundtrip() {
        let original = TrendzSettings {
            base_url: "http://trendz:8888".into(),
            enabled: true,
            additional_info: serde_json::json!({"version": "3.5"}),
        };
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: TrendzSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.base_url, original.base_url);
        assert_eq!(deserialized.enabled, original.enabled);
        assert_eq!(deserialized.additional_info, original.additional_info);
    }
}
