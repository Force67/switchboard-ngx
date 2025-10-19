//! Response types for the user management system.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use switchboard_database::{User, AuthSession, Notification, UserSettings, UserRole, UserStatus};

/// Response containing user information (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub public_id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub is_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response containing authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub session_id: String,
}

/// Response containing session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: i64,
    pub session_id: String,
    pub user_id: i64,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Response containing notification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub data: Option<serde_json::Value>,
    pub action_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

/// Response containing user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub id: i64,
    pub user_id: i64,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub language: String,
    pub timezone: String,
    pub theme: String,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub privacy_show_email: bool,
    pub privacy_show_status: bool,
    pub message_content_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response containing search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUsersResponse {
    pub users: Vec<UserResponse>,
    pub total_count: u64,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Response containing notifications list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total_count: u64,
    pub unread_count: u64,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Response containing notification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatsResponse {
    pub total_count: u64,
    pub unread_count: u64,
    pub read_count: u64,
    pub by_type: Vec<(String, u64)>,
    pub by_priority: Vec<(String, u64)>,
}

/// Response containing active sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionResponse>,
    pub current_session_id: String,
    pub total_count: u64,
}

/// Response for two-factor authentication setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorSetupResponse {
    pub qr_code: String,
    pub secret: String,
    pub backup_codes: Vec<String>,
}

/// Response for password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetResponse {
    pub message: String,
    pub reset_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response for account deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountResponse {
    pub message: String,
    pub deletion_date: DateTime<Utc>,
    pub can_be_cancelled_until: DateTime<Utc>,
}

/// Response for data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportResponse {
    pub message: String,
    pub export_id: String,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub download_url: Option<String>,
}

/// Response for OAuth login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub authorization_url: String,
    pub state: String,
    pub provider: String,
}

/// Generic success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Generic error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        // Convert string timestamps to DateTime<Utc>
        let created_at = DateTime::parse_from_rfc3339(&user.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&user.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let last_login_at = user.last_login_at.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Self {
            id: user.id,
            public_id: user.public_id,
            username: user.username.unwrap_or_else(|| "".to_string()),
            display_name: user.display_name.unwrap_or_else(|| "Unknown User".to_string()),
            email: user.email.unwrap_or_else(|| "".to_string()),
            avatar_url: user.avatar_url,
            bio: user.bio,
            role: user.role,
            status: user.status,
            is_verified: user.email_verified, // Map from email_verified
            last_login_at,
            created_at,
            updated_at,
        }
    }
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        // Convert string timestamps to DateTime<Utc>
        let created_at = DateTime::parse_from_rfc3339(&notification.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let expires_at = notification.expires_at.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Self {
            id: notification.id.expect("Notification should have an ID"),
            user_id: notification.user_id,
            title: notification.title,
            message: notification.message,
            notification_type: notification.notification_type.to_string(),
            priority: notification.priority.to_string(),
            is_read: notification.is_read,
            data: notification.metadata,
            action_url: None, // Not in Notification structure
            expires_at,
            created_at,
            read_at: None, // Not available in Notification structure
        }
    }
}

impl From<UserSettings> for SettingsResponse {
    fn from(settings: UserSettings) -> Self {
        // Convert string timestamps to DateTime<Utc>
        let created_at = DateTime::parse_from_rfc3339(&settings.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&settings.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Self {
            id: settings.id,
            user_id: settings.user_id,
            display_name: None, // Not in UserSettings structure
            bio: None, // Not in UserSettings structure
            avatar_url: None, // Not in UserSettings structure
            language: settings.preferences.language,
            timezone: settings.preferences.timezone,
            theme: settings.preferences.theme.clone(),
            email_notifications: settings.preferences.email_notifications,
            push_notifications: settings.preferences.notifications_enabled, // Use notifications_enabled for push
            privacy_show_email: false, // Default value - not in UserSettings structure
            privacy_show_status: false, // Default value - not in UserSettings structure
            message_content_type: crate::types::requests::MessageContentType::Text.to_string(), // Default value
            created_at,
            updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use switchboard_database::{NotificationType, NotificationPriority};

    #[test]
    fn test_convert_user_to_response() {
        let user = User {
            id: 1,
            public_id: "abc123".to_string(),
            username: Some("testuser".to_string()),
            display_name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            bio: Some("Test bio".to_string()),
            role: UserRole::User,
            status: UserStatus::Active,
            is_active: true,
            email_verified: true,
            last_login_at: Some(Utc::now().to_rfc3339()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let response: UserResponse = user.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, UserRole::User);
    }

    #[test]
    fn test_serialize_deserialize_responses() {
        let response = UserResponse {
            id: 1,
            public_id: "abc123".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            avatar_url: None,
            bio: None,
            role: UserRole::User,
            status: UserStatus::Active,
            is_verified: true,
            last_login_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: UserResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.username, deserialized.username);
    }
}