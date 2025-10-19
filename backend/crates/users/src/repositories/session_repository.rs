//! Session repository for database operations.

use crate::entities::{AuthSession, CreateSessionRequest};
use crate::types::{AuthResult};
use crate::types::errors::AuthError;
use sqlx::SqlitePool;

/// Repository for session database operations
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    /// Create a new session repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find session by database ID
    async fn find_by_id(&self, id: i64) -> AuthResult<Option<AuthSession>> {
        let row = sqlx::query(
            "SELECT id, token, user_id, device_info, ip_address, user_agent, is_active, created_at, updated_at, expires_at
             FROM auth_sessions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let device_info: Option<String> = row.try_get("device_info")
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

            let device_info_parsed = device_info
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(AuthSession {
                id: Some(row.try_get("id").map_err(|e| AuthError::DatabaseError(e.to_string()))?),
                token: row.try_get("token").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                device_info: device_info_parsed,
                ip_address: row.try_get("ip_address").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_agent: row.try_get("user_agent").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                is_active: row.try_get("is_active").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new session
    pub async fn create(&self, request: &CreateSessionRequest) -> AuthResult<AuthSession> {
        let now = chrono::Utc::now().to_rfc3339();
        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(24))
            .to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO auth_sessions (token, user_id, device_info, ip_address, user_agent, is_active, created_at, updated_at, expires_at)
             VALUES (?, ?, ?, ?, ?, true, ?, ?, ?)"
        )
        .bind(&request.token)
        .bind(request.user_id)
        .bind(serde_json::to_string(&request.device_info).unwrap_or_default())
        .bind(&request.ip_address)
        .bind(&request.user_agent)
        .bind(&now)
        .bind(&now)
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let session_id = result.last_insert_rowid();
        self.find_by_id(session_id).await?.ok_or_else(|| {
            AuthError::DatabaseError("Failed to retrieve created session".to_string())
        })
    }

    /// Find session by token
    pub async fn find_by_token(&self, token: &str) -> AuthResult<Option<AuthSession>> {
        let row = sqlx::query(
            "SELECT id, token, user_id, device_info, ip_address, user_agent, is_active, created_at, updated_at, expires_at
             FROM auth_sessions WHERE token = ? AND is_active = true"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let device_info: Option<String> = row.try_get("device_info")
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

            let device_info_parsed = device_info
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(AuthSession {
                id: Some(row.try_get("id").map_err(|e| AuthError::DatabaseError(e.to_string()))?),
                token: row.try_get("token").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                device_info: device_info_parsed,
                ip_address: row.try_get("ip_address").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_agent: row.try_get("user_agent").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                is_active: row.try_get("is_active").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find session by user ID
    pub async fn find_by_user_id(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        let rows = sqlx::query(
            "SELECT id, token, user_id, device_info, ip_address, user_agent, is_active, created_at, updated_at, expires_at
             FROM auth_sessions WHERE user_id = ? AND is_active = true ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            let device_info: Option<String> = row.try_get("device_info")
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

            let device_info_parsed = device_info
                .and_then(|s| serde_json::from_str(&s).ok());

            sessions.push(AuthSession {
                id: Some(row.try_get("id").map_err(|e| AuthError::DatabaseError(e.to_string()))?),
                token: row.try_get("token").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                device_info: device_info_parsed,
                ip_address: row.try_get("ip_address").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                user_agent: row.try_get("user_agent").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                is_active: row.try_get("is_active").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| AuthError::DatabaseError(e.to_string()))?,
            });
        }

        Ok(sessions)
    }

    /// Update session last used timestamp
    pub async fn update_last_used(&self, session_id: &str) -> AuthResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE auth_sessions SET updated_at = ? WHERE token = ?"
        )
        .bind(&now)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete session by token
    pub async fn delete_by_token(&self, token: &str) -> AuthResult<()> {
        sqlx::query(
            "UPDATE auth_sessions SET is_active = false WHERE token = ?"
        )
        .bind(token)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Delete all sessions for a user
    pub async fn delete_by_user_id(&self, user_id: i64) -> AuthResult<u32> {
        let result = sqlx::query(
            "UPDATE auth_sessions SET is_active = false WHERE user_id = ? AND is_active = true"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Delete expired sessions
    pub async fn delete_expired(&self) -> AuthResult<u32> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE auth_sessions SET is_active = false WHERE expires_at < ? AND is_active = true"
        )
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Get active session count for user
    pub async fn count_active_sessions(&self, user_id: i64) -> AuthResult<i64> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM auth_sessions WHERE user_id = ? AND is_active = true AND expires_at > ?"
        )
        .bind(user_id)
        .bind(chrono::Utc::now().to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let count: i64 = count.try_get("count").map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        Ok(count)
    }

    /// Validate session exists and is not expired
    pub async fn validate_session(&self, token: &str) -> AuthResult<bool> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM auth_sessions WHERE token = ? AND is_active = true AND expires_at > ?"
        )
        .bind(token)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let count: i64 = result.try_get("count").map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;
    use std::path::Path;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
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

        (pool, temp_dir)
    }

    fn create_test_session_request() -> CreateSessionRequest {
        CreateSessionRequest {
            token: "test_token_12345".to_string(),
            user_id: 1,
            device_info: Some(serde_json::json!({
                "user_agent": "Mozilla/5.0",
                "platform": "linux"
            })),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        let session = repo.create(&request).await.unwrap();

        assert_eq!(session.user_id, request.user_id);
        assert_eq!(session.token, request.token);
        assert_eq!(session.ip_address, request.ip_address);
        assert_eq!(session.user_agent, request.user_agent);
        assert!(session.is_active);
    }

    #[tokio::test]
    async fn test_find_by_token() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        let created = repo.create(&request).await.unwrap();
        let found = repo.find_by_token(&request.token).await.unwrap();

        assert!(found.is_some());
        let found_session = found.unwrap();
        assert_eq!(found_session.id, created.id);
        assert_eq!(found_session.token, request.token);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        repo.create(&request).await.unwrap();
        let sessions = repo.find_by_user_id(request.user_id).await.unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, request.user_id);
    }

    #[tokio::test]
    async fn test_update_last_used() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        let session = repo.create(&request).await.unwrap();
        let original_updated_at = session.updated_at.clone();

        // Give a small delay to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        repo.update_last_used(&session.token).await.unwrap();

        let updated = repo.find_by_token(&session.token).await.unwrap().unwrap();
        assert_ne!(updated.updated_at, original_updated_at);
    }

    #[tokio::test]
    async fn test_delete_by_token() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        let session = repo.create(&request).await.unwrap();
        repo.delete_by_token(&session.token).await.unwrap();

        let found = repo.find_by_token(&session.token).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_user_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request1 = create_test_session_request();
        let mut request2 = create_test_session_request();
        request2.token = "test_token_67890".to_string();

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let deleted_count = repo.delete_by_user_id(request1.user_id).await.unwrap();
        assert_eq!(deleted_count, 2);

        let sessions = repo.find_by_user_id(request1.user_id).await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_count_active_sessions() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request1 = create_test_session_request();
        let mut request2 = create_test_session_request();
        request2.token = "test_token_67890".to_string();

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let count = repo.count_active_sessions(request1.user_id).await.unwrap();
        assert_eq!(count, 2);

        repo.delete_by_token(&request1.token).await.unwrap();
        let count = repo.count_active_sessions(request1.user_id).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_validate_session() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SessionRepository::new(pool);
        let request = create_test_session_request();

        let session = repo.create(&request).await.unwrap();

        let is_valid = repo.validate_session(&session.token).await.unwrap();
        assert!(is_valid);

        repo.delete_by_token(&session.token).await.unwrap();
        let is_valid = repo.validate_session(&session.token).await.unwrap();
        assert!(!is_valid);
    }
}