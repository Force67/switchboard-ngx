//! Authentication service for managing auth operations.

use crate::entities::auth::{AuthSession, LoginRequest, RegisterRequest, AuthProvider};
use crate::entities::user::User;
use crate::types::{AuthResult, UserResult};
use crate::types::errors::{AuthError, UserError};
use crate::repositories::{UserRepository, SessionRepository};
use crate::services::{UserService, SessionService};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Service for managing authentication operations
pub struct AuthService {
    user_service: UserService,
    session_service: SessionService,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            user_service: UserService::new(pool.clone()),
            session_service: SessionService::new(pool),
        }
    }

    /// Create auth service with custom services (for testing)
    pub fn with_services(user_service: UserService, session_service: SessionService) -> Self {
        Self {
            user_service,
            session_service,
        }
    }

    /// Login user
    pub async fn login(&self, request: LoginRequest) -> AuthResult<AuthSession> {
        // Validate login request
        if let Err(e) = request.validate() {
            return Err(AuthError::AuthenticationFailed(e));
        }

        // For OAuth-based login, we would validate the auth_code with the provider
        // This is a placeholder implementation
        let oauth_user_info = self.validate_oauth_token(request.provider, &request.auth_code).await?;

        // Find existing user or create a new one
        let user = match self.user_service.get_user_by_email(&oauth_user_info.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                // Create new user from OAuth
                let create_request = crate::entities::user::CreateUserRequest {
                    email: oauth_user_info.email,
                    display_name: oauth_user_info.display_name,
                    avatar_url: oauth_user_info.avatar_url,
                    role: Some(crate::entities::user::UserRole::User),
                };

                self.user_service.create_user(create_request).await
                    .map_err(|e| match e {
                        UserError::EmailAlreadyExists => AuthError::AuthenticationFailed("Email already exists".to_string()),
                        _ => AuthError::AuthenticationFailed(e.to_string()),
                    })?
            }
            Err(e) => return Err(AuthError::AuthenticationFailed(e.to_string())),
        };

        // Create session
        let session = self.create_user_session(user.id, &request.user_agent, &request.ip_address).await?;

        // Update user's last login
        if let Err(e) = self.user_service.update_last_login(user.id).await {
            log::warn!("Failed to update last login for user {}: {}", user.id, e);
        }

        log::info!("User logged in via {:?}: {:?} (ID: {})", request.provider, user.email, user.id);
        Ok(session)
    }

    /// Register new user
    pub async fn register(&self, request: RegisterRequest) -> AuthResult<(User, AuthSession)> {
        // Validate registration request
        if let Err(e) = request.validate() {
            return Err(AuthError::AuthenticationFailed(e));
        }

        // For OAuth-based registration, we would validate the auth_code with the provider
        // For email registration, we create user with password
        let user = match request.provider {
            AuthProvider::Email => {
                if request.password.is_none() {
                    return Err(AuthError::AuthenticationFailed("Password required for email registration".to_string()));
                }

                // Create user with email - this would need password support in the entity
                let create_request = crate::entities::user::CreateUserRequest {
                    email: request.email.clone(),
                    display_name: request.display_name.clone(),
                    avatar_url: None, // Not in RegisterRequest
                    role: Some(crate::entities::user::UserRole::User),
                };

                self.user_service.create_user(create_request).await
                    .map_err(|e| match e {
                        UserError::EmailAlreadyExists => AuthError::AuthenticationFailed("Email already exists".to_string()),
                        _ => AuthError::AuthenticationFailed(e.to_string()),
                    })?
            }
            _ => {
                // OAuth registration - provider_data should contain the necessary auth info
                let create_request = crate::entities::user::CreateUserRequest {
                    email: request.email.clone(),
                    display_name: request.display_name.clone(),
                    avatar_url: None, // Could be extracted from provider_data
                    role: Some(crate::entities::user::UserRole::User),
                };

                self.user_service.create_user(create_request).await
                    .map_err(|e| match e {
                        UserError::EmailAlreadyExists => AuthError::AuthenticationFailed("Email already exists".to_string()),
                        _ => AuthError::AuthenticationFailed(e.to_string()),
                    })?
            }
        };

        // Create session for new user
        let session = self.create_user_session(user.id, &request.user_agent, &request.ip_address).await?;

        log::info!("User registered via {:?}: {:?} (ID: {})", request.provider, user.email, user.id);
        Ok((user, session))
    }

    /// Validate session token
    pub async fn validate_session(&self, token: &str) -> AuthResult<AuthSession> {
        // Validate token format (basic check)
        if token.trim().is_empty() {
            return Err(AuthError::InvalidSessionToken);
        }

        // Check if session exists and is valid
        let session = self.session_service.validate_session(token)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        Ok(session)
    }

    /// Logout user
    pub async fn logout(&self, token: &str) -> AuthResult<()> {
        // Validate token
        if token.trim().is_empty() {
            return Err(AuthError::InvalidSessionToken);
        }

        // Delete session
        self.session_service.delete_session(token).await?;

        log::info!("User logged out with token: {}", &token[..8.min(token.len())]);
        Ok(())
    }

    /// Logout from all devices
    pub async fn logout_all(&self, user_id: i64) -> AuthResult<()> {
        self.session_service.delete_user_sessions(user_id).await?;

        log::info!("User logged out from all devices: ID {}", user_id);
        Ok(())
    }

    /// Refresh session
    pub async fn refresh_session(&self, token: &str) -> AuthResult<AuthSession> {
        // Validate current token
        let current_session = self.validate_session(token).await?;

        // Create new session
        let new_session = self.create_user_session(
            current_session.user_id,
            &current_session.user_agent,
            &current_session.ip_address,
        ).await?;

        // Delete old session
        self.session_service.delete_session(token).await?;

        log::info!("Session refreshed for user ID: {}", current_session.user_id);
        Ok(new_session)
    }

    /// OAuth login
    pub async fn oauth_login(&self, provider: AuthProvider, access_token: &str, user_agent: Option<String>, ip_address: Option<String>) -> AuthResult<AuthSession> {
        // Validate OAuth token and get user info
        let oauth_user_info = self.validate_oauth_token(provider, access_token).await?;

        // Find or create user
        let user = match self.user_service.get_user_by_email(&oauth_user_info.email).await {
            Ok(Some(user)) => {
                // User exists
                user
            }
            Ok(None) => {
                // Create new user from OAuth
                let create_request = crate::entities::user::CreateUserRequest {
                    email: oauth_user_info.email.clone(),
                    display_name: oauth_user_info.display_name,
                    avatar_url: oauth_user_info.avatar_url,
                    role: Some(crate::entities::user::UserRole::User),
                };
                self.user_service.create_user(create_request).await
                    .map_err(|e| AuthError::AuthenticationFailed(e.to_string()))?
            }
            Err(e) => return Err(AuthError::AuthenticationFailed(e.to_string())),
        };

        // Create session
        let session = self.create_user_session(user.id, &user_agent, &ip_address).await?;

        log::info!("OAuth login successful: {:?} via {:?} (ID: {})", user.email, provider, user.id);
        Ok(session)
    }

    /// Get user sessions
    pub async fn get_user_sessions(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        self.session_service.get_user_sessions(user_id).await
    }

    // Private helper methods

    /// Create a session for a user
    async fn create_user_session(&self, user_id: i64, user_agent: &Option<String>, ip_address: &Option<String>) -> AuthResult<AuthSession> {
        let create_request = crate::entities::auth::CreateSessionRequest {
            user_id,
            user_agent: user_agent.clone(),
            ip_address: ip_address.clone(),
            expires_in_seconds: Some(24 * 60 * 60), // 24 hours default
        };

        self.session_service.create_session(create_request).await
    }

    /// Validate OAuth token (placeholder implementation)
    async fn validate_oauth_token(&self, provider: AuthProvider, _access_token: &str) -> AuthResult<OAuthUserInfo> {
        // This is a placeholder implementation
        // In a real application, you would validate the OAuth token with the provider

        match provider {
            AuthProvider::Google => {
                // Simulate Google OAuth validation
                Ok(OAuthUserInfo {
                    email: "user@gmail.com".to_string(),
                    display_name: "Google User".to_string(),
                    avatar_url: Some("https://lh3.googleusercontent.com/avatar.jpg".to_string()),
                })
            }
            AuthProvider::GitHub => {
                // Simulate GitHub OAuth validation
                Ok(OAuthUserInfo {
                    email: "user@github.com".to_string(),
                    display_name: "GitHub User".to_string(),
                    avatar_url: Some("https://avatars.githubusercontent.com/u/1".to_string()),
                })
            }
            AuthProvider::Email => {
                Err(AuthError::AuthenticationFailed("Email provider doesn't support OAuth token validation".to_string()))
            }
            AuthProvider::Development => {
                // Development provider - accept any token
                Ok(OAuthUserInfo {
                    email: "dev@example.com".to_string(),
                    display_name: "Development User".to_string(),
                    avatar_url: None,
                })
            }
        }
    }
}

/// OAuth user information from provider
struct OAuthUserInfo {
    email: String,
    display_name: String,
    avatar_url: Option<String>,
}

#[cfg(test)]
mod tests {
    // Test implementation will be added later
}