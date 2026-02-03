//! Auth service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::AuthServiceTrait;
use crate::rest::models::{
    GithubCallbackRequest, GithubLoginQuery, GithubLoginResponse, SessionResponse, UserResponse,
};
use crate::state::GatewayState;

/// Implementation of AuthServiceTrait
pub struct AuthServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> AuthServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
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

impl AuthServiceTrait for AuthServiceImpl<'_> {
    async fn github_login(&self, query: GithubLoginQuery) -> GatewayResult<GithubLoginResponse> {
        let login = self
            .state
            .authenticator()
            .github_authorization_url(&query.redirect_uri)
            .map_err(map_auth_error)?;

        Ok(GithubLoginResponse {
            authorize_url: login.authorize_url,
        })
    }

    async fn github_callback(&self, req: GithubCallbackRequest) -> GatewayResult<SessionResponse> {
        let (session, user) = self
            .state
            .authenticator()
            .login_with_github_code(&req.code, &req.state, &req.redirect_uri)
            .await
            .map_err(map_auth_error)?;

        Ok(SessionResponse::new(session, user))
    }

    async fn logout(&self, _user_id: i64) -> GatewayResult<()> {
        // Note: The generated handler passes user_id, but proper logout requires the token.
        // The original auth handler (rest/auth.rs) should be used for token-based logout.
        // This generated endpoint is a no-op - use /auth/logout from the original routes.
        Ok(())
    }

    async fn me(&self, user_id: i64) -> GatewayResult<UserResponse> {
        let user = self
            .state
            .authenticator()
            .user_profile(user_id)
            .await
            .map_err(map_auth_error)?;

        Ok(UserResponse::from(user))
    }
}
