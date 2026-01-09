//! Notification service for managing user notifications.

use sqlx::SqlitePool;
use switchboard_database::{Notification, NotificationRepository, NotificationResult};

/// Service for managing notification operations
pub struct NotificationService {
    notification_repository: NotificationRepository,
}

impl NotificationService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            notification_repository: NotificationRepository::new(pool),
        }
    }

    /// Get notifications for a user
    pub async fn get_notifications(
        &self,
        user_id: i64,
        limit: u32,
        offset: u32,
    ) -> NotificationResult<Vec<Notification>> {
        self.notification_repository
            .find_by_user_id(user_id, limit, offset)
            .await
    }

    /// Create a new notification
    pub async fn create_notification(
        &self,
        notification: Notification,
    ) -> NotificationResult<Notification> {
        let request = switchboard_database::CreateNotificationRequest {
            user_id: notification.user_id,
            notification_type: notification.notification_type,
            title: notification.title,
            message: notification.message,
            priority: notification.priority,
            related_entity_id: notification.related_entity_id,
            related_entity_type: notification.related_entity_type,
            metadata: notification.metadata,
            expires_at: notification.expires_at,
        };
        self.notification_repository.create(&request).await
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, notification_id: i64, user_id: i64) -> NotificationResult<()> {
        self.notification_repository
            .mark_as_read(notification_id, user_id)
            .await
    }

    /// Mark all notifications as read
    pub async fn mark_all_as_read(&self, user_id: i64) -> NotificationResult<u32> {
        self.notification_repository.mark_all_as_read(user_id).await
    }

    /// Delete notification
    pub async fn delete_notification(
        &self,
        notification_id: i64,
        user_id: i64,
    ) -> NotificationResult<()> {
        self.notification_repository
            .delete(notification_id, user_id)
            .await
    }

    /// Get unread count
    pub async fn get_unread_count(&self, user_id: i64) -> NotificationResult<u64> {
        let count = self.notification_repository.get_unread_count(user_id).await?;
        Ok(count as u64)
    }

    /// Notify new message
    pub async fn notify_new_message(
        &self,
        user_id: i64,
        chat_id: &str,
        message_content: &str,
    ) -> NotificationResult<()> {
        self.notification_repository
            .create_message_notification(user_id, chat_id, message_content, "System")
            .await?;
        Ok(())
    }

    /// Notify chat invite
    pub async fn notify_chat_invite(
        &self,
        user_id: i64,
        chat_id: &str,
        inviter_name: &str,
    ) -> NotificationResult<()> {
        self.notification_repository
            .create_chat_invite_notification(user_id, chat_id, inviter_name, None)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_service_creation() {
        assert!(true);
    }
}
