//! Settings repository for database operations.

use crate::entities::UserSettings;
use crate::types::{UserResult, UpdateSettingsRequest};
use crate::types::errors::UserError;
use sqlx::SqlitePool;

/// Repository for user settings database operations
pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    /// Create a new settings repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find settings by user ID
    pub async fn find_by_user_id(&self, user_id: i64) -> UserResult<Option<UserSettings>> {
        let row = sqlx::query(
            "SELECT id, user_id, display_name, bio, avatar_url, language, timezone, theme,
                    email_notifications, push_notifications, privacy_show_email, privacy_show_status,
                    message_content_type, created_at, updated_at
             FROM user_settings WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let message_content_type_str: String = row.try_get("message_content_type")
                .map_err(|e| UserError::DatabaseError(e.to_string()))?;

            Ok(Some(UserSettings {
                id: Some(row.try_get("id").map_err(|e| UserError::DatabaseError(e.to_string()))?),
                user_id: row.try_get("user_id").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                display_name: row.try_get("display_name").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                bio: row.try_get("bio").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                avatar_url: row.try_get("avatar_url").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                language: row.try_get("language").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                timezone: row.try_get("timezone").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                theme: row.try_get("theme").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                email_notifications: row.try_get("email_notifications").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                push_notifications: row.try_get("push_notifications").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                privacy_show_email: row.try_get("privacy_show_email").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                privacy_show_status: row.try_get("privacy_show_status").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                message_content_type: message_content_type_str.parse().map_err(|_| UserError::InvalidSettings)?,
                created_at: row.try_get("created_at").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| UserError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create new settings
    pub async fn create(&self, user_id: i64) -> UserResult<UserSettings> {
        let now = chrono::Utc::now().to_rfc3339();
        let default_settings = Self::get_default_settings();

        let result = sqlx::query(
            "INSERT INTO user_settings (user_id, display_name, bio, avatar_url, language, timezone, theme,
                    email_notifications, push_notifications, privacy_show_email, privacy_show_status,
                    message_content_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(&default_settings.display_name)
        .bind(&default_settings.bio)
        .bind(&default_settings.avatar_url)
        .bind(&default_settings.language)
        .bind(&default_settings.timezone)
        .bind(&default_settings.theme)
        .bind(default_settings.email_notifications)
        .bind(default_settings.push_notifications)
        .bind(default_settings.privacy_show_email)
        .bind(default_settings.privacy_show_status)
        .bind(default_settings.message_content_type.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let settings_id = result.last_insert_rowid();
        self.find_by_user_id(user_id).await?.ok_or_else(|| {
            UserError::DatabaseError("Failed to retrieve created settings".to_string())
        })
    }

    /// Update settings
    pub async fn update(&self, user_id: i64, request: &UpdateSettingsRequest) -> UserResult<UserSettings> {
        let mut set_clauses = Vec::new();
        let mut bind_values: Vec<Box<dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Send + 'static>> = Vec::new();

        if let Some(ref display_name) = request.display_name {
            set_clauses.push("display_name = ?");
            bind_values.push(Box::new(display_name.clone()));
        }
        if let Some(ref bio) = request.bio {
            set_clauses.push("bio = ?");
            bind_values.push(Box::new(bio.clone()));
        }
        if let Some(ref avatar_url) = request.avatar_url {
            set_clauses.push("avatar_url = ?");
            bind_values.push(Box::new(avatar_url.clone()));
        }
        if let Some(ref language) = request.language {
            set_clauses.push("language = ?");
            bind_values.push(Box::new(language.clone()));
        }
        if let Some(ref timezone) = request.timezone {
            set_clauses.push("timezone = ?");
            bind_values.push(Box::new(timezone.clone()));
        }
        if let Some(ref theme) = request.theme {
            set_clauses.push("theme = ?");
            bind_values.push(Box::new(theme.clone()));
        }
        if let Some(email_notifications) = request.email_notifications {
            set_clauses.push("email_notifications = ?");
            bind_values.push(Box::new(email_notifications));
        }
        if let Some(push_notifications) = request.push_notifications {
            set_clauses.push("push_notifications = ?");
            bind_values.push(Box::new(push_notifications));
        }
        if let Some(privacy_show_email) = request.privacy_show_email {
            set_clauses.push("privacy_show_email = ?");
            bind_values.push(Box::new(privacy_show_email));
        }
        if let Some(privacy_show_status) = request.privacy_show_status {
            set_clauses.push("privacy_show_status = ?");
            bind_values.push(Box::new(privacy_show_status));
        }
        if let Some(ref message_content_type) = request.message_content_type {
            set_clauses.push("message_content_type = ?");
            bind_values.push(Box::new(message_content_type.to_string()));
        }

        if set_clauses.is_empty() {
            return self.find_by_user_id(user_id).await?.ok_or_else(|| {
                UserError::UserNotFound
            });
        }

        set_clauses.push("updated_at = ?");
        let now = chrono::Utc::now().to_rfc3339();
        bind_values.push(Box::new(now));

        let sql = format!(
            "UPDATE user_settings SET {} WHERE user_id = ?",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for value in bind_values {
            query = query.bind(value);
        }
        query = query.bind(user_id);

        query
            .execute(&self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        self.find_by_user_id(user_id).await?.ok_or_else(|| {
            UserError::UserNotFound
        })
    }

    /// Delete settings
    pub async fn delete(&self, user_id: i64) -> UserResult<()> {
        sqlx::query(
            "DELETE FROM user_settings WHERE user_id = ?"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get default settings
    pub fn get_default_settings() -> UserSettings {
        UserSettings::new(0) // user_id doesn't matter for defaults
    }

    /// Reset settings to default
    pub async fn reset_to_default(&self, user_id: i64) -> UserResult<UserSettings> {
        let default_settings = Self::get_default_settings();

        sqlx::query(
            "UPDATE user_settings SET display_name = ?, bio = ?, avatar_url = ?, language = ?, timezone = ?, theme = ?,
                    email_notifications = ?, push_notifications = ?, privacy_show_email = ?, privacy_show_status = ?,
                    message_content_type = ?, updated_at = ? WHERE user_id = ?"
        )
        .bind(&default_settings.display_name)
        .bind(&default_settings.bio)
        .bind(&default_settings.avatar_url)
        .bind(&default_settings.language)
        .bind(&default_settings.timezone)
        .bind(&default_settings.theme)
        .bind(default_settings.email_notifications)
        .bind(default_settings.push_notifications)
        .bind(default_settings.privacy_show_email)
        .bind(default_settings.privacy_show_status)
        .bind(default_settings.message_content_type.to_string())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        self.find_by_user_id(user_id).await?.ok_or_else(|| {
            UserError::UserNotFound
        })
    }

    /// Export settings
    pub async fn export_settings(&self, user_id: i64) -> UserResult<String> {
        let settings = self.find_by_user_id(user_id).await?.ok_or_else(|| {
            UserError::UserNotFound
        })?;

        let export_data = serde_json::json!({
            "display_name": settings.display_name,
            "bio": settings.bio,
            "avatar_url": settings.avatar_url,
            "language": settings.language,
            "timezone": settings.timezone,
            "theme": settings.theme,
            "email_notifications": settings.email_notifications,
            "push_notifications": settings.push_notifications,
            "privacy_show_email": settings.privacy_show_email,
            "privacy_show_status": settings.privacy_show_status,
            "message_content_type": settings.message_content_type.to_string(),
            "exported_at": chrono::Utc::now().to_rfc3339()
        });

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| UserError::SerializationError(e.to_string()))
    }

    /// Import settings
    pub async fn import_settings(&self, user_id: i64, settings_json: &str) -> UserResult<UserSettings> {
        let import_data: serde_json::Value = serde_json::from_str(settings_json)
            .map_err(|e| UserError::SerializationError(e.to_string()))?;

        let mut update_request = UpdateSettingsRequest {
            display_name: import_data.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            bio: import_data.get("bio").and_then(|v| v.as_str()).map(|s| s.to_string()),
            avatar_url: import_data.get("avatar_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            language: import_data.get("language").and_then(|v| v.as_str()).map(|s| s.to_string()),
            timezone: import_data.get("timezone").and_then(|v| v.as_str()).map(|s| s.to_string()),
            theme: import_data.get("theme").and_then(|v| v.as_str()).map(|s| s.to_string()),
            email_notifications: import_data.get("email_notifications").and_then(|v| v.as_bool()),
            push_notifications: import_data.get("push_notifications").and_then(|v| v.as_bool()),
            privacy_show_email: import_data.get("privacy_show_email").and_then(|v| v.as_bool()),
            privacy_show_status: import_data.get("privacy_show_status").and_then(|v| v.as_bool()),
            message_content_type: import_data.get("message_content_type")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
        };

        // If settings don't exist for user, create them first
        if self.find_by_user_id(user_id).await?.is_none() {
            self.create(user_id).await?;
        }

        self.update(user_id, &update_request).await
    }

    /// Get or create settings (creates if not found)
    pub async fn get_or_create(&self, user_id: i64) -> UserResult<UserSettings> {
        if let Some(settings) = self.find_by_user_id(user_id).await? {
            Ok(settings)
        } else {
            self.create(user_id).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;
    use std::path::Path;
    use crate::types::requests::MessageContentType;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_settings.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE user_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL UNIQUE,
                display_name TEXT,
                bio TEXT,
                avatar_url TEXT,
                language TEXT,
                timezone TEXT,
                theme TEXT,
                email_notifications BOOLEAN NOT NULL DEFAULT true,
                push_notifications BOOLEAN NOT NULL DEFAULT true,
                privacy_show_email BOOLEAN NOT NULL DEFAULT false,
                privacy_show_status BOOLEAN NOT NULL DEFAULT true,
                message_content_type TEXT NOT NULL DEFAULT 'text',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    fn create_test_update_request() -> UpdateSettingsRequest {
        UpdateSettingsRequest {
            display_name: Some("Updated Name".to_string()),
            bio: Some("Updated bio".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            language: Some("en".to_string()),
            timezone: Some("UTC".to_string()),
            theme: Some("dark".to_string()),
            email_notifications: Some(false),
            push_notifications: Some(false),
            privacy_show_email: Some(true),
            privacy_show_status: Some(false),
            message_content_type: Some(MessageContentType::Markdown),
        }
    }

    #[tokio::test]
    async fn test_create_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        let settings = repo.create(1).await.unwrap();

        assert_eq!(settings.user_id, 1);
        assert!(settings.id.is_some());
        assert!(!settings.display_name.is_empty());
        assert_eq!(settings.email_notifications, true);
        assert_eq!(settings.push_notifications, true);
        assert_eq!(settings.message_content_type, MessageContentType::Text);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        let created = repo.create(1).await.unwrap();
        let found = repo.find_by_user_id(1).await.unwrap();

        assert!(found.is_some());
        let found_settings = found.unwrap();
        assert_eq!(found_settings.id, created.id);
        assert_eq!(found_settings.user_id, 1);
    }

    #[tokio::test]
    async fn test_update_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);
        let request = create_test_update_request();

        repo.create(1).await.unwrap();
        let updated = repo.update(1, &request).await.unwrap();

        assert_eq!(updated.display_name, request.display_name.unwrap());
        assert_eq!(updated.bio, request.bio.unwrap());
        assert_eq!(updated.avatar_url, request.avatar_url.unwrap());
        assert_eq!(updated.language, request.language.unwrap());
        assert_eq!(updated.timezone, request.timezone.unwrap());
        assert_eq!(updated.theme, request.theme.unwrap());
        assert_eq!(updated.email_notifications, request.email_notifications.unwrap());
        assert_eq!(updated.push_notifications, request.push_notifications.unwrap());
        assert_eq!(updated.privacy_show_email, request.privacy_show_email.unwrap());
        assert_eq!(updated.privacy_show_status, request.privacy_show_status.unwrap());
        assert_eq!(updated.message_content_type, request.message_content_type.unwrap());
    }

    #[tokio::test]
    async fn test_update_partial_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        repo.create(1).await.unwrap();

        let mut request = UpdateSettingsRequest {
            display_name: Some("New Name".to_string()),
            ..Default::default()
        };

        let updated = repo.update(1, &request).await.unwrap();
        assert_eq!(updated.display_name, "New Name");

        // Only display_name should change, others should remain default
        assert_eq!(updated.email_notifications, true);
        assert_eq!(updated.push_notifications, true);
    }

    #[tokio::test]
    async fn test_delete_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        repo.create(1).await.unwrap();
        repo.delete(1).await.unwrap();

        let found = repo.find_by_user_id(1).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_reset_to_default() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);
        let request = create_test_update_request();

        repo.create(1).await.unwrap();
        repo.update(1, &request).await.unwrap();

        let reset_settings = repo.reset_to_default(1).await.unwrap();
        let default_settings = SettingsRepository::get_default_settings();

        assert_eq!(reset_settings.display_name, default_settings.display_name);
        assert_eq!(reset_settings.email_notifications, default_settings.email_notifications);
        assert_eq!(reset_settings.message_content_type, default_settings.message_content_type);
    }

    #[tokio::test]
    async fn test_export_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);
        let request = create_test_update_request();

        repo.create(1).await.unwrap();
        repo.update(1, &request).await.unwrap();

        let exported = repo.export_settings(1).await.unwrap();
        let exported_json: serde_json::Value = serde_json::from_str(&exported).unwrap();

        assert_eq!(exported_json["display_name"], request.display_name.unwrap());
        assert_eq!(exported_json["bio"], request.bio.unwrap());
        assert_eq!(exported_json["language"], request.language.unwrap());
        assert!(exported_json["exported_at"].is_string());
    }

    #[tokio::test]
    async fn test_import_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        repo.create(1).await.unwrap();

        let import_json = serde_json::json!({
            "display_name": "Imported Name",
            "bio": "Imported bio",
            "language": "fr",
            "theme": "light",
            "email_notifications": false,
            "message_content_type": "markdown"
        });

        let import_str = serde_json::to_string(&import_json).unwrap();
        let imported = repo.import_settings(1, &import_str).await.unwrap();

        assert_eq!(imported.display_name, "Imported Name");
        assert_eq!(imported.bio, "Imported bio");
        assert_eq!(imported.language, "fr");
        assert_eq!(imported.theme, "light");
        assert_eq!(imported.email_notifications, false);
        assert_eq!(imported.message_content_type, MessageContentType::Markdown);
    }

    #[tokio::test]
    async fn test_get_or_create() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        // Should create if not found
        let settings1 = repo.get_or_create(1).await.unwrap();
        assert!(settings1.id.is_some());

        // Should return existing if found
        let settings2 = repo.get_or_create(1).await.unwrap();
        assert_eq!(settings1.id, settings2.id);
    }

    #[tokio::test]
    async fn test_import_creates_if_not_exists() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        let import_json = serde_json::json!({
            "display_name": "New User",
            "email_notifications": false
        });

        let import_str = serde_json::to_string(&import_json).unwrap();
        let imported = repo.import_settings(1, &import_str).await.unwrap();

        assert_eq!(imported.user_id, 1);
        assert_eq!(imported.display_name, "New User");
        assert_eq!(imported.email_notifications, false);
    }

    #[tokio::test]
    async fn test_invalid_message_content_type() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        let import_json = serde_json::json!({
            "display_name": "Test",
            "message_content_type": "invalid_type"
        });

        let import_str = serde_json::to_string(&import_json).unwrap();
        let imported = repo.import_settings(1, &import_str).await.unwrap();

        // Should default to Text if invalid type provided
        assert_eq!(imported.message_content_type, MessageContentType::Text);
    }

    impl Default for UpdateSettingsRequest {
        fn default() -> Self {
            Self {
                display_name: None,
                bio: None,
                avatar_url: None,
                language: None,
                timezone: None,
                theme: None,
                email_notifications: None,
                push_notifications: None,
                privacy_show_email: None,
                privacy_show_status: None,
                message_content_type: None,
            }
        }
    }
}