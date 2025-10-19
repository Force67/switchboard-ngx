//! Error types for the gateway layer

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Gateway error types
#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Service error: {0}")]
    ServiceError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl GatewayError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            GatewayError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            GatewayError::AuthorizationFailed(_) => StatusCode::FORBIDDEN,
            GatewayError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            GatewayError::NotFound(_) => StatusCode::NOT_FOUND,
            GatewayError::InternalError(_) | GatewayError::DatabaseError(_) | GatewayError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::WebSocketError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            GatewayError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = json!({
            "error": status.as_str(),
            "message": self.to_string(),
        });

        (status, Json(error_response)).into_response()
    }
}

/// Result type for gateway operations
pub type GatewayResult<T> = Result<T, GatewayError>;

/// Convert from common error types
impl From<switchboard_users::UserError> for GatewayError {
    fn from(error: switchboard_users::UserError) -> Self {
        match error {
            switchboard_users::UserError::UserNotFound => GatewayError::NotFound("User not found".to_string()),
            switchboard_users::UserError::AuthenticationFailed => GatewayError::AuthenticationFailed("Invalid credentials".to_string()),
            switchboard_users::UserError::AccountLocked => GatewayError::AuthorizationFailed("Account is locked".to_string()),
            switchboard_users::UserError::AccountSuspended => GatewayError::AuthorizationFailed("Account is suspended".to_string()),
            switchboard_users::UserError::InvalidEmail => GatewayError::InvalidRequest("Invalid email format".to_string()),
            switchboard_users::UserError::InvalidPassword => GatewayError::InvalidRequest("Invalid password".to_string()),
            switchboard_users::UserError::DatabaseError(msg) => GatewayError::DatabaseError(msg),
            switchboard_users::UserError::SerializationError(msg) => GatewayError::InternalError(format!("Serialization error: {}", msg)),
        }
    }
}

impl From<switchboard_chats::ChatError> for GatewayError {
    fn from(error: switchboard_chats::ChatError) -> Self {
        match error {
            switchboard_chats::ChatError::NotFound => GatewayError::NotFound("Chat not found".to_string()),
            switchboard_chats::ChatError::AccessDenied => GatewayError::AuthorizationFailed("Access denied".to_string()),
            switchboard_chats::ChatError::InvalidInput(msg) => GatewayError::InvalidRequest(msg),
            switchboard_chats::ChatError::RepositoryError(_) => GatewayError::ServiceError("Repository error".to_string()),
            switchboard_chats::ChatError::DatabaseError(msg) => GatewayError::DatabaseError(msg),
        }
    }
}

impl From<sqlx::Error> for GatewayError {
    fn from(error: sqlx::Error) -> Self {
        GatewayError::DatabaseError(error.to_string())
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for GatewayError {
    fn from(error: tokio::sync::broadcast::error::RecvError) -> Self {
        GatewayError::WebSocketError(format!("Broadcast receive error: {}", error))
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(error: serde_json::Error) -> Self {
        GatewayError::InvalidRequest(format!("JSON serialization error: {}", error))
    }
}