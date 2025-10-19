//! Settings repository for database operations.

use crate::entities::UserSettings;
use crate::types::{UserResult, UpdateSettingsRequest};
use crate::types::errors::UserError;
use sqlx::{SqlitePool, Row};

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
            "SELECT id, user_id, preferences, created_at, updated_at
             FROM user_settings WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let preferences_str: String = row.try_get("preferences")
                .map_err(|e| UserError::DatabaseError(e.to_string()))?;

            let preferences = serde_json::from_str(&preferences_str)
                .map_err(|e| UserError::DatabaseError(e.to_string()))?;

            Ok(Some(UserSettings {
                id: row.try_get("id").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                preferences,
                created_at: row.try_get("created_at").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| UserError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create new settings
    pub async fn create(&self, user_id: i64) -> UserResult<UserSettings> {
        let settings = UserSettings::new(0, user_id);
        let preferences_json = serde_json::to_string(&settings.preferences)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let result = sqlx::query(
            "INSERT INTO user_settings (user_id, preferences, created_at, updated_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(preferences_json)
        .bind(&settings.created_at)
        .bind(&settings.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let settings_id = result.last_insert_rowid();
        let mut created_settings = settings;
        created_settings.id = settings_id;
        Ok(created_settings)
    }

    /// Update settings
    pub async fn update(&self, user_id: i64, request: &UpdateSettingsRequest) -> UserResult<UserSettings> {
        // Update with new preferences
        let preferences_json = serde_json::to_string(&request.preferences)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE user_settings SET preferences = ?, updated_at = ? WHERE user_id = ?"
        )
        .bind(preferences_json)
        .bind(&now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::UserNotFound);
        }

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
        UserSettings::new(0, 0) // user_id doesn't matter for defaults
    }

    /// Reset settings to default
    pub async fn reset_to_default(&self, user_id: i64) -> UserResult<UserSettings> {
        let default_settings = Self::get_default_settings();
        let update_request = UpdateSettingsRequest {
            preferences: default_settings.preferences,
        };
        self.update(user_id, &update_request).await
    }

    /// Export settings
    pub async fn export_settings(&self, user_id: i64) -> UserResult<String> {
        let settings = self.find_by_user_id(user_id).await?.ok_or_else(|| {
            UserError::UserNotFound
        })?;

        let export_data = serde_json::json!({
            "preferences": settings.preferences,
            "exported_at": chrono::Utc::now().to_rfc3339()
        });

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| UserError::DatabaseError(e.to_string()))
    }

    /// Import settings
    pub async fn import_settings(&self, user_id: i64, settings_json: &str) -> UserResult<UserSettings> {
        let import_data: serde_json::Value = serde_json::from_str(settings_json)
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        let preferences: crate::entities::UserPreferences = if let Some(pref) = import_data.get("preferences") {
            serde_json::from_value(pref.clone())
                .map_err(|e| UserError::DatabaseError(e.to_string()))?
        } else {
            // Try to extract individual fields if preferences wrapper is not present
            crate::entities::UserPreferences {
                theme: import_data.get("theme").and_then(|v| v.as_str()).unwrap_or("dark").to_string(),
                language: import_data.get("language").and_then(|v| v.as_str()).unwrap_or("en").to_string(),
                notifications_enabled: import_data.get("notifications_enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                email_notifications: import_data.get("email_notifications").and_then(|v| v.as_bool()).unwrap_or(true),
                timezone: import_data.get("timezone").and_then(|v| v.as_str()).unwrap_or("UTC").to_string(),
            }
        };

        let update_request = UpdateSettingsRequest { preferences };

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
                preferences TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        let settings = repo.create(1).await.unwrap();

        assert_eq!(settings.user_id, 1);
        assert!(settings.id > 0);
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

        repo.create(1).await.unwrap();

        let request = UpdateSettingsRequest {
            preferences: crate::entities::UserPreferences {
                theme: "light".to_string(),
                language: "fr".to_string(),
                notifications_enabled: false,
                email_notifications: false,
                timezone: "EST".to_string(),
            },
        };

        let updated = repo.update(1, &request).await.unwrap();
        assert_eq!(updated.preferences.theme, "light");
        assert_eq!(updated.preferences.language, "fr");
        assert_eq!(updated.preferences.notifications_enabled, false);
    }

    #[tokio::test]
    async fn test_get_or_create() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        // Should create if not found
        let settings1 = repo.get_or_create(1).await.unwrap();
        assert!(settings1.id > 0);

        // Should return existing if found
        let settings2 = repo.get_or_create(1).await.unwrap();
        assert_eq!(settings1.id, settings2.id);
    }

    #[tokio::test]
    async fn test_export_import_settings() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = SettingsRepository::new(pool);

        repo.create(1).await.unwrap();

        let exported = repo.export_settings(1).await.unwrap();
        let imported = repo.import_settings(2, &exported).await.unwrap();

        assert_eq!(imported.user_id, 2);
        assert_eq!(imported.preferences.theme, "dark"); // default theme
    }
}