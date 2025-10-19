//! User repository for database operations.

use crate::entities::{User, CreateUserRequest, UpdateUserRequest};
use crate::types::{UserResult, UserError};
use sqlx::{SqlitePool, Row};
use chrono::Utc;

/// Repository for user database operations
#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: i64) -> UserResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active FROM users WHERE id = ? AND status != 'deleted'"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Find user by public ID
    pub async fn find_by_public_id(&self, public_id: &str) -> UserResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active FROM users WHERE public_id = ? AND status != 'deleted'"
        )
        .bind(public_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> UserResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active FROM users WHERE email = ? AND status != 'deleted'"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Create new user
    pub async fn create(&self, request: &CreateUserRequest) -> UserResult<User> {
        let now = Utc::now().to_rfc3339();
        let public_id = cuid2::cuid();
        let default_role = crate::entities::user::UserRole::User;

        let result = sqlx::query(
            "INSERT OR IGNORE INTO users (public_id, email, display_name, avatar_url, status, role, created_at, updated_at, email_verified, is_active) VALUES (?, ?, ?, ?, 'active', ?, ?, ?, false, true)"
        )
        .bind(public_id)
        .bind(&request.email)
        .bind(&request.display_name)
        .bind(&request.avatar_url)
        .bind(default_role.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                if e.to_string().contains("email") {
                    UserError::EmailAlreadyExists
                } else {
                    UserError::UserAlreadyExists
                }
            } else {
                UserError::DatabaseError(e.to_string())
            }
        })?;

        let user_id = result.last_insert_rowid();

        // Fetch the created user
        self.find_by_id(user_id).await?.ok_or_else(|| {
            UserError::DatabaseError("Failed to retrieve created user".to_string())
        })
    }

    /// Update user
    pub async fn update(&self, user_id: i64, request: &UpdateUserRequest) -> UserResult<User> {
        let now = Utc::now().to_rfc3339();

        // Build dynamic update query based on provided fields
        let mut query_parts = Vec::new();
        let mut values = Vec::new();

        if let Some(ref display_name) = request.display_name {
            query_parts.push("display_name = ?");
            values.push(display_name.clone());
        }

        if let Some(ref avatar_url) = request.avatar_url {
            query_parts.push("avatar_url = ?");
            values.push(avatar_url.clone());
        }

        if let Some(ref bio) = request.bio {
            query_parts.push("bio = ?");
            values.push(bio.clone());
        }

        if let Some(role) = &request.role {
            query_parts.push("role = ?");
            values.push(role.to_string());
        }

        if query_parts.is_empty() {
            return self.find_by_id(user_id).await?.ok_or(UserError::UserNotFound);
        }

        query_parts.push("updated_at = ?");
        values.push(now);

        let set_clause = query_parts.join(", ");
        let query_str = format!("UPDATE users SET {} WHERE id = ? AND status != 'deleted'", set_clause);

        let mut query = sqlx::query(&query_str);
        for value in values {
            query = query.bind(value);
        }
        query = query.bind(user_id);

        query
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") && e.to_string().contains("email") {
                    UserError::EmailAlreadyExists
                } else {
                    UserError::DatabaseError(e.to_string())
                }
            })?;

        self.find_by_id(user_id).await?.ok_or(UserError::UserNotFound)
    }

    /// Delete user (soft delete)
    pub async fn delete(&self, id: i64) -> UserResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE users SET status = 'deleted', updated_at = ? WHERE id = ? AND status != 'deleted'"
        )
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    /// Update user last login
    pub async fn update_last_login(&self, id: i64) -> UserResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ? AND status != 'deleted'"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    /// Search users by display name
    pub async fn search_by_display_name(&self, query: &str, limit: u32) -> UserResult<Vec<User>> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            r#"
            SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active
            FROM users
            WHERE display_name LIKE ? AND status = 'active' AND is_active = true
            ORDER BY display_name
            LIMIT ?
            "#
        )
        .bind(search_pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            users.push(User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            });
        }

        Ok(users)
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str) -> UserResult<bool> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE email = ? AND status != 'deleted'"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(count.unwrap_or(0) > 0)
    }

    /// Get user count
    pub async fn count(&self) -> UserResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE status != 'deleted'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }

    /// Get active users count
    pub async fn count_active(&self) -> UserResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE status = 'active' AND is_active = true"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }

    /// Update user active status
    pub async fn update_active_status(&self, user_id: i64, is_active: bool) -> UserResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE users SET is_active = ?, updated_at = ? WHERE id = ? AND status != 'deleted'"
        )
        .bind(is_active)
        .bind(&now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    /// Verify user email
    pub async fn verify_email(&self, user_id: i64) -> UserResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE users SET email_verified = true, updated_at = ? WHERE id = ? AND status != 'deleted'"
        )
        .bind(&now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    /// Update user password (for password change functionality)
    pub async fn update_password(&self, user_id: i64, password_hash: &str) -> UserResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ? AND status != 'deleted'"
        )
        .bind(password_hash)
        .bind(&now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

        Ok(())
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> UserResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active FROM users WHERE username = ? AND status != 'deleted'"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Get users by role
    pub async fn find_by_role(&self, role: crate::entities::user::UserRole, limit: u32) -> UserResult<Vec<User>> {
        let rows = sqlx::query(
            r#"
            SELECT id, public_id, email, username, display_name, avatar_url, bio, status, role, created_at, updated_at, last_login_at, email_verified, is_active
            FROM users
            WHERE role = ? AND status = 'active' AND is_active = true
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(role.to_string())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            users.push(User {
                id: row.get("id"),
                public_id: row.get("public_id"),
                email: row.get("email"),
                username: row.get("username"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                bio: row.get("bio"),
                status: crate::entities::user::UserStatus::from(row.get::<String, _>("status").as_str()),
                role: crate::entities::user::UserRole::from(row.get::<String, _>("role").as_str()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                email_verified: row.get("email_verified"),
                is_active: row.get("is_active"),
            });
        }

        Ok(users)
    }

    /// Batch update user status
    pub async fn batch_update_status(&self, user_ids: &[i64], status: crate::entities::user::UserStatus) -> UserResult<u32> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let now = Utc::now().to_rfc3339();
        let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query_str = format!(
            "UPDATE users SET status = ?, updated_at = ? WHERE id IN ({})",
            placeholders
        );

        let mut query = sqlx::query(&query_str);
        query = query.bind(status.to_string());
        query = query.bind(now);
        for &user_id in user_ids {
            query = query.bind(user_id);
        }

        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Get user statistics
    pub async fn get_user_stats(&self) -> UserResult<UserStats> {
        let total_count = self.count().await?;
        let active_count = self.count_active().await?;

        // Get counts by role
        let role_rows = sqlx::query(
            "SELECT role, COUNT(*) as count FROM users WHERE status != 'deleted' GROUP BY role"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let mut by_role = Vec::new();
        for row in role_rows {
            if let (Some(role), Some(count)) = (row.get::<Option<String>, _>("role"), row.get::<Option<i64>, _>("count")) {
                let user_role = crate::entities::user::UserRole::from(role.as_str());
                by_role.push((user_role, count));
            }
        }

        Ok(UserStats {
            total_count,
            active_count,
            by_role,
        })
    }
}

/// User statistics
#[derive(Debug, Clone)]
pub struct UserStats {
    pub total_count: i64,
    pub active_count: i64,
    pub by_role: Vec<(crate::entities::user::UserRole, i64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use sqlx::SqlitePool;

    async fn create_test_pool() -> SqlitePool {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema (simplified version of the actual schema)
        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                email TEXT UNIQUE,
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

        pool
    }

    #[tokio::test]
    async fn test_user_creation_and_retrieval() {
        let pool = create_test_pool().await;
        let repo = UserRepository::new(pool);

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password123".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            bio: None,
        };

        let created_user = repo.create(&request).await.unwrap();
        assert_eq!(created_user.email, Some(request.email.clone()));
        assert_eq!(created_user.display_name, Some(request.display_name));

        let found_user = repo.find_by_id(created_user.id).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().email, Some(request.email));
    }

    #[tokio::test]
    async fn test_user_search() {
        let pool = create_test_pool().await;
        let repo = UserRepository::new(pool);

        // Create test users
        let request1 = CreateUserRequest {
            email: "user1@example.com".to_string(),
            username: "user1".to_string(),
            display_name: "Alice Smith".to_string(),
            password: "password".to_string(),
            avatar_url: None,
            bio: None,
        };

        let request2 = CreateUserRequest {
            email: "user2@example.com".to_string(),
            username: "user2".to_string(),
            display_name: "Bob Johnson".to_string(),
            password: "password".to_string(),
            avatar_url: None,
            bio: None,
        };

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        // Search for users
        let results = repo.search_by_display_name("Alice", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, Some("Alice Smith".to_string()));
    }

    #[tokio::test]
    async fn test_email_exists() {
        let pool = create_test_pool().await;
        let repo = UserRepository::new(pool);

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password".to_string(),
            avatar_url: None,
            bio: None,
        };

        assert!(!repo.email_exists("test@example.com").await.unwrap());

        repo.create(&request).await.unwrap();

        assert!(repo.email_exists("test@example.com").await.unwrap());
        assert!(!repo.email_exists("nonexistent@example.com").await.unwrap());
    }

    #[tokio::test]
    async fn test_user_stats() {
        let pool = create_test_pool().await;
        let repo = UserRepository::new(pool);

        // Create some test users
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password".to_string(),
            avatar_url: None,
            bio: None,
        };

        repo.create(&request).await.unwrap();
        repo.create(&request).await.unwrap();

        let stats = repo.get_user_stats().await.unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.active_count, 2);
    }

    #[tokio::test]
    async fn test_update_last_login() {
        let pool = create_test_pool().await;
        let repo = UserRepository::new(pool);

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password".to_string(),
            avatar_url: None,
            bio: None,
        };

        let user = repo.create(&request).await.unwrap();

        // Initially no last login
        assert!(user.last_login_at.is_none());

        // Update last login
        repo.update_last_login(user.id).await.unwrap();

        // Check that last login was updated
        let updated_user = repo.find_by_id(user.id).await.unwrap().unwrap();
        assert!(updated_user.last_login_at.is_some());
    }
}