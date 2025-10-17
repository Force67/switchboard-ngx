use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use switchboard_auth::{AuthSession, User};
use utoipa::{IntoParams, ToSchema};

use crate::{
    services::auth as auth_service,
    ApiError, AppState,
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
        (status = 503, description = "GitHub OAuth not configured", body = crate::error::ErrorResponse)
    )
)]
pub async fn github_login(
    State(state): State<AppState>,
    Query(params): Query<GithubLoginQuery>,
) -> Result<Json<GithubLoginResponse>, ApiError> {
    let authorize_url = auth_service::github_login_url(
        state.authenticator(),
        state.oauth_state(),
        params.redirect_uri,
    )
    .await
    .map_err(|e| {
        // Consume the oauth state if there was an error
        // Note: In a more sophisticated implementation, we might want to handle this differently
        ApiError::from(e)
    })?;

    Ok(Json(GithubLoginResponse { authorize_url }))
}

#[utoipa::path(
    post,
    path = "/api/auth/github/callback",
    tag = "Auth",
    request_body = GithubCallbackRequest,
    responses(
        (status = 200, description = "GitHub OAuth callback succeeded", body = SessionResponse),
        (status = 400, description = "Invalid OAuth payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication failed", body = crate::error::ErrorResponse),
        (status = 503, description = "GitHub OAuth not configured", body = crate::error::ErrorResponse)
    )
)]
pub async fn github_callback(
    State(state): State<AppState>,
    Json(payload): Json<GithubCallbackRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let (session, user) = auth_service::github_callback(
        state.authenticator(),
        state.oauth_state(),
        payload.code,
        payload.state,
        payload.redirect_uri,
    )
    .await
    .map_err(ApiError::from)?;

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
        (status = 500, description = "Failed to create development session", body = crate::error::ErrorResponse)
    )
)]
pub async fn dev_token(State(state): State<AppState>) -> Result<Json<SessionResponse>, ApiError> {
    let (session, user) = auth_service::create_dev_token(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create dev token: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(SessionResponse::new(session, user)))
}
