//! Authentication service for user management

use sqlx::SqlitePool;
use crate::{
    types::{User, AuthSession},
    UserError, UserResult,
    repositories::{UserRepository, SessionRepository},
    utils::jwt::{JwtManager},
};
use chrono::{Utc, Duration};

/// Authentication service
pub struct AuthService {
    user_repository: UserRepository,
    session_repository: SessionRepository,
    jwt_manager: JwtManager,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(pool: SqlitePool, jwt_secret: &str, jwt_issuer: String, jwt_audience: String) -> Self {
        let jwt_manager = JwtManager::new(jwt_secret, jwt_issuer, jwt_audience);

        Self {
            user_repository: UserRepository::new(pool.clone()),
            session_repository: SessionRepository::new(pool),
            jwt_manager,
        }
    }

    /// Authenticate user with email and password
    pub async fn authenticate(&self, email: &str, password: &str) -> UserResult<(AuthSession, User)> {
        // Find user by email
        let user = self.user_repository
            .find_by_email(email)
            .await?
            .ok_or(UserError::AuthenticationFailed)?;

        // In a real implementation, you would hash and verify the password here
        // For now, we'll just check if the user exists and is active
        if !user.is_active {
            return Err(UserError::AccountLocked);
        }

        // Create session
        let session = self.create_session(user.id).await?;

        Ok((session, user))
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: i64) -> UserResult<AuthSession> {
        let token = self.jwt_manager.generate_token(
            &user_id.to_string(),
            &crate::utils::jwt::generate_session_id(),
            "user",
        )?;

        let expires_at = Utc::now() + Duration::hours(24); // 24 hour session

        let session = self.session_repository
            .create(&crate::types::CreateSessionRequest {
                token: token.clone(),
                user_id,
                expires_at: expires_at.to_rfc3339(),
                auth_provider: crate::types::AuthProvider::Local,
            })
            .await?;

        Ok(session)
    }

    /// Validate a session token
    pub async fn validate_session(&self, token: &str) -> UserResult<AuthSession> {
        // First validate JWT
        let claims = self.jwt_manager.validate_token(token)
            .map_err(|_| UserError::AuthenticationFailed)?;

        // Then check if session exists in database
        let session = self.session_repository
            .find_by_token(token)
            .await?
            .ok_or(UserError::AuthenticationFailed)?;

        // Check if session is expired
        let expires_at = session.expires_at.parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|_| UserError::AuthenticationFailed)?;

        if expires_at < Utc::now() {
            return Err(UserError::AuthenticationFailed);
        }

        Ok(session)
    }

    /// Invalidate a session (logout)
    pub async fn invalidate_session(&self, token: &str) -> UserResult<()> {
        self.session_repository.delete_by_token(token).await
    }

    /// Get user by session token
    pub async fn get_user_by_token(&self, token: &str) -> UserResult<User> {
        let session = self.validate_session(token).await?;
        self.user_repository
            .find_by_id(session.user_id)
            .await?
            .ok_or(UserError::UserNotFound)
    }

    /// Refresh a session token
    pub async fn refresh_token(&self, token: &str) -> UserResult<String> {
        let claims = self.jwt_manager.validate_token(token)
            .map_err(|_| UserError::AuthenticationFailed)?;

        // Check if session exists and is not expired
        let session = self.session_repository
            .find_by_token(token)
            .await?
            .ok_or(UserError::AuthenticationFailed)?;

        let expires_at = session.expires_at.parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|_| UserError::AuthenticationFailed)?;

        if expires_at < Utc::now() {
            return Err(UserError::AuthenticationFailed);
        }

        // Generate new token
        let new_token = self.jwt_manager.generate_token(
            &claims.sub,
            &claims.session_id,
            &claims.user_role,
        )?;

        // Update session with new token
        let new_expires_at = Utc::now() + Duration::hours(24);
        self.session_repository
            .update_token(&session.token, &new_token, &new_expires_at.to_rfc3339())
            .await?;

        Ok(new_token)
    }

    /// Create a development token for testing
    pub async fn create_dev_token(&self) -> UserResult<(AuthSession, User)> {
        // Ensure dev user exists
        let dev_user_id = 1i64;
        let dev_public_id = "dev-user-123";
        let dev_email = "dev@example.com";
        let dev_display_name = "Dev User";

        // Check if dev user exists, create if not
        let user = match self.user_repository.find_by_id(dev_user_id).await? {
            Some(user) => user,
            None => {
                // Create dev user
                let create_req = crate::types::CreateUserRequest {
                    email: dev_email.to_string(),
                    username: "dev-user".to_string(),
                    display_name: dev_display_name.to_string(),
                    password: "dev-password".to_string(),
                    avatar_url: None,
                    bio: None,
                };

                self.user_repository.create(&create_req).await?
            }
        };

        // Create session for dev user
        let session = self.create_session(user.id).await?;

        Ok((session, user))
    }

    /// Extract session ID from token (for performance)
    pub fn extract_session_id(&self, token: &str) -> UserResult<String> {
        self.jwt_manager.extract_session_id(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use sqlx::SqlitePool;

    async fn create_test_service() -> (AuthService, SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_auth.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                email TEXT UNIQUE,
                username TEXT UNIQUE,
                display_name TEXT,
                avatar_url TEXT,
                bio TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                role TEXT NOT NULL DEFAULT 'user',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_login_at TEXT,
                email_verified BOOLEAN NOT NULL DEFAULT false,
                is_active BOOLEAN NOT NULL DEFAULT true,
                password_hash TEXT
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token TEXT NOT NULL UNIQUE,
                user_id INTEGER NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                auth_provider TEXT NOT NULL DEFAULT 'local',
                last_used_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        let auth_service = AuthService::new(
            pool.clone(),
            "test_secret_key_that_is_long_enough_for_hs256",
            "test_issuer".to_string(),
            "test_audience".to_string(),
        );

        (auth_service, pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_dev_token() {
        let (auth_service, _pool, _temp_dir) = create_test_service().await;

        let result = auth_service.create_dev_token().await;

        assert!(result.is_ok());
        let (session, user) = result.unwrap();
        assert!(session.user_id > 0);
        assert!(!session.token.is_empty());
        assert_eq!(user.public_id, "dev-user-123");
        assert_eq!(user.email.unwrap(), "dev@example.com");
        assert_eq!(user.display_name.unwrap(), "Dev User");
    }

    #[tokio::test]
    async fn test_validate_session() {
        let (auth_service, _pool, _temp_dir) = create_test_service().await;

        // Create a dev token first
        let (session, _) = auth_service.create_dev_token().await.unwrap();

        // Validate the session
        let result = auth_service.validate_session(&session.token).await;

        assert!(result.is_ok());
        let validated_session = result.unwrap();
        assert_eq!(validated_session.token, session.token);
        assert_eq!(validated_session.user_id, session.user_id);
    }

    #[tokio::test]
    async fn test_extract_session_id() {
        let (auth_service, _pool, _temp_dir) = create_test_service().await;

        // Create a dev token first
        let (session, _) = auth_service.create_dev_token().await.unwrap();

        // Extract session ID
        let result = auth_service.extract_session_id(&session.token).await;

        assert!(result.is_ok());
        let session_id = result.unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_invalidate_session() {
        let (auth_service, _pool, _temp_dir) = create_test_service().await;

        // Create a dev token first
        let (session, _) = auth_service.create_dev_token().await.unwrap();

        // Invalidate the session
        let result = auth_service.invalidate_session(&session.token).await;

        assert!(result.is_ok());

        // Try to validate the invalidated session
        let validate_result = auth_service.validate_session(&session.token).await;
        assert!(validate_result.is_err());
    }
}