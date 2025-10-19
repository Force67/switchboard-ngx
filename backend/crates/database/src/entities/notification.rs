//! Notification entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<i64>,
    pub user_id: i64,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub priority: NotificationPriority,
    pub is_read: bool,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
    pub related_entity_id: Option<String>,
    pub related_entity_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub related_entity_id: Option<String>,
    pub related_entity_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    System,
    Message,
    ChatInvite,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
            NotificationType::Success => "success",
            NotificationType::System => "system",
            NotificationType::Message => "message",
            NotificationType::ChatInvite => "chat_invite",
        }
    }
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        match s {
            "warning" => NotificationType::Warning,
            "error" => NotificationType::Error,
            "success" => NotificationType::Success,
            "system" => NotificationType::System,
            "message" => NotificationType::Message,
            "chat_invite" => NotificationType::ChatInvite,
            _ => NotificationType::Info,
        }
    }
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl std::str::FromStr for NotificationType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NotificationType::from(s))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    Medium,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::Medium => "medium",
            NotificationPriority::High => "high",
            NotificationPriority::Urgent => "urgent",
        }
    }
}

impl From<&str> for NotificationPriority {
    fn from(s: &str) -> Self {
        match s {
            "normal" => NotificationPriority::Normal,
            "medium" => NotificationPriority::Medium,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Low,
        }
    }
}

impl ToString for NotificationPriority {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl std::str::FromStr for NotificationPriority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NotificationPriority::from(s))
    }
}