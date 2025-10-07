use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use denkwerk::LLMError;
use serde::Serialize;
use switchboard_auth::AuthError;
use switchboard_orchestrator::OrchestratorError;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        error!(error = ?error, "internal error");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    }
}

impl From<LLMError> for ApiError {
    fn from(error: LLMError) -> Self {
        error!(error = ?error, "llm error");
        Self::new(StatusCode::BAD_GATEWAY, error.to_string())
    }
}

impl From<OrchestratorError> for ApiError {
    fn from(error: OrchestratorError) -> Self {
        error!(error = ?error, "orchestrator error");
        let status = match error {
            OrchestratorError::ProviderNotFound(_) => StatusCode::BAD_REQUEST,
            OrchestratorError::OpenRouterApiKeyMissing
            | OrchestratorError::OpenRouterUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self::new(status, error.to_string())
    }
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        error!(error = ?error, "auth error");
        let status = match error {
            AuthError::GithubOauthDisabled => StatusCode::SERVICE_UNAVAILABLE,
            AuthError::GithubOauth(_) => StatusCode::BAD_GATEWAY,
            AuthError::InvalidCredentials
            | AuthError::SessionNotFound
            | AuthError::SessionExpired
            | AuthError::InvalidSession => StatusCode::UNAUTHORIZED,
            AuthError::UserExists => StatusCode::BAD_REQUEST,
            AuthError::Database(_) | AuthError::PasswordHash(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Self::new(status, error.to_string())
    }
}
