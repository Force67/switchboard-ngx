use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use switchboard_auth::{AuthSession, User};

use crate::{ApiError, AppState};

#[derive(Debug, Serialize)]
pub struct GithubLoginResponse {
    pub authorize_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubLoginQuery {
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubCallbackRequest {
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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
