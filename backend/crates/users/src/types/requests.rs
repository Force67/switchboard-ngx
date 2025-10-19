//! Request types for the user management system.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::entities::{AuthProvider};
use crate::entities::user::UserRole;
use crate::entities::notification::{NotificationType, NotificationPriority};

/// Message content type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageContentType {
    Text,
    Markdown,
    Code,
    Image,
    File,
    Audio,
    Video,
}

impl Default for MessageContentType {
    fn default() -> Self {
        MessageContentType::Text
    }
}

impl From<&str> for MessageContentType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "markdown" => MessageContentType::Markdown,
            "code" => MessageContentType::Code,
            "image" => MessageContentType::Image,
            "file" => MessageContentType::File,
            "audio" => MessageContentType::Audio,
            "video" => MessageContentType::Video,
            _ => MessageContentType::Text,
        }
    }
}

impl From<MessageContentType> for String {
    fn from(content_type: MessageContentType) -> Self {
        match content_type {
            MessageContentType::Text => "text".to_string(),
            MessageContentType::Markdown => "markdown".to_string(),
            MessageContentType::Code => "code".to_string(),
            MessageContentType::Image => "image".to_string(),
            MessageContentType::File => "file".to_string(),
            MessageContentType::Audio => "audio".to_string(),
            MessageContentType::Video => "video".to_string(),
        }
    }
}

impl std::fmt::Display for MessageContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

/// Request to create a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: i64,
    pub notification_type: crate::entities::notification::NotificationType,
    pub title: String,
    pub message: String,
    pub priority: crate::entities::notification::NotificationPriority,
    pub related_entity_id: Option<String>,
    pub related_entity_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

/// Request to create a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// Request to update a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<UserRole>,
}

/// Request to change password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Request to reset password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_token: String,
    pub new_password: String,
}

/// Request to initiate password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub device_info: Option<DeviceInfo>,
}

/// OAuth login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthLoginRequest {
    pub provider: AuthProvider,
    pub access_token: String,
    pub device_info: Option<DeviceInfo>,
}

/// Registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub invite_code: Option<String>,
    pub device_info: Option<DeviceInfo>,
}

/// Device information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: Option<String>,
    pub platform: Option<String>,
}

/// Request to refresh session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSessionRequest {
    pub refresh_token: String,
}

/// Request to logout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
    pub logout_all_devices: Option<bool>,
}

/// Request to enable two-factor authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableTwoFactorRequest {
    pub password: String,
}

/// Request to verify two-factor authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyTwoFactorRequest {
    pub code: String,
    pub backup_code: Option<bool>,
}

/// Request to update notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub email_notifications: Option<bool>,
    pub push_notifications: Option<bool>,
    pub notification_types: Option<Vec<NotificationType>>,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub weekly_digest: Option<bool>,
}

/// Request to mark notifications as read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkNotificationsReadRequest {
    pub notification_ids: Option<Vec<i64>>,
    pub mark_all: Option<bool>,
}

/// Request to search users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUsersRequest {
    pub query: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub include_inactive: Option<bool>,
}

/// Request to upload avatar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadAvatarRequest {
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub file_data: Vec<u8>,
}

/// Request to update user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub theme: Option<String>,
    pub email_notifications: Option<bool>,
    pub push_notifications: Option<bool>,
    pub privacy_show_email: Option<bool>,
    pub privacy_show_status: Option<bool>,
    pub message_content_type: Option<MessageContentType>,
}

/// Request to export user data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserDataRequest {
    pub include_messages: Option<bool>,
    pub include_settings: Option<bool>,
    pub format: Option<String>,
}

/// Request to delete user account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
    pub confirmation: String,
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize_requests() {
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            remember_me: Some(true),
            device_info: Some(DeviceInfo {
                user_agent: Some("Mozilla/5.0".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                device_type: Some("desktop".to_string()),
                platform: Some("Linux".to_string()),
            }),
        };

        let json = serde_json::to_string(&login_request).unwrap();
        let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(login_request.email, deserialized.email);
        assert_eq!(login_request.password, deserialized.password);
        assert_eq!(login_request.remember_me, deserialized.remember_me);
    }
}