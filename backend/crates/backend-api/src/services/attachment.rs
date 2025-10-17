use sqlx::SqlitePool;
use crate::routes::models::{CreateAttachmentRequest, MessageAttachment};
use super::error::ServiceError;

/// Check if user is a member of the chat and return chat database ID
pub async fn check_chat_membership(pool: &SqlitePool, chat_id: &str, user_id: i64) -> Result<i64, ServiceError> {
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    chat_db_id.ok_or_else(|| ServiceError::forbidden("Not a member of this chat"))
}

/// Resolve message public ID to database ID
pub async fn resolve_message_id(pool: &SqlitePool, message_public_id: &str, chat_db_id: i64) -> Result<i64, ServiceError> {
    let message_db_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM messages WHERE public_id = ? AND chat_id = ?"
    )
    .bind(message_public_id)
    .bind(chat_db_id)
    .fetch_optional(pool)
    .await?;

    message_db_id.ok_or_else(|| ServiceError::not_found("Message not found"))
}

/// Get attachments for a message
pub async fn get_message_attachments(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    user_id: i64,
) -> Result<Vec<MessageAttachment>, ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;
    let message_db_id = resolve_message_id(pool, message_public_id, chat_db_id).await?;

    let attachments = sqlx::query_as::<_, MessageAttachment>(
        r#"
        SELECT id, message_id, file_name, file_type, file_url, file_size_bytes, created_at
        FROM message_attachments
        WHERE message_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(message_db_id)
    .fetch_all(pool)
    .await?;

    Ok(attachments)
}

/// Create attachment for a message
pub async fn create_message_attachment(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    user_id: i64,
    req: CreateAttachmentRequest,
) -> Result<MessageAttachment, ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;
    let message_db_id = resolve_message_id(pool, message_public_id, chat_db_id).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Create the attachment
    let attachment_db_id = sqlx::query(
        r#"
        INSERT INTO message_attachments (message_id, file_name, file_type, file_url, file_size_bytes, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(message_db_id)
    .bind(&req.file_name)
    .bind(&req.file_type)
    .bind(&req.file_url)
    .bind(req.file_size_bytes)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Fetch the created attachment
    let attachment = sqlx::query_as::<_, MessageAttachment>(
        r#"
        SELECT id, message_id, file_name, file_type, file_url, file_size_bytes, created_at
        FROM message_attachments
        WHERE id = ?
        "#,
    )
    .bind(attachment_db_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::internal("Failed to fetch created attachment"))?;

    Ok(attachment)
}

/// Delete an attachment
pub async fn delete_attachment(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    attachment_id: i64,
    user_id: i64,
) -> Result<(), ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    // Verify attachment belongs to this message/chat
    let message_details: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT ma.message_id
        FROM message_attachments ma
        JOIN messages m ON ma.id = ? AND ma.message_id = m.id
        WHERE m.public_id = ? AND m.chat_id = ?
        "#,
    )
    .bind(attachment_id)
    .bind(message_public_id)
    .bind(chat_db_id)
    .fetch_optional(pool)
    .await?;

    let message_db_id = message_details.ok_or_else(|| ServiceError::not_found("Attachment not found"))?.0;

    // Check if user can delete attachments from this message
    let can_delete = check_attachment_delete_permission(pool, message_public_id, chat_db_id, user_id).await?;

    if !can_delete {
        return Err(ServiceError::forbidden("Cannot delete this attachment"));
    }

    // Delete the attachment
    let result = sqlx::query("DELETE FROM message_attachments WHERE id = ?")
        .bind(attachment_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::not_found("Attachment not found"));
    }

    Ok(())
}

/// Check if user can delete attachments from a message (owner or admin)
async fn check_attachment_delete_permission(
    pool: &SqlitePool,
    message_public_id: &str,
    chat_db_id: i64,
    user_id: i64,
) -> Result<bool, ServiceError> {
    // Check if user owns the message
    let message_user_id: Option<i64> = sqlx::query_scalar(
        "SELECT user_id FROM messages WHERE public_id = ? AND chat_id = ?"
    )
    .bind(message_public_id)
    .bind(chat_db_id)
    .fetch_optional(pool)
    .await?;

    if message_user_id == Some(user_id) {
        return Ok(true);
    }

    // Check if user is admin or owner of the chat
    let user_role: Option<String> = sqlx::query_scalar(
        "SELECT cm.role FROM chat_members cm WHERE cm.chat_id = ? AND cm.user_id = ?",
    )
    .bind(chat_db_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(matches!(user_role.as_deref(), Some("admin") | Some("owner")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_utils::{create_test_db, create_test_user, create_test_chat, create_test_message};
    use crate::services::test_utils::fixtures::*;

    #[tokio::test]
    async fn test_get_message_attachments_success() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test user
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user");

        // Create test chat
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Create test message
        let message_id = create_test_message(&pool, chat_id, TEST_USER_ID, "Test message", "user")
            .await.expect("Failed to create message");

        // Create test attachment
        let attachment_id = create_test_attachment(&pool, message_id, "test.pdf", "application/pdf", "http://example.com/test.pdf", 1024)
            .await.expect("Failed to create attachment");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id: String = sqlx::query_scalar("SELECT public_id FROM messages WHERE id = ?")
            .bind(message_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get message public ID");

        let result = get_message_attachments(&pool, &chat_public_id, &message_public_id, TEST_USER_ID).await;

        assert!(result.is_ok());
        let attachments = result.unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].file_name, "test.pdf");
        assert_eq!(attachments[0].file_type, "application/pdf");
        assert_eq!(attachments[0].file_size_bytes, 1024);
    }

    #[tokio::test]
    async fn test_get_message_attachments_not_member() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test users
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user 1");
        create_test_user(&pool, TEST_USER_ID_2, "test-user-456", Some("test2@example.com"), Some("Test User 2"))
            .await.expect("Failed to create test user 2");

        // Create test chat with user 1
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id = "non-existent-message";

        // Try to get attachments as user 2 (not a member)
        let result = get_message_attachments(&pool, &chat_public_id, message_public_id, TEST_USER_ID_2).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::Forbidden => {} // Expected
            _ => panic!("Expected ServiceError::Forbidden"),
        }
    }

    #[tokio::test]
    async fn test_create_message_attachment_success() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test user
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user");

        // Create test chat
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Create test message
        let message_id = create_test_message(&pool, chat_id, TEST_USER_ID, "Test message", "user")
            .await.expect("Failed to create message");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id: String = sqlx::query_scalar("SELECT public_id FROM messages WHERE id = ?")
            .bind(message_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get message public ID");

        let create_req = CreateAttachmentRequest {
            file_name: "test.pdf".to_string(),
            file_type: "application/pdf".to_string(),
            file_url: "http://example.com/test.pdf".to_string(),
            file_size_bytes: 2048,
        };

        let result = create_message_attachment(&pool, &chat_public_id, &message_public_id, TEST_USER_ID, create_req).await;

        assert!(result.is_ok());
        let attachment = result.unwrap();
        assert_eq!(attachment.file_name, "test.pdf");
        assert_eq!(attachment.file_type, "application/pdf");
        assert_eq!(attachment.file_url, "http://example.com/test.pdf");
        assert_eq!(attachment.file_size_bytes, 2048);
    }

    #[tokio::test]
    async fn test_delete_attachment_success() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test user
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user");

        // Create test chat
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Create test message
        let message_id = create_test_message(&pool, chat_id, TEST_USER_ID, "Test message", "user")
            .await.expect("Failed to create message");

        // Create test attachment
        let attachment_id = create_test_attachment(&pool, message_id, "test.pdf", "application/pdf", "http://example.com/test.pdf", 1024)
            .await.expect("Failed to create attachment");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id: String = sqlx::query_scalar("SELECT public_id FROM messages WHERE id = ?")
            .bind(message_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get message public ID");

        let result = delete_attachment(&pool, &chat_public_id, &message_public_id, attachment_id, TEST_USER_ID).await;

        assert!(result.is_ok());

        // Verify attachment is deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message_attachments WHERE id = ?")
            .bind(attachment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to check if attachment exists");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_attachment_not_owner() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test users
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user 1");
        create_test_user(&pool, TEST_USER_ID_2, "test-user-456", Some("test2@example.com"), Some("Test User 2"))
            .await.expect("Failed to create test user 2");

        // Create test chat with user 1
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Create test message by user 1
        let message_id = create_test_message(&pool, chat_id, TEST_USER_ID, "Test message", "user")
            .await.expect("Failed to create message");

        // Add user 2 as a regular member (not admin/owner)
        add_chat_member(&pool, chat_id, TEST_USER_ID_2, "member")
            .await.expect("Failed to add user 2 to chat");

        // Create test attachment
        let attachment_id = create_test_attachment(&pool, message_id, "test.pdf", "application/pdf", "http://example.com/test.pdf", 1024)
            .await.expect("Failed to create attachment");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id: String = sqlx::query_scalar("SELECT public_id FROM messages WHERE id = ?")
            .bind(message_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get message public ID");

        // Try to delete attachment as user 2 (not owner)
        let result = delete_attachment(&pool, &chat_public_id, &message_public_id, attachment_id, TEST_USER_ID_2).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::Forbidden => {} // Expected
            _ => panic!("Expected ServiceError::Forbidden"),
        }
    }

    #[tokio::test]
    async fn test_delete_attachment_as_admin() {
        let (pool, _temp_dir) = create_test_db().await;

        // Create test users
        create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME))
            .await.expect("Failed to create test user 1");
        create_test_user(&pool, TEST_USER_ID_2, "test-user-456", Some("test2@example.com"), Some("Test User 2"))
            .await.expect("Failed to create test user 2");

        // Create test chat with user 1
        let chat_id = create_test_chat(&pool, TEST_USER_ID, "Test Chat", TEST_CHAT_TYPE)
            .await.expect("Failed to create chat");

        // Create test message by user 1
        let message_id = create_test_message(&pool, chat_id, TEST_USER_ID, "Test message", "user")
            .await.expect("Failed to create message");

        // Add user 2 as an admin
        add_chat_member(&pool, chat_id, TEST_USER_ID_2, "admin")
            .await.expect("Failed to add user 2 as admin");

        // Create test attachment
        let attachment_id = create_test_attachment(&pool, message_id, "test.pdf", "application/pdf", "http://example.com/test.pdf", 1024)
            .await.expect("Failed to create attachment");

        // Get public IDs
        let chat_public_id: String = sqlx::query_scalar("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get chat public ID");

        let message_public_id: String = sqlx::query_scalar("SELECT public_id FROM messages WHERE id = ?")
            .bind(message_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to get message public ID");

        // Delete attachment as user 2 (admin)
        let result = delete_attachment(&pool, &chat_public_id, &message_public_id, attachment_id, TEST_USER_ID_2).await;

        assert!(result.is_ok());

        // Verify attachment is deleted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM message_attachments WHERE id = ?")
            .bind(attachment_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to check if attachment exists");
        assert_eq!(count, 0);
    }

    // Helper function to create test attachment
    async fn create_test_attachment(
        pool: &SqlitePool,
        message_id: i64,
        file_name: &str,
        file_type: &str,
        file_url: &str,
        file_size_bytes: i64,
    ) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO message_attachments (message_id, file_name, file_type, file_url, file_size_bytes, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(message_id)
        .bind(file_name)
        .bind(file_type)
        .bind(file_url)
        .bind(file_size_bytes)
        .bind(&now)
        .execute(pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    // Helper function to add chat member
    async fn add_chat_member(
        pool: &SqlitePool,
        chat_id: i64,
        user_id: i64,
        role: &str,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO chat_members (chat_id, user_id, role, joined_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(role)
        .bind(&now)
        .execute(pool)
        .await?;

        Ok(())
    }
}