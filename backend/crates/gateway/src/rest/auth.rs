//! Authentication REST endpoints

use axum::{
    extract::{Query, Request, State},
    http::header,
    response::{IntoResponse, Response},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::state::GatewayState;

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
    pub fn new(session: switchboard_auth::AuthSession, user: switchboard_auth::User) -> Self {
        Self {
            token: session.token,
            user: user.into(),
            expires_at: session.expires_at.to_rfc3339(),
        }
    }
}

impl From<switchboard_auth::User> for UserResponse {
    fn from(user: switchboard_auth::User) -> Self {
        Self {
            id: user.public_id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: None,
        }
    }
}

/// Create authentication routes
pub fn create_auth_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/auth/github/login", axum::routing::get(github_login))
        .route(
            "/auth/github/callback",
            axum::routing::post(github_callback),
        )
        .route("/auth/logout", axum::routing::post(logout))
        .route("/auth/me", axum::routing::get(me))
        // Development endpoint (no auth required)
        .route("/auth/dev/token", axum::routing::get(dev_token))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/github/login",
    tag = "Auth",
    params(GithubLoginQuery),
    responses(
        (status = 200, description = "GitHub OAuth authorization URL", body = GithubLoginResponse),
        (status = 503, description = "GitHub OAuth not configured", body = ErrorResponse)
    )
)]
pub async fn github_login(
    Query(params): Query<GithubLoginQuery>,
    State(state): State<Arc<GatewayState>>,
) -> GatewayResult<Json<GithubLoginResponse>> {
    let login = state
        .authenticator()
        .github_authorization_url(&params.redirect_uri)
        .map_err(map_auth_error)?;

    Ok(Json(GithubLoginResponse {
        authorize_url: login.authorize_url,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/github/callback",
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
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<GithubCallbackRequest>,
) -> GatewayResult<Json<SessionResponse>> {
    let (session, user) = state
        .authenticator()
        .login_with_github_code(&payload.code, &payload.state, &payload.redirect_uri)
        .await
        .map_err(map_auth_error)?;

    Ok(Json(SessionResponse::new(session, user)))
}

/// Development endpoint to create a test token
#[cfg(debug_assertions)]
#[utoipa::path(
    get,
    path = "/api/v1/auth/dev/token",
    tag = "Auth",
    responses(
        (status = 200, description = "Development session issued", body = SessionResponse),
        (status = 500, description = "Failed to create development session", body = ErrorResponse)
    )
)]
pub async fn dev_token(
    State(state): State<Arc<GatewayState>>,
) -> GatewayResult<Json<SessionResponse>> {
    let (session, user) = state
        .authenticator()
        .create_dev_session()
        .await
        .map_err(|e| GatewayError::InternalError(format!("Failed to create dev token: {}", e)))?;

    Ok(Json(SessionResponse::new(session, user)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Successfully logged out"),
        (status = 401, description = "Invalid token", body = ErrorResponse),
        (status = 500, description = "Failed to logout", body = ErrorResponse)
    )
)]
pub async fn logout(State(state): State<Arc<GatewayState>>, request: Request) -> GatewayResult<()> {
    let token = extract_token(&request).ok_or_else(|| {
        GatewayError::AuthenticationFailed("Missing authentication token".to_string())
    })?;

    state
        .authenticator()
        .revoke_session(&token)
        .await
        .map_err(map_auth_error)?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "Auth",
    responses(
        (status = 200, description = "Current user information", body = UserResponse),
        (status = 401, description = "Invalid token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn me(
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<UserResponse>> {
    let user_id = extract_user_id(&request)?;

    let user = state
        .authenticator()
        .user_profile(user_id)
        .await
        .map_err(map_auth_error)?;

    Ok(Json(UserResponse::from(user)))
}

fn extract_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|value| value.to_string())
}

fn map_auth_error(error: switchboard_auth::AuthError) -> GatewayError {
    match error {
        switchboard_auth::AuthError::GithubOauthDisabled => GatewayError::ServiceUnavailable,
        switchboard_auth::AuthError::GithubOauth(err) => {
            GatewayError::AuthenticationFailed(format!("GitHub OAuth failed: {}", err))
        }
        switchboard_auth::AuthError::InvalidState
        | switchboard_auth::AuthError::InvalidCredentials
        | switchboard_auth::AuthError::SessionNotFound
        | switchboard_auth::AuthError::SessionExpired
        | switchboard_auth::AuthError::InvalidSession => {
            GatewayError::AuthenticationFailed(error.to_string())
        }
        switchboard_auth::AuthError::UserExists => GatewayError::InvalidRequest(error.to_string()),
        switchboard_auth::AuthError::Database(err) => GatewayError::DatabaseError(err.to_string()),
        switchboard_auth::AuthError::PasswordHash(_) => {
            GatewayError::InternalError(error.to_string())
        }
    }
}
