//! Notification repository for database operations.

use crate::entities::{Notification};
use crate::entities::notification::{NotificationType, NotificationPriority};
use crate::types::{NotificationResult, CreateNotificationRequest};
use crate::types::errors::NotificationError;
use sqlx::{SqlitePool, Row};

/// Repository for notification database operations
pub struct NotificationRepository {
    pool: SqlitePool,
}

impl NotificationRepository {
    /// Create a new notification repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find notification by database ID (internal helper)
    async fn find_by_id_internal(&self, id: i64) -> NotificationResult<Option<Notification>> {
        let row = sqlx::query(
            "SELECT id, user_id, type, title, message, priority, is_read, created_at, updated_at, expires_at, related_entity_id, related_entity_type, metadata
             FROM notifications WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let notification_type_str: String = row.try_get("type")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let priority_str: String = row.try_get("priority")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let metadata_str: Option<String> = row.try_get("metadata")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

            Ok(Some(Notification {
                id: Some(row.try_get("id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?),
                user_id: row.try_get("user_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                notification_type: notification_type_str.parse().map_err(|_| NotificationError::InvalidNotificationType)?,
                title: row.try_get("title").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                message: row.try_get("message").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                priority: priority_str.parse().map_err(|_| NotificationError::InvalidPriority)?,
                is_read: row.try_get("is_read").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_id: row.try_get("related_entity_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_type: row.try_get("related_entity_type").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new notification
    pub async fn create(&self, request: &CreateNotificationRequest) -> NotificationResult<Notification> {
        let now = chrono::Utc::now().to_rfc3339();
        let metadata_json = request.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

        let result = sqlx::query(
            "INSERT INTO notifications (user_id, type, title, message, priority, is_read, created_at, updated_at, expires_at, related_entity_id, related_entity_type, metadata)
             VALUES (?, ?, ?, ?, ?, false, ?, ?, ?, ?, ?, ?)"
        )
        .bind(request.user_id)
        .bind(request.notification_type.to_string())
        .bind(&request.title)
        .bind(&request.message)
        .bind(request.priority.to_string())
        .bind(&now)
        .bind(&now)
        .bind(request.expires_at.as_ref())
        .bind(&request.related_entity_id)
        .bind(&request.related_entity_type)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        let notification_id = result.last_insert_rowid();
        self.find_by_id_internal(notification_id).await?.ok_or_else(|| {
            NotificationError::DatabaseError("Failed to retrieve created notification".to_string())
        })
    }

    /// Find notification by ID (public method)
    pub async fn find_by_id(&self, id: i64) -> NotificationResult<Option<Notification>> {
        self.find_by_id_internal(id).await
    }

    /// Find notifications for user
    pub async fn find_by_user_id(
        &self,
        user_id: i64,
        limit: u32,
        offset: u32,
    ) -> NotificationResult<Vec<Notification>> {
        let rows = sqlx::query(
            "SELECT id, user_id, type, title, message, priority, is_read, created_at, updated_at, expires_at, related_entity_id, related_entity_type, metadata
             FROM notifications WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        let mut notifications = Vec::new();
        for row in rows {
            let notification_type_str: String = row.try_get("type")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let priority_str: String = row.try_get("priority")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let metadata_str: Option<String> = row.try_get("metadata")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

            notifications.push(Notification {
                id: Some(row.try_get("id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?),
                user_id: row.try_get("user_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                notification_type: notification_type_str.parse().map_err(|_| NotificationError::InvalidNotificationType)?,
                title: row.try_get("title").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                message: row.try_get("message").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                priority: priority_str.parse().map_err(|_| NotificationError::InvalidPriority)?,
                is_read: row.try_get("is_read").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_id: row.try_get("related_entity_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_type: row.try_get("related_entity_type").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            });
        }

        Ok(notifications)
    }

    /// Find unread notifications for user
    pub async fn find_unread_by_user_id(
        &self,
        user_id: i64,
        limit: u32,
    ) -> NotificationResult<Vec<Notification>> {
        let rows = sqlx::query(
            "SELECT id, user_id, type, title, message, priority, is_read, created_at, updated_at, expires_at, related_entity_id, related_entity_type, metadata
             FROM notifications WHERE user_id = ? AND is_read = false ORDER BY created_at DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        let mut notifications = Vec::new();
        for row in rows {
            let notification_type_str: String = row.try_get("type")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let priority_str: String = row.try_get("priority")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
            let metadata_str: Option<String> = row.try_get("metadata")
                .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

            notifications.push(Notification {
                id: Some(row.try_get("id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?),
                user_id: row.try_get("user_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                notification_type: notification_type_str.parse().map_err(|_| NotificationError::InvalidNotificationType)?,
                title: row.try_get("title").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                message: row.try_get("message").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                priority: priority_str.parse().map_err(|_| NotificationError::InvalidPriority)?,
                is_read: row.try_get("is_read").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                expires_at: row.try_get("expires_at").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_id: row.try_get("related_entity_id").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                related_entity_type: row.try_get("related_entity_type").map_err(|e| NotificationError::DatabaseError(e.to_string()))?,
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            });
        }

        Ok(notifications)
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, id: i64, user_id: i64) -> NotificationResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE notifications SET is_read = true, updated_at = ? WHERE id = ? AND user_id = ?"
        )
        .bind(&now)
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Mark all notifications as read for user
    pub async fn mark_all_as_read(&self, user_id: i64) -> NotificationResult<u32> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE notifications SET is_read = true, updated_at = ? WHERE user_id = ? AND is_read = false"
        )
        .bind(&now)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Delete notification
    pub async fn delete(&self, id: i64, user_id: i64) -> NotificationResult<()> {
        sqlx::query(
            "DELETE FROM notifications WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get unread count for user
    pub async fn get_unread_count(&self, user_id: i64) -> NotificationResult<i64> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = ? AND is_read = false"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| NotificationError::DatabaseError(e.to_string()))?;

        let count: i64 = count.try_get("count").map_err(|e| NotificationError::DatabaseError(e.to_string()))?;
        Ok(count)
    }

    /// Create notification for new message
    pub async fn create_message_notification(
        &self,
        user_id: i64,
        chat_id: &str,
        message_content: &str,
        sender_name: &str,
    ) -> NotificationResult<Notification> {
        let request = CreateNotificationRequest {
            user_id,
            notification_type: NotificationType::Message,
            title: format!("New message from {}", sender_name),
            message: if message_content.len() > 100 {
                format!("{}...", &message_content[..100])
            } else {
                message_content.to_string()
            },
            priority: NotificationPriority::Normal,
            related_entity_id: Some(chat_id.to_string()),
            related_entity_type: Some("chat".to_string()),
            metadata: Some(serde_json::json!({
                "sender_name": sender_name,
                "chat_id": chat_id
            })),
            expires_at: None,
        };

        self.create(&request).await
    }

    /// Create notification for chat invite
    pub async fn create_chat_invite_notification(
        &self,
        user_id: i64,
        chat_id: &str,
        inviter_name: &str,
        chat_name: Option<&str>,
    ) -> NotificationResult<Notification> {
        let title = if let Some(chat_name) = chat_name {
            format!("Chat invite: {} from {}", chat_name, inviter_name)
        } else {
            format!("Chat invite from {}", inviter_name)
        };

        let request = CreateNotificationRequest {
            user_id,
            notification_type: NotificationType::ChatInvite,
            title,
            message: "You have been invited to join a chat".to_string(),
            priority: NotificationPriority::Normal,
            related_entity_id: Some(chat_id.to_string()),
            related_entity_type: Some("chat_invite".to_string()),
            metadata: Some(serde_json::json!({
                "inviter_name": inviter_name,
                "chat_id": chat_id,
                "chat_name": chat_name
            })),
            expires_at: Some((chrono::Utc::now() + chrono::Duration::days(7)).to_rfc3339()),
        };

        self.create(&request).await
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
        let db_path = temp_dir.path().join("test_notifications.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                type TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                priority TEXT NOT NULL,
                is_read BOOLEAN NOT NULL DEFAULT false,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT,
                related_entity_id TEXT,
                related_entity_type TEXT,
                metadata TEXT
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    fn create_test_notification_request() -> CreateNotificationRequest {
        CreateNotificationRequest {
            user_id: 1,
            notification_type: NotificationType::Message,
            title: "Test Notification".to_string(),
            message: "This is a test notification".to_string(),
            priority: NotificationPriority::Normal,
            related_entity_id: Some("chat_123".to_string()),
            related_entity_type: Some("chat".to_string()),
            metadata: Some(serde_json::json!({
                "sender_name": "Test User",
                "chat_id": "chat_123"
            })),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_notification() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request = create_test_notification_request();

        let notification = repo.create(&request).await.unwrap();

        assert_eq!(notification.user_id, request.user_id);
        assert_eq!(notification.title, request.title);
        assert_eq!(notification.message, request.message);
        assert_eq!(notification.notification_type, request.notification_type);
        assert_eq!(notification.priority, request.priority);
        assert!(!notification.is_read);
        assert!(notification.id.is_some());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request = create_test_notification_request();

        let created = repo.create(&request).await.unwrap();
        let found = repo.find_by_id(created.id.unwrap()).await.unwrap();

        assert!(found.is_some());
        let found_notification = found.unwrap();
        assert_eq!(found_notification.id, created.id);
        assert_eq!(found_notification.title, request.title);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request1 = create_test_notification_request();
        let mut request2 = create_test_notification_request();
        request2.title = "Second Notification".to_string();

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let notifications = repo.find_by_user_id(1, 10, 0).await.unwrap();

        assert_eq!(notifications.len(), 2);
        assert!(notifications.iter().any(|n| n.title == "Test Notification"));
        assert!(notifications.iter().any(|n| n.title == "Second Notification"));
    }

    #[tokio::test]
    async fn test_find_unread_by_user_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request = create_test_notification_request();

        let notification = repo.create(&request).await.unwrap();
        let unread = repo.find_unread_by_user_id(1, 10).await.unwrap();

        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].id, notification.id);

        // Mark as read and check again
        repo.mark_as_read(notification.id.unwrap(), 1).await.unwrap();
        let unread_after = repo.find_unread_by_user_id(1, 10).await.unwrap();

        assert_eq!(unread_after.len(), 0);
    }

    #[tokio::test]
    async fn test_mark_as_read() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request = create_test_notification_request();

        let notification = repo.create(&request).await.unwrap();
        assert!(!notification.is_read);

        repo.mark_as_read(notification.id.unwrap(), 1).await.unwrap();

        let updated = repo.find_by_id(notification.id.unwrap()).await.unwrap().unwrap();
        assert!(updated.is_read);
    }

    #[tokio::test]
    async fn test_mark_all_as_read() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request1 = create_test_notification_request();
        let mut request2 = create_test_notification_request();
        request2.title = "Second Notification".to_string();

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let unread_count_before = repo.get_unread_count(1).await.unwrap();
        assert_eq!(unread_count_before, 2);

        let marked_count = repo.mark_all_as_read(1).await.unwrap();
        assert_eq!(marked_count, 2);

        let unread_count_after = repo.get_unread_count(1).await.unwrap();
        assert_eq!(unread_count_after, 0);
    }

    #[tokio::test]
    async fn test_delete_notification() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request = create_test_notification_request();

        let notification = repo.create(&request).await.unwrap();
        let notification_id = notification.id.unwrap();

        repo.delete(notification_id, 1).await.unwrap();

        let found = repo.find_by_id(notification_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_unread_count() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);
        let request1 = create_test_notification_request();
        let mut request2 = create_test_notification_request();
        request2.title = "Second Notification".to_string();

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let count = repo.get_unread_count(1).await.unwrap();
        assert_eq!(count, 2);

        let notification = repo.find_by_user_id(1, 1, 0).await.unwrap().pop().unwrap();
        repo.mark_as_read(notification.id.unwrap(), 1).await.unwrap();

        let count_after = repo.get_unread_count(1).await.unwrap();
        assert_eq!(count_after, 1);
    }

    #[tokio::test]
    async fn test_create_message_notification() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);

        let notification = repo.create_message_notification(
            1,
            "chat_123",
            "Hello, this is a test message",
            "Test Sender"
        ).await.unwrap();

        assert_eq!(notification.user_id, 1);
        assert_eq!(notification.notification_type, NotificationType::Message);
        assert_eq!(notification.title, "New message from Test Sender");
        assert_eq!(notification.message, "Hello, this is a test message");
        assert_eq!(notification.related_entity_id, Some("chat_123".to_string()));
        assert_eq!(notification.related_entity_type, Some("chat".to_string()));
    }

    #[tokio::test]
    async fn test_create_chat_invite_notification() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);

        let notification = repo.create_chat_invite_notification(
            1,
            "chat_456",
            "Test Inviter",
            Some("Test Chat")
        ).await.unwrap();

        assert_eq!(notification.user_id, 1);
        assert_eq!(notification.notification_type, NotificationType::ChatInvite);
        assert_eq!(notification.title, "Chat invite: Test Chat from Test Inviter");
        assert_eq!(notification.message, "You have been invited to join a chat");
        assert_eq!(notification.related_entity_id, Some("chat_456".to_string()));
        assert_eq!(notification.related_entity_type, Some("chat_invite".to_string()));
    }

    #[tokio::test]
    async fn test_long_message_truncation() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = NotificationRepository::new(pool);

        let long_message = "a".repeat(200);
        let notification = repo.create_message_notification(
            1,
            "chat_123",
            &long_message,
            "Test Sender"
        ).await.unwrap();

        assert!(notification.message.len() <= 103); // "..." + 100 chars
        assert!(notification.message.ends_with("..."));
    }
}