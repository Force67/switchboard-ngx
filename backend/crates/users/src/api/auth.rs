//! Authentication API endpoints

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    services::{auth::AuthService, session::SessionService},
    UserError, UserResult,
    types::{User, AuthSession},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct GithubLoginResponse {
    pub authorize_url: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GithubLoginQuery {
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GithubCallbackRequest {
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub token: String,
    pub user: UserResponse,
    pub expires_at: String,
}

impl SessionResponse {
    pub fn new(session: AuthSession, user: User) -> Self {
        Self {
            token: session.token,
            user: user.into(),
            expires_at: session.expires_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.public_id,
            email: value.email,
            display_name: value.display_name,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/auth/github/login",
    tag = "Auth",
    params(GithubLoginQuery),
    responses(
        (status = 200, description = "GitHub OAuth authorization URL", body = GithubLoginResponse),
        (status = 503, description = "GitHub OAuth not configured", body = ErrorResponse)
    )
)]
pub async fn github_login(
    Query(params): Query<GithubLoginQuery>,
    auth_service: State<AuthService>,
) -> Result<Json<GithubLoginResponse>, ErrorResponse> {
    let authorize_url = auth_service
        .github_login_url(&params.redirect_uri)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(GithubLoginResponse { authorize_url }))
}

#[utoipa::path(
    post,
    path = "/api/auth/github/callback",
    tag = "Auth",
    request_body = GithubCallbackRequest,
    responses(
        (status = 200, description = "GitHub OAuth callback succeeded", body = SessionResponse),
        (status = 400, description = "Invalid OAuth payload", body = ErrorResponse),
        (status = 401, description = "Authentication failed", body = ErrorResponse),
        (status = 503, description = "GitHub OAuth not configured", body = ErrorResponse)
    )
)]
pub async fn github_callback(
    Json(payload): Json<GithubCallbackRequest>,
    auth_service: State<AuthService>,
) -> Result<Json<SessionResponse>, ErrorResponse> {
    let (session, user) = auth_service
        .github_callback(&payload.code, &payload.state, &payload.redirect_uri)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(SessionResponse::new(session, user)))
}

// Development endpoint to create a test token
#[cfg(debug_assertions)]
#[utoipa::path(
    get,
    path = "/api/auth/dev/token",
    tag = "Auth",
    responses(
        (status = 200, description = "Development session issued", body = SessionResponse),
        (status = 500, description = "Failed to create development session", body = ErrorResponse)
    )
)]
pub async fn dev_token(
    session_service: State<SessionService>,
) -> Result<Json<SessionResponse>, ErrorResponse> {
    let (session, user) = session_service
        .create_dev_token()
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(SessionResponse::new(session, user)))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Successfully logged out"),
        (status = 401, description = "Invalid token", body = ErrorResponse),
        (status = 500, description = "Failed to logout", body = ErrorResponse)
    )
)]
pub async fn logout(
    session_service: State<SessionService>,
    token: String,
) -> Result<(), ErrorResponse> {
    session_service
        .delete_session(&token)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn me(
    session_service: State<SessionService>,
    user_service: State<crate::services::user::UserService<crate::services::repositories::UserRepository>>,
    token: String,
) -> Result<Json<UserResponse>, ErrorResponse> {
    // Validate session
    let session = session_service
        .find_by_token(&token)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| UserError::AuthenticationFailed)?;

    // Get user
    let user = user_service
        .get_user(session.user_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(UserResponse::from(user)))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<&UserError> for ErrorResponse {
    fn from(error: &UserError) -> Self {
        match error {
            UserError::UserNotFound => Self {
                error: "USER_NOT_FOUND".to_string(),
                message: "User not found".to_string(),
            },
            UserError::AuthenticationFailed => Self {
                error: "AUTHENTICATION_FAILED".to_string(),
                message: "Authentication failed".to_string(),
            },
            UserError::DatabaseError(msg) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Internal error: {}", msg),
            },
            UserError::InvalidEmail => Self {
                error: "INVALID_EMAIL".to_string(),
                message: "Invalid email format".to_string(),
            },
            UserError::InvalidPassword => Self {
                error: "INVALID_PASSWORD".to_string(),
                message: "Invalid password".to_string(),
            },
            UserError::AccountLocked => Self {
                error: "ACCOUNT_LOCKED".to_string(),
                message: "Account is locked".to_string(),
            },
            UserError::AccountSuspended => Self {
                error: "ACCOUNT_SUSPENDED".to_string(),
                message: "Account is suspended".to_string(),
            },
            UserError::SerializationError(msg) => Self {
                error: "SERIALIZATION_ERROR".to_string(),
                message: format!("Serialization error: {}", msg),
            },
        }
    }
}

impl From<UserError> for ErrorResponse {
    fn from(error: UserError) -> Self {
        Self::from(&error)
    }
}