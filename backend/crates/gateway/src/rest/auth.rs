//! Authentication REST endpoints

use axum::{
    extract::{Query, State, Request},
    Json,
    response::{IntoResponse, Response},
    middleware,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::state::GatewayState;
use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;

#[derive(Debug, Serialize, ToSchema)]
pub struct GithubLoginResponse {
    pub authorize_url: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl SessionResponse {
    pub fn new(session: switchboard_database::AuthSession, user: switchboard_database::User) -> Self {
        Self {
            token: session.token,
            user: user.into(),
            expires_at: session.expires_at,
        }
    }
}

impl From<switchboard_database::User> for UserResponse {
    fn from(user: switchboard_database::User) -> Self {
        Self {
            id: user.public_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        }
    }
}

/// Create authentication routes
pub fn create_auth_routes() -> Router<GatewayState> {
    Router::new()
        .route("/github/login", axum::routing::get(github_login))
        .route("/github/callback", axum::routing::post(github_callback))
        .route("/logout", axum::routing::post(logout))
        .route("/me", axum::routing::get(me))
        // Development endpoint (no auth required)
        .route("/dev/token", axum::routing::get(dev_token))
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
    State(state): State<GatewayState>,
) -> GatewayResult<Json<GithubLoginResponse>> {
    // TODO: Implement GitHub OAuth login
    // For now, return a placeholder response
    Err(GatewayError::ServiceUnavailable)
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
    State(state): State<GatewayState>,
) -> GatewayResult<Json<SessionResponse>> {
    // TODO: Implement GitHub OAuth callback
    // For now, return a placeholder response
    Err(GatewayError::ServiceUnavailable)
}

/// Development endpoint to create a test token
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
    State(state): State<GatewayState>,
) -> GatewayResult<Json<SessionResponse>> {
    let (session, user) = state
        .session_service()
        .create_dev_token()
        .await
        .map_err(|e| GatewayError::InternalError(format!("Failed to create dev token: {}", e)))?;

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
    State(state): State<GatewayState>,
    request: Request,
) -> GatewayResult<()> {
    let user_id = extract_user_id(&request)?;

    // For now, we don't have a direct way to logout by user_id
    // In a real implementation, you might want to invalidate all user sessions
    // or track active sessions more granularly

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
    State(state): State<GatewayState>,
    request: Request,
) -> GatewayResult<Json<UserResponse>> {
    let user_id = extract_user_id(&request)?;

    let user = state
        .user_service()
        .get_user(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?;

    Ok(Json(UserResponse::from(user)))
}