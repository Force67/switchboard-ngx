use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use switchboard_auth::{AuthSession, User};
use utoipa::{IntoParams, ToSchema};

use crate::{ApiError, AppState};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.public_id,
            email: value.email,
            username: value.username,
            display_name: value.display_name,
            bio: value.bio,
            avatar_url: value.avatar_url,
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
    if !state.authenticator().github_enabled() {
        return Err(ApiError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "GitHub OAuth is not configured",
        ));
    }

    let oauth_state = state.oauth_state().issue().await;
    let authorize_url = match state
        .authenticator()
        .github_authorization_url(&oauth_state, &params.redirect_uri)
    {
        Ok(url) => url,
        Err(err) => {
            state.oauth_state().consume(&oauth_state).await;
            return Err(ApiError::from(err));
        }
    };

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
    if !state.oauth_state().consume(&payload.state).await {
        return Err(ApiError::bad_request("invalid or expired OAuth state"));
    }

    let session = state
        .authenticator()
        .login_with_github_code(&payload.code, &payload.redirect_uri)
        .await
        .map_err(ApiError::from)?;
    let user = state
        .authenticator()
        .user_profile(session.user_id)
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
    // Create a development user in the database first
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO users (id, public_id, email, display_name, username, avatar_url, bio, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(1i64)
    .bind("dev-user-123")
    .bind("dev@example.com")
    .bind("Dev User")
    .bind("devuser")
    .bind::<Option<&str>>(None)
    .bind::<Option<&str>>(None)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create dev user: {}", e);
        ApiError::internal_server_error("Failed to create dev user")
    })?;

    // Create a development session by inserting directly into the database
    let session_token = cuid2::create_id();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    sqlx::query(
        r#"
        INSERT INTO sessions (token, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&session_token)
    .bind(1i64)
    .bind(expires_at.to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create dev session: {}", e);
        ApiError::internal_server_error("Failed to create dev session")
    })?;

    // Create session and user objects
    let session = AuthSession {
        token: session_token.clone(),
        user_id: 1,
        expires_at,
    };

    let user = switchboard_auth::User {
        id: 1,
        public_id: "dev-user-123".to_string(),
        email: Some("dev@example.com".to_string()),
        username: Some("devuser".to_string()),
        display_name: Some("Dev User".to_string()),
        bio: None,
        avatar_url: None,
    };

    Ok(Json(SessionResponse::new(session, user)))
}
