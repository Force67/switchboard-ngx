//! Session service for managing user sessions.

use crate::entities::auth::{AuthSession, CreateSessionRequest};
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
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        const MAX_SESSIONS_PER_USER: u32 = 10;
        if active_sessions as u32 >= MAX_SESSIONS_PER_USER {
            // Delete oldest session to make room
            self.cleanup_oldest_sessions(request.user_id, 1).await?;
            log::warn!("User {} exceeded max sessions, cleaned up oldest session", request.user_id);
        }

        // Create session
        let session = self.session_repository.create(&request).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        log::info!("Created new session for user ID: {}", request.user_id);
        Ok(session)
    }

    /// Get session by token
    pub async fn get_session(&self, token: &str) -> AuthResult<Option<AuthSession>> {
        if token.trim().is_empty() {
            return Ok(None);
        }

        self.session_repository.find_by_token(token).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Validate and refresh session
    pub async fn validate_session(&self, token: &str) -> AuthResult<AuthSession> {
        let session = self.get_session(token).await?
            .ok_or(AuthError::InvalidSessionToken)?;

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
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

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
            .ok_or(AuthError::InvalidSessionToken)?;

        // Delete session
        self.session_repository.delete_by_token(token).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        log::info!("Deleted session: {}", &token[..8]);
        Ok(())
    }

    /// Delete all user sessions (logout from all devices)
    pub async fn delete_user_sessions(&self, user_id: i64) -> AuthResult<()> {
        let count = self.session_repository.delete_by_user_id(user_id).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        log::info!("Deleted {} sessions for user ID: {}", count, user_id);
        Ok(())
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        self.session_repository.find_by_user_id(user_id).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> AuthResult<u32> {
        let count = self.session_repository.delete_expired().await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if count > 0 {
            log::info!("Cleaned up {} expired sessions", count);
        }

        Ok(count)
    }

    /// Clean up old sessions for a user (when exceeding limit)
    async fn cleanup_oldest_sessions(&self, user_id: i64, count_to_delete: u32) -> AuthResult<()> {
        let sessions = self.session_repository.find_by_user_id(user_id).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Sort by created_at and delete oldest
        let mut sorted_sessions = sessions;
        sorted_sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for session in sorted_sessions.into_iter().take(count_to_delete as usize) {
            self.session_repository.delete_by_token(&session.token).await
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// Clean up specific expired session
    async fn cleanup_expired_session(&self, token: &str) -> AuthResult<()> {
        self.session_repository.delete_by_token(token).await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
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
        if request.user_id <= 0 {
            return Err(AuthError::InvalidSessionToken);
        }

        // Validate user agent if provided
        if let Some(ref user_agent) = request.user_agent {
            if user_agent.len() > 512 {
                return Err(AuthError::InvalidSessionToken);
            }
        }

        // Validate IP address if provided
        if let Some(ref ip_address) = request.ip_address {
            if ip_address.len() > 45 {
                return Err(AuthError::InvalidSessionToken);
            }
        }

        // Validate expiration if provided
        if let Some(expires_in) = request.expires_in_seconds {
            if expires_in == 0 || expires_in > 30 * 24 * 60 * 60 { // Max 30 days
                return Err(AuthError::InvalidSessionToken);
            }
        }

        Ok(())
    }

    /// Get session statistics (placeholder - would need total count method in repository)
    pub async fn get_session_stats(&self) -> AuthResult<SessionStats> {
        let expired_cleaned = self.cleanup_expired_sessions().await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

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
    // Test implementation will be added later
}