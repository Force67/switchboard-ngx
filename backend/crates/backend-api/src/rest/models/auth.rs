//! Authentication-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Response containing the GitHub OAuth authorization URL
#[derive(Debug, Serialize, ToSchema)]
pub struct GithubLoginResponse {
    pub authorize_url: String,
}

/// Query parameters for GitHub login
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GithubLoginQuery {
    pub redirect_uri: String,
}

/// Request body for GitHub OAuth callback
#[derive(Debug, Deserialize, ToSchema)]
pub struct GithubCallbackRequest {
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

/// Response containing session information after successful authentication
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub token: String,
    pub user: UserResponse,
    pub expires_at: String,
}

/// User information response
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
