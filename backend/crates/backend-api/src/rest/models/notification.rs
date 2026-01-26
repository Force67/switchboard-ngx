//! Notification-related request and response types

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

/// Notification entity
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    /// Notification type
    pub r#type: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub created_at: String,
}

/// Query parameters for listing notifications
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response containing the count of unread notifications
#[derive(Debug, Serialize, ToSchema)]
pub struct UnreadCountResponse {
    pub unread_count: i64,
}

/// Response after bulk update operations
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkUpdateResponse {
    pub updated_count: u64,
}

/// Response containing multiple notifications
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationsResponse {
    pub notifications: Vec<Notification>,
}

/// Response containing a single notification
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub notification: Notification,
}

/// Request body for marking a notification as read/unread
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkNotificationReadRequest {
    pub read: bool,
}

/// Convert database notification to API response model
pub fn to_api_notification(n: switchboard_database::Notification) -> Notification {
    Notification {
        id: n.id.unwrap_or(0),
        user_id: n.user_id,
        r#type: n.notification_type.to_string(),
        title: n.title,
        body: n.message,
        read: n.is_read,
        created_at: n.created_at,
    }
}
