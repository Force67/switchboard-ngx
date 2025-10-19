//! Authentication service for managing auth operations.

use switchboard_database::{AuthSession, User, AuthResult, AuthError, UserError, UserRepository, SessionRepository, AuthProvider, CreateUserRequest, CreateSessionRequest};
use crate::services::{UserService, SessionService};
use sqlx::SqlitePool;

/// Service for managing authentication operations
pub struct AuthService {
    user_service: UserService<UserRepository>,
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
    pub fn with_services(user_service: UserService<UserRepository>, session_service: SessionService) -> Self {
        Self {
            user_service,
            session_service,
        }
    }

    /// Login user with email and password
    pub async fn login(&self, request: switchboard_database::types::LoginRequest) -> AuthResult<AuthSession> {
        // Find existing user by email
        let user = match self.user_service.get_user_by_email(&request.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return Err(AuthError::AuthenticationFailed);
            }
            Err(_) => return Err(AuthError::AuthenticationFailed),
        };

        // Create session
        let session = self.create_user_session(user.id, &None, &None).await?;

        // Update user's last login
        if let Err(e) = self.user_service.update_last_login(user.id).await {
            log::warn!("Failed to update last login for user {}: {}", user.id, e);
        }

        log::info!("User logged in: {:?} (ID: {})", user.email, user.id);
        Ok(session)
    }

    /// Register new user
    pub async fn register(&self, request: switchboard_database::types::RegisterRequest) -> AuthResult<(User, AuthSession)> {
        // Create user with email
        let create_request = CreateUserRequest {
            email: request.email.clone(),
            username: format!("user_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")),
            display_name: request.display_name.clone(),
            password: request.password.clone(),
            avatar_url: None,
            bio: None,
        };

        let user = self.user_service.create_user(create_request).await
            .map_err(|_| AuthError::AuthenticationFailed)?;

        // Create session for new user
        let session = self.create_user_session(user.id, &None, &None).await?;

        log::info!("User registered: {:?} (ID: {})", user.email, user.id);
        Ok((user, session))
    }

    /// Validate session token
    pub async fn validate_session(&self, token: &str) -> AuthResult<AuthSession> {
        // Validate token format (basic check)
        if token.trim().is_empty() {
            return Err(AuthError::InvalidToken);
        }

        // Check if session exists and is valid
        let session = self.session_service.validate_session(token)
            .await
            .map_err(|_| AuthError::AuthenticationFailed)?;

        Ok(session)
    }

    /// Logout user
    pub async fn logout(&self, token: &str) -> AuthResult<()> {
        // Validate token
        if token.trim().is_empty() {
            return Err(AuthError::InvalidToken);
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
            &None, // AuthSession doesn't have user_agent field
            &None, // AuthSession doesn't have ip_address field
        ).await?;

        // Delete old session
        self.session_service.delete_session(token).await?;

        log::info!("Session refreshed for user ID: {}", current_session.user_id);
        Ok(new_session)
    }

    /// Get user sessions
    pub async fn get_user_sessions(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        self.session_service.get_user_sessions(user_id).await
    }

    // Private helper methods

    /// Create a session for a user
    async fn create_user_session(&self, user_id: i64, _user_agent: &Option<String>, _ip_address: &Option<String>) -> AuthResult<AuthSession> {
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();

        let create_request = CreateSessionRequest {
            user_id,
            token,
            provider: AuthProvider::Email, // Default provider for now
            expires_at,
        };

        self.session_service.create_session(create_request).await
    }
}

#[cfg(test)]
mod tests {
    // Test implementation will be added later
}