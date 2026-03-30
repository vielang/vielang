use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use vl_dao::DaoError;

/// Khớp Java: ThingsboardException response format
/// {"status": 404, "message": "...", "errorCode": 32}
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    TooManyRequests(String),
    NotImplemented(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::NotFound(msg)         => (StatusCode::NOT_FOUND, 32, msg.as_str()),
            ApiError::BadRequest(msg)       => (StatusCode::BAD_REQUEST, 2, msg.as_str()),
            ApiError::Unauthorized(msg)     => (StatusCode::UNAUTHORIZED, 10, msg.as_str()),
            ApiError::Forbidden(msg)        => (StatusCode::FORBIDDEN, 20, msg.as_str()),
            ApiError::Conflict(msg)         => (StatusCode::CONFLICT, 40, msg.as_str()),
            ApiError::TooManyRequests(msg)  => (StatusCode::TOO_MANY_REQUESTS, 429, msg.as_str()),
            ApiError::NotImplemented(msg)   => (StatusCode::NOT_IMPLEMENTED, 60, msg.as_str()),
            ApiError::Internal(msg)         => (StatusCode::INTERNAL_SERVER_ERROR, 50, msg.as_str()),
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let body = json!({
            "status": status.as_u16(),
            "message": message,
            "errorCode": error_code,
            "timestamp": timestamp,
        });

        (status, Json(body)).into_response()
    }
}

impl From<DaoError> for ApiError {
    fn from(err: DaoError) -> Self {
        match err {
            DaoError::NotFound            => ApiError::NotFound("Entity not found".into()),
            DaoError::Constraint(msg)     => ApiError::Conflict(msg),
            DaoError::Database(e)         => ApiError::Internal(e.to_string()),
            DaoError::Serialization(e)    => ApiError::Internal(e.to_string()),
            DaoError::Cassandra(e)        => ApiError::Internal(e),
            DaoError::InvalidInput(msg)   => ApiError::BadRequest(msg),
        }
    }
}
