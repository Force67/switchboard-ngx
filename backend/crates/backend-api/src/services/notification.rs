use sqlx::SqlitePool;
use crate::routes::models::{Notification, MarkNotificationReadRequest};
use super::error::ServiceError;

pub async fn list_notifications(
    pool: &SqlitePool,
    user_id: i64,
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Notification>, ServiceError> {
    let notifications = if unread_only {
        sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, type, title, body, read, created_at
            FROM notifications
            WHERE user_id = ? AND read = FALSE
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Notification>(
            r#"
            SELECT id, user_id, type, title, body, read, created_at
            FROM notifications
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(notifications)
}

pub async fn get_unread_count(pool: &SqlitePool, user_id: i64) -> Result<i64, ServiceError> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = ? AND read = FALSE")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn mark_notification_read(
    pool: &SqlitePool,
    user_id: i64,
    notification_id: i64,
    req: MarkNotificationReadRequest,
) -> Result<Notification, ServiceError> {
    // Check if notification exists and belongs to user
    let existing_notification: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM notifications WHERE id = ? AND user_id = ?")
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if existing_notification.is_none() {
        return Err(ServiceError::NotFound);
    }

    // Update the notification
    sqlx::query("UPDATE notifications SET read = ? WHERE id = ? AND user_id = ?")
        .bind(req.read)
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Fetch the updated notification
    let notification = sqlx::query_as::<_, Notification>(
        r#"
        SELECT id, user_id, type, title, body, read, created_at
        FROM notifications
        WHERE id = ?
        "#,
    )
    .bind(notification_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::Internal("Failed to fetch updated notification".to_string()))?;

    Ok(notification)
}

pub async fn mark_all_read(pool: &SqlitePool, user_id: i64) -> Result<u64, ServiceError> {
    let result = sqlx::query("UPDATE notifications SET read = TRUE WHERE user_id = ? AND read = FALSE")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn delete_notification(
    pool: &SqlitePool,
    user_id: i64,
    notification_id: i64,
) -> Result<(), ServiceError> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = ? AND user_id = ?")
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::NotFound);
    }

    Ok(())
}

pub async fn create_notification(
    pool: &SqlitePool,
    user_id: i64,
    notification_type: &str,
    title: &str,
    body: &str,
) -> Result<i64, ServiceError> {
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, title, body, read, created_at)
        VALUES (?, ?, ?, ?, FALSE, ?)
        "#,
    )
    .bind(user_id)
    .bind(notification_type)
    .bind(title)
    .bind(body)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn notify_new_message(
    pool: &SqlitePool,
    chat_id: i64,
    sender_user_id: i64,
    sender_name: &str,
    chat_title: &str,
) -> Result<Vec<i64>, ServiceError> {
    // Get all members of the chat except the sender
    let members = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT u.id
        FROM users u
        JOIN chat_members cm ON u.id = cm.user_id
        WHERE cm.chat_id = ? AND cm.user_id != ?
        "#,
    )
    .bind(chat_id)
    .bind(sender_user_id)
    .fetch_all(pool)
    .await?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut notification_ids = Vec::new();

    // Create notifications for all members
    for user_id in &members {
        let title = format!("New message in {}", chat_title);
        let body = format!("{} sent a message", sender_name);

        let result = sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, title, body, read, created_at)
            VALUES (?, ?, ?, ?, FALSE, ?)
            "#,
        )
        .bind(user_id)
        .bind("new_message")
        .bind(&title)
        .bind(&body)
        .bind(&now)
        .execute(pool)
        .await?;

        notification_ids.push(result.last_insert_rowid());
    }

    Ok(notification_ids)
}

pub async fn notify_chat_invite(
    pool: &SqlitePool,
    invited_user_id: i64,
    chat_title: &str,
    inviter_name: &str,
) -> Result<i64, ServiceError> {
    let title = format!("Chat invite: {}", chat_title);
    let body = format!("{} invited you to join a chat", inviter_name);

    create_notification(pool, invited_user_id, "chat_invite", &title, &body).await
}

pub async fn notify_invite_accepted(
    pool: &SqlitePool,
    inviter_user_id: i64,
    accepted_user_name: &str,
    chat_title: &str,
) -> Result<i64, ServiceError> {
    let title = format!("Invite accepted for {}", chat_title);
    let body = format!("{} accepted your chat invite", accepted_user_name);

    create_notification(pool, inviter_user_id, "invite_accepted", &title, &body).await
}

#[allow(dead_code)]
pub async fn notify_mention(
    pool: &SqlitePool,
    mentioned_user_id: i64,
    sender_name: &str,
    chat_title: &str,
) -> Result<i64, ServiceError> {
    let title = format!("You were mentioned in {}", chat_title);
    let body = format!("{} mentioned you in a message", sender_name);

    create_notification(pool, mentioned_user_id, "mention", &title, &body).await
}

// Service functions that handle business logic for API routes

/// Get user notifications with parameter handling and business logic
pub async fn get_user_notifications(
    pool: &SqlitePool,
    user_id: i64,
    unread_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::routes::models::Notification>, ServiceError> {
    let unread_only = unread_only.unwrap_or(false);
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    list_notifications(
        pool,
        user_id,
        unread_only,
        limit,
        offset,
    ).await
}

/// Get unread count for a user
pub async fn get_user_unread_count(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<i64, ServiceError> {
    get_unread_count(pool, user_id).await
}

/// Mark notification as read/unread with validation
pub async fn mark_notification_as_read(
    pool: &SqlitePool,
    user_id: i64,
    notification_id: i64,
    request: crate::routes::models::MarkNotificationReadRequest,
) -> Result<crate::routes::models::Notification, ServiceError> {
    mark_notification_read(
        pool,
        user_id,
        notification_id,
        request,
    ).await
}

/// Mark all notifications as read for a user
pub async fn mark_all_user_notifications_read(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<u64, ServiceError> {
    mark_all_read(pool, user_id).await
}

/// Delete a notification with ownership validation
pub async fn delete_user_notification(
    pool: &SqlitePool,
    user_id: i64,
    notification_id: i64,
) -> Result<(), ServiceError> {
    delete_notification(pool, user_id, notification_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_utils::{create_test_db, create_test_user};
    use crate::services::test_utils::fixtures::*;

    #[tokio::test]
    async fn test_list_notifications_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Test empty list
        let result = list_notifications(&pool, user.id, false, 50, 0).await;
        assert!(result.is_ok());
        let notifications = result.unwrap();
        assert_eq!(notifications.len(), 0);
    }

    #[tokio::test]
    async fn test_create_notification_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let result = create_notification(
            &pool,
            user.id,
            "test_type",
            "Test Title",
            "Test Body"
        ).await;

        assert!(result.is_ok());
        let notification_id = result.unwrap();
        assert!(notification_id > 0);
    }

    #[tokio::test]
    async fn test_get_unread_count_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Test empty count
        let result = get_unread_count(&pool, user.id).await;
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 0);

        // Create a notification
        create_notification(&pool, user.id, "test", "Test", "Test").await.unwrap();

        // Test count with notification
        let result = get_unread_count(&pool, user.id).await;
        assert!(result.is_ok());
        let count = result.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_mark_notification_read_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Create a notification first
        let notification_id = create_notification(&pool, user.id, "test", "Test", "Test").await.unwrap();

        // Mark as read
        let req = MarkNotificationReadRequest { read: true };
        let result = mark_notification_read(&pool, user.id, notification_id, req).await;
        assert!(result.is_ok());
        let notification = result.unwrap();
        assert_eq!(notification.id, notification_id);
        assert!(notification.read);
    }

    #[tokio::test]
    async fn test_mark_notification_read_not_found() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let req = MarkNotificationReadRequest { read: true };
        let result = mark_notification_read(&pool, user.id, 999, req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::NotFound));
    }

    #[tokio::test]
    async fn test_mark_all_read_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Create multiple notifications
        create_notification(&pool, user.id, "test1", "Test1", "Test1").await.unwrap();
        create_notification(&pool, user.id, "test2", "Test2", "Test2").await.unwrap();

        // Mark all as read
        let result = mark_all_read(&pool, user.id).await;
        assert!(result.is_ok());
        let updated_count = result.unwrap();
        assert_eq!(updated_count, 2);

        // Verify unread count is now 0
        let unread_count = get_unread_count(&pool, user.id).await.unwrap();
        assert_eq!(unread_count, 0);
    }

    #[tokio::test]
    async fn test_delete_notification_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Create a notification first
        let notification_id = create_notification(&pool, user.id, "test", "Test", "Test").await.unwrap();

        // Delete the notification
        let result = delete_notification(&pool, user.id, notification_id).await;
        assert!(result.is_ok());

        // Verify it's deleted
        let notifications = list_notifications(&pool, user.id, false, 50, 0).await.unwrap();
        assert_eq!(notifications.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_notification_not_found() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let result = delete_notification(&pool, user.id, 999).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::NotFound));
    }

    #[tokio::test]
    async fn test_list_unread_only_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Create a notification
        create_notification(&pool, user.id, "test", "Test", "Test").await.unwrap();

        // Test unread_only = true
        let result = list_notifications(&pool, user.id, true, 50, 0).await;
        assert!(result.is_ok());
        let notifications = result.unwrap();
        assert_eq!(notifications.len(), 1);
        assert!(!notifications[0].read);

        // Mark as read
        let req = MarkNotificationReadRequest { read: true };
        mark_notification_read(&pool, user.id, notifications[0].id, req).await.unwrap();

        // Test unread_only = true again
        let result = list_notifications(&pool, user.id, true, 50, 0).await;
        assert!(result.is_ok());
        let notifications = result.unwrap();
        assert_eq!(notifications.len(), 0);

        // Test unread_only = false
        let result = list_notifications(&pool, user.id, false, 50, 0).await;
        assert!(result.is_ok());
        let notifications = result.unwrap();
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].read);
    }
}