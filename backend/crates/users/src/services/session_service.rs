//! Session service for managing user sessions.

use crate::entities::{AuthSession, CreateSessionRequest};
use crate::types::{AuthResult};
use crate::types::errors::AuthError;
use crate::repositories::SessionRepository;
use sqlx::SqlitePool;

/// Service for managing session operations
pub struct SessionService {
    session_repository: SessionRepository,
}

impl SessionService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            session_repository: SessionRepository::new(pool),
        }
    }

    /// Create session service with custom repository (for testing)
    pub fn with_repository(session_repository: SessionRepository) -> Self {
        Self { session_repository }
    }

    /// Create a new session
    pub async fn create_session(&self, request: CreateSessionRequest) -> AuthResult<AuthSession> {
        // Validate session request
        self.validate_create_session_request(&request)?;

        // Check if user has too many active sessions (configurable limit)
        let active_sessions = self.session_repository.count_active_sessions(request.user_id).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        const MAX_SESSIONS_PER_USER: u32 = 10;
        if active_sessions as u32 >= MAX_SESSIONS_PER_USER {
            // Delete oldest session to make room
            self.cleanup_oldest_sessions(request.user_id, 1).await?;
            log::warn!("User {} exceeded max sessions, cleaned up oldest session", request.user_id);
        }

        // Create session
        let session = self.session_repository.create(&request).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        log::info!("Created new session for user ID: {}", request.user_id);
        Ok(session)
    }

    /// Get session by token
    pub async fn get_session(&self, token: &str) -> AuthResult<Option<AuthSession>> {
        if token.trim().is_empty() {
            return Ok(None);
        }

        self.session_repository.find_by_token(token).await
            .map_err(|e| AuthError::InternalError(e.to_string()))
    }

    /// Validate and refresh session
    pub async fn validate_session(&self, token: &str) -> AuthResult<AuthSession> {
        let session = self.get_session(token).await?
            .ok_or(AuthError::InvalidSession)?;

        // Check if session is still active and not expired
        if !session.is_active {
            return Err(AuthError::SessionExpired);
        }

        // Check if session has expired
        if self.is_session_expired(&session) {
            self.cleanup_expired_session(&session.token).await?;
            return Err(AuthError::SessionExpired);
        }

        // Update last used time to extend session life
        self.session_repository.update_last_used(&session.token).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        Ok(session)
    }

    /// Invalidate session (alias for logout)
    pub async fn invalidate_session(&self, token: &str) -> AuthResult<()> {
        self.delete_session(token).await
    }

    /// Delete session (logout)
    pub async fn delete_session(&self, token: &str) -> AuthResult<()> {
        // Check if session exists first
        let _session = self.get_session(token).await?
            .ok_or(AuthError::InvalidSession)?;

        // Delete session
        self.session_repository.delete_by_token(token).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        log::info!("Deleted session: {}", &token[..8]);
        Ok(())
    }

    /// Delete all user sessions (logout from all devices)
    pub async fn delete_user_sessions(&self, user_id: i64) -> AuthResult<()> {
        let count = self.session_repository.delete_by_user_id(user_id).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        log::info!("Deleted {} sessions for user ID: {}", count, user_id);
        Ok(())
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        self.session_repository.find_by_user_id(user_id).await
            .map_err(|e| AuthError::InternalError(e.to_string()))
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> AuthResult<u32> {
        let count = self.session_repository.delete_expired().await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        if count > 0 {
            log::info!("Cleaned up {} expired sessions", count);
        }

        Ok(count)
    }

    /// Clean up old sessions for a user (when exceeding limit)
    async fn cleanup_oldest_sessions(&self, user_id: i64, count_to_delete: u32) -> AuthResult<()> {
        let sessions = self.session_repository.find_by_user_id(user_id).await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        // Sort by created_at and delete oldest
        let mut sorted_sessions = sessions;
        sorted_sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for session in sorted_sessions.into_iter().take(count_to_delete as usize) {
            self.session_repository.delete_by_token(&session.token).await
                .map_err(|e| AuthError::InternalError(e.to_string()))?;
        }

        Ok(())
    }

    /// Clean up specific expired session
    async fn cleanup_expired_session(&self, token: &str) -> AuthResult<()> {
        self.session_repository.delete_by_token(token).await
            .map_err(|e| AuthError::InternalError(e.to_string()))
    }

    /// Check if session has expired
    fn is_session_expired(&self, session: &AuthSession) -> bool {
        let expires_at = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok();

        if let Some(expires_at) = expires_at {
            chrono::Utc::now() > expires_at
        } else {
            // If we can't parse the expiration date, consider it expired
            true
        }
    }

    /// Validate session creation request
    fn validate_create_session_request(&self, request: &CreateSessionRequest) -> AuthResult<()> {
        if request.token.trim().is_empty() {
            return Err(AuthError::ValidationError("Session token cannot be empty".to_string()));
        }

        if request.user_id <= 0 {
            return Err(AuthError::ValidationError("Invalid user ID".to_string()));
        }

        // Validate token length
        if request.token.len() < 10 {
            return Err(AuthError::ValidationError("Session token too short".to_string()));
        }

        if request.token.len() > 512 {
            return Err(AuthError::ValidationError("Session token too long".to_string()));
        }

        // Validate device info if provided
        if let Some(ref device_info) = request.device_info {
            if let Some(ref user_agent) = device_info.user_agent {
                if user_agent.len() > 512 {
                    return Err(AuthError::ValidationError("User agent too long".to_string()));
                }
            }

            if let Some(ref ip_address) = device_info.ip_address {
                if ip_address.len() > 45 {
                    return Err(AuthError::ValidationError("IP address too long".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Get session statistics (placeholder - would need total count method in repository)
    pub async fn get_session_stats(&self) -> AuthResult<SessionStats> {
        let expired_cleaned = self.cleanup_expired_sessions().await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        // For now, return 0 for active sessions since we don't have a total count method
        Ok(SessionStats {
            active_sessions: 0,
            expired_cleaned,
        })
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub active_sessions: i64,
    pub expired_cleaned: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SessionRepository;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn create_test_session_service() -> (SessionService, SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_sessions.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE auth_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token TEXT NOT NULL UNIQUE,
                user_id INTEGER NOT NULL,
                device_info TEXT,
                ip_address TEXT,
                user_agent TEXT,
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        let service = SessionService::new(pool.clone());
        (service, pool, temp_dir)
    }

    fn create_valid_session_request() -> CreateSessionRequest {
        CreateSessionRequest {
            token: "sess_1234567890abcdef".to_string(),
            user_id: 1,
            device_info: Some(crate::entities::DeviceInfo {
                user_agent: Some("Mozilla/5.0".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                device_type: Some("desktop".to_string()),
                platform: Some("linux".to_string()),
            }),
        }
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let request = create_valid_session_request();

        let session = service.create_session(request).await.unwrap();

        assert_eq!(session.token, "sess_1234567890abcdef");
        assert_eq!(session.user_id, 1);
        assert!(session.is_active);
        assert!(session.id.is_some());
    }

    #[tokio::test]
    async fn test_create_session_invalid_token() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let mut request = create_valid_session_request();
        request.token = "".to_string();

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_create_session_invalid_user_id() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let mut request = create_valid_session_request();
        request.user_id = 0;

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_get_session_success() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let request = create_valid_session_request();

        let created = service.create_session(request).await.unwrap();
        let found = service.get_session(&created.token).await.unwrap();

        assert!(found.is_some());
        let found_session = found.unwrap();
        assert_eq!(found_session.token, created.token);
        assert_eq!(found_session.user_id, created.user_id);
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        let result = service.get_session("nonexistent_token").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_validate_session_success() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let request = create_valid_session_request();

        let created = service.create_session(request).await.unwrap();
        let validated = service.validate_session(&created.token).await.unwrap();

        assert_eq!(validated.token, created.token);
        assert_eq!(validated.user_id, created.user_id);
    }

    #[tokio::test]
    async fn test_validate_session_invalid() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        let result = service.validate_session("invalid_token").await;
        assert!(matches!(result, Err(AuthError::InvalidSession)));
    }

    #[tokio::test]
    async fn test_delete_session_success() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let request = create_valid_session_request();

        let created = service.create_session(request).await.unwrap();
        service.delete_session(&created.token).await.unwrap();

        let result = service.get_session(&created.token).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        let result = service.delete_session("nonexistent_token").await;
        assert!(matches!(result, Err(AuthError::InvalidSession)));
    }

    #[tokio::test]
    async fn test_get_user_sessions() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let user_id = 1;

        // Create multiple sessions for the same user
        let mut request1 = create_valid_session_request();
        request1.token = "sess_token1".to_string();
        request1.user_id = user_id;

        let mut request2 = create_valid_session_request();
        request2.token = "sess_token2".to_string();
        request2.user_id = user_id;

        service.create_session(request1).await.unwrap();
        service.create_session(request2).await.unwrap();

        let sessions = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().any(|s| s.token == "sess_token1"));
        assert!(sessions.iter().any(|s| s.token == "sess_token2"));
    }

    #[tokio::test]
    async fn test_delete_user_sessions() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let user_id = 1;

        // Create multiple sessions for the same user
        let mut request1 = create_valid_session_request();
        request1.token = "sess_token1".to_string();
        request1.user_id = user_id;

        let mut request2 = create_valid_session_request();
        request2.token = "sess_token2".to_string();
        request2.user_id = user_id;

        service.create_session(request1).await.unwrap();
        service.create_session(request2).await.unwrap();

        // Delete all user sessions
        service.delete_user_sessions(user_id).await.unwrap();

        // Verify all sessions are deleted
        let sessions = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        // Create a session that's already expired by setting an old expiration time
        let mut request = create_valid_session_request();
        request.token = "expired_token".to_string();

        let session = service.create_session(request).await.unwrap();

        // Manually expire the session in the database
        let past_time = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        sqlx::query("UPDATE auth_sessions SET expires_at = ? WHERE token = ?")
            .bind(&past_time)
            .bind(&session.token)
            .execute(&service.session_repository.pool)
            .await
            .unwrap();

        // Run cleanup
        let cleaned_count = service.cleanup_expired_sessions().await.unwrap();
        assert_eq!(cleaned_count, 1);

        // Verify expired session is deleted
        let result = service.get_session(&session.token).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_max_sessions_per_user() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let user_id = 1;

        // Create the maximum number of sessions (10)
        for i in 0..10 {
            let mut request = create_valid_session_request();
            request.token = format!("sess_token_{}", i);
            request.user_id = user_id;
            service.create_session(request).await.unwrap();
        }

        // Verify we have exactly 10 sessions
        let sessions_before = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(sessions_before.len(), 10);

        // Try to create one more session (should delete the oldest)
        let mut request = create_valid_session_request();
        request.token = "sess_token_newest".to_string();
        request.user_id = user_id;

        let new_session = service.create_session(request).await.unwrap();

        // Should still have at most 10 sessions
        let sessions_after = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(sessions_after.len(), 10);
        assert!(sessions_after.iter().any(|s| s.token == "sess_token_newest"));

        // The oldest session should be deleted
        assert!(!sessions_after.iter().any(|s| s.token == "sess_token_0"));
        assert!(sessions_after.iter().any(|s| s.token == "sess_token_1"));
    }

    #[tokio::test]
    async fn test_session_creation_with_device_info() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let mut request = create_valid_session_request();

        request.device_info = Some(crate::entities::DeviceInfo {
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()),
            ip_address: Some("192.168.1.100".to_string()),
            device_type: Some("desktop".to_string()),
            platform: Some("windows".to_string()),
        });

        let session = service.create_session(request).await.unwrap();

        assert!(session.device_info.is_some());
        assert_eq!(session.ip_address, Some("192.168.1.100".to_string()));
        assert_eq!(session.user_agent, Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()));
    }

    #[tokio::test]
    async fn test_session_creation_without_device_info() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let mut request = create_valid_session_request();
        request.device_info = None;

        let session = service.create_session(request).await.unwrap();

        assert!(session.device_info.is_none());
        assert!(session.ip_address.is_none());
        assert!(session.user_agent.is_none());
    }

    #[tokio::test]
    async fn test_session_validation_updates_last_used() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let request = create_valid_session_request();

        let created_session = service.create_session(request).await.unwrap();
        let original_updated_at = created_session.updated_at.clone();

        // Give a small delay to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Validate session (should update last_used)
        let validated_session = service.validate_session(&created_session.token).await.unwrap();

        assert_eq!(validated_session.token, created_session.token);
        assert_ne!(validated_session.updated_at, original_updated_at);
    }

    #[tokio::test]
    async fn test_multiple_users_sessions_isolated() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        // Create sessions for different users
        let user1_id = 1;
        let user2_id = 2;

        let mut request1 = create_valid_session_request();
        request1.token = "user1_session".to_string();
        request1.user_id = user1_id;

        let mut request2 = create_valid_session_request();
        request2.token = "user2_session".to_string();
        request2.user_id = user2_id;

        let session1 = service.create_session(request1).await.unwrap();
        let session2 = service.create_session(request2).await.unwrap();

        // Each user should only see their own sessions
        let user1_sessions = service.get_user_sessions(user1_id).await.unwrap();
        let user2_sessions = service.get_user_sessions(user2_id).await.unwrap();

        assert_eq!(user1_sessions.len(), 1);
        assert_eq!(user2_sessions.len(), 1);
        assert_eq!(user1_sessions[0].token, "user1_session");
        assert_eq!(user2_sessions[0].token, "user2_session");
    }

    #[tokio::test]
    async fn test_session_creation_with_different_token_lengths() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        // Test minimum valid length (10 characters)
        let mut request = create_valid_session_request();
        request.token = "sess_12345".to_string(); // 11 characters
        request.user_id = 1;

        let session = service.create_session(request).await.unwrap();
        assert_eq!(session.token, "sess_12345");

        // Test maximum valid length (512 characters)
        let mut request = create_valid_session_request();
        request.token = "sess_".to_string() + &"a".repeat(506); // 511 characters
        request.user_id = 2;

        let session = service.create_session(request).await.unwrap();
        assert_eq!(session.token.len(), 511);

        // Test too short token
        let mut request = create_valid_session_request();
        request.token = "short".to_string();
        request.user_id = 3;

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));

        // Test too long token
        let mut request = create_valid_session_request();
        request.token = "sess_".to_string() + &"a".repeat(600); // 605 characters
        request.user_id = 4;

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_cleanup_multiple_expired_sessions() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;

        // Create multiple sessions that will be expired
        let mut expired_tokens = Vec::new();
        for i in 0..5 {
            let mut request = create_valid_session_request();
            request.token = format!("expired_token_{}", i);
            let session = service.create_session(request).await.unwrap();
            expired_tokens.push(session.token.clone());

            // Manually expire the session in the database
            let past_time = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
            sqlx::query("UPDATE auth_sessions SET expires_at = ? WHERE token = ?")
                .bind(&past_time)
                .bind(&session.token)
                .execute(&service.session_repository.pool)
                .await
                .unwrap();
        }

        // Create one valid session
        let mut request = create_valid_session_request();
        request.token = "valid_session".to_string();
        let valid_session = service.create_session(request).await.unwrap();

        // Run cleanup
        let cleaned_count = service.cleanup_expired_sessions().await.unwrap();
        assert_eq!(cleaned_count, 5);

        // Verify expired sessions are deleted
        for token in expired_tokens {
            let result = service.get_session(&token).await.unwrap();
            assert!(result.is_none());
        }

        // Verify valid session still exists
        let result = service.get_session(&valid_session.token).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_complete_session_lifecycle() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let user_id = 1;

        // Create multiple sessions
        let mut sessions = Vec::new();
        for i in 0..3 {
            let mut request = create_valid_session_request();
            request.token = format!("lifecycle_session_{}", i);
            request.user_id = user_id;
            let session = service.create_session(request).await.unwrap();
            sessions.push(session);
        }

        // Verify all sessions exist
        let all_sessions = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(all_sessions.len(), 3);

        // Validate each session (should update last_used)
        for session in &sessions {
            let validated = service.validate_session(&session.token).await.unwrap();
            assert_eq!(validated.user_id, user_id);
        }

        // Delete one session
        service.delete_session(&sessions[0].token).await.unwrap();
        let remaining_sessions = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(remaining_sessions.len(), 2);

        // Delete all remaining sessions
        service.delete_user_sessions(user_id).await.unwrap();
        let final_sessions = service.get_user_sessions(user_id).await.unwrap();
        assert_eq!(final_sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_session_with_invalid_device_info() {
        let (service, _pool, _temp_dir) = create_test_session_service().await;
        let mut request = create_valid_session_request();

        // Test with too long user agent
        request.device_info = Some(crate::entities::DeviceInfo {
            user_agent: Some("a".repeat(600)), // Too long
            ip_address: Some("127.0.0.1".to_string()),
            device_type: Some("desktop".to_string()),
            platform: Some("linux".to_string()),
        });

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));

        // Test with too long IP address
        let mut request = create_valid_session_request();
        request.device_info = Some(crate::entities::DeviceInfo {
            user_agent: Some("Mozilla/5.0".to_string()),
            ip_address: Some("a".repeat(50)), // Too long
            device_type: Some("desktop".to_string()),
            platform: Some("linux".to_string()),
        });

        let result = service.create_session(request).await;
        assert!(matches!(result, Err(AuthError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_session_expiration_edge_cases() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let service = SessionService::new(pool);

        // Test with invalid expiration date format
        let session_with_invalid_date = AuthSession {
            id: Some(1),
            token: "invalid_date_token".to_string(),
            user_id: 1,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_active: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: "invalid_date".to_string(),
        };

        // Should be considered expired if date can't be parsed
        assert!(service.is_session_expired(&session_with_invalid_date));

        // Test with session exactly at expiration time (edge case)
        let now = chrono::Utc::now();
        let session_at_expiration = AuthSession {
            id: Some(2),
            token: "expiring_now_token".to_string(),
            user_id: 1,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_active: true,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expires_at: now.to_rfc3339(),
        };

        // Should be considered expired if time is exactly at expiration
        assert!(service.is_session_expired(&session_at_expiration));
    }

    #[test]
    fn test_validate_create_session_request() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let service = SessionService::new(pool);

        // Valid request
        let valid_request = create_valid_session_request();
        assert!(service.validate_create_session_request(&valid_request).is_ok());

        // Invalid token
        let mut invalid_request = create_valid_session_request();
        invalid_request.token = "".to_string();
        assert!(service.validate_create_session_request(&invalid_request).is_err());

        // Invalid user ID
        invalid_request = create_valid_session_request();
        invalid_request.user_id = 0;
        assert!(service.validate_create_session_request(&invalid_request).is_err());

        // Token too short
        invalid_request = create_valid_session_request();
        invalid_request.token = "short".to_string();
        assert!(service.validate_create_session_request(&invalid_request).is_err());

        // Token too long
        invalid_request = create_valid_session_request();
        invalid_request.token = "a".repeat(600);
        assert!(service.validate_create_session_request(&invalid_request).is_err());
    }

    #[test]
    fn test_is_session_expired() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let service = SessionService::new(pool);

        // Expired session
        let expired_session = AuthSession {
            id: Some(1),
            token: "expired_token".to_string(),
            user_id: 1,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_active: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339(),
        };

        assert!(service.is_session_expired(&expired_session));

        // Valid session
        let valid_session = AuthSession {
            id: Some(2),
            token: "valid_token".to_string(),
            user_id: 1,
            device_info: None,
            ip_address: None,
            user_agent: None,
            is_active: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
        };

        assert!(!service.is_session_expired(&valid_session));
    }
}