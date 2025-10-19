//! Settings entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,
    pub preferences: UserPreferences,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub email_notifications: bool,
    pub timezone: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications_enabled: true,
            email_notifications: true,
            timezone: "UTC".to_string(),
        }
    }
}

impl UserSettings {
    pub fn new(id: i64, user_id: i64) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            user_id,
            preferences: UserPreferences::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}