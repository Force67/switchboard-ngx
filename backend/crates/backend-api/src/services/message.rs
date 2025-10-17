use sqlx::SqlitePool;
use uuid::Uuid;
use crate::routes::models::{CreateMessageRequest, Message, MessageEdit};
use super::error::ServiceError;

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

pub async fn get_messages(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
) -> Result<Vec<Message>, ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, public_id, chat_id, user_id, content, role, model, message_type,
               thread_id, reply_to_id, created_at, updated_at
        FROM messages
        WHERE chat_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn create_message(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
    req: CreateMessageRequest,
) -> Result<(Message, Vec<i64>), ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Resolve reply_to_id if provided
    let reply_to_db_id = if let Some(reply_to_public_id) = &req.reply_to_id {
        sqlx::query_scalar::<_, i64>("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(reply_to_public_id)
            .bind(chat_db_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // Resolve thread_id if provided
    let thread_db_id = if let Some(thread_public_id) = &req.thread_id {
        sqlx::query_scalar::<_, i64>("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(thread_public_id)
            .bind(chat_db_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let message_type = req.message_type.unwrap_or_else(|| "text".to_string());

    // Create the message
    let message_db_id = sqlx::query(
        r#"
        INSERT INTO messages (public_id, chat_id, user_id, content, message_type, role, model, thread_id, reply_to_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(chat_db_id)
    .bind(user_id)
    .bind(&req.content)
    .bind(&message_type)
    .bind(&req.role)
    .bind(&req.model)
    .bind(thread_db_id)
    .bind(reply_to_db_id)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Fetch the created message
    let message = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, public_id, chat_id, user_id, content, role, model, message_type,
               thread_id, reply_to_id, created_at, updated_at
        FROM messages
        WHERE id = ?
        "#,
    )
    .bind(message_db_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::internal("Failed to fetch created message"))?;

    let member_ids = fetch_chat_member_ids(pool, chat_db_id).await?;

    Ok((message, member_ids))
}

pub async fn update_message(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    user_id: i64,
    new_content: String,
) -> Result<(Message, Vec<i64>), ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    // Get the original message
    let original_message: Option<(i64, String)> =
        sqlx::query_as("SELECT id, content FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(message_public_id)
            .bind(chat_db_id)
            .fetch_optional(pool)
            .await?;

    let (message_db_id, original_content) =
        original_message.ok_or_else(|| ServiceError::not_found("Message not found"))?;

    // Check if user can edit this message (owner or admin)
    let can_edit = check_message_edit_permission(pool, message_db_id, chat_db_id, user_id).await?;

    if !can_edit {
        return Err(ServiceError::forbidden("Cannot edit this message"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Create audit entry for the edit
    sqlx::query(
        r#"
        INSERT INTO message_edits (message_id, edited_by_user_id, old_content, new_content, edited_at)
        VALUES (?, ?, ?, ?, ?)
        "#
    )
    .bind(message_db_id)
    .bind(user_id)
    .bind(&original_content)
    .bind(&new_content)
    .bind(&now)
    .execute(pool)
    .await?;

    // Update the message
    sqlx::query(
        r#"
        UPDATE messages
        SET content = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&new_content)
    .bind(&now)
    .bind(message_db_id)
    .execute(pool)
    .await?;

    // Fetch the updated message
    let message = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, public_id, chat_id, user_id, content, role, model, message_type,
               thread_id, reply_to_id, created_at, updated_at
        FROM messages
        WHERE id = ?
        "#,
    )
    .bind(message_db_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::internal("Failed to fetch updated message"))?;

    let member_ids = fetch_chat_member_ids(pool, chat_db_id).await?;

    Ok((message, member_ids))
}

pub async fn delete_message(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    user_id: i64,
) -> Result<(Vec<i64>, String), ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    // Get the message details
    let message_details: Option<(i64, i64)> =
        sqlx::query_as("SELECT id, user_id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(message_public_id)
            .bind(chat_db_id)
            .fetch_optional(pool)
            .await?;

    let (message_db_id, _message_user_id) =
        message_details.ok_or_else(|| ServiceError::not_found("Message not found"))?;

    // Check if user can delete this message
    let can_delete = check_message_edit_permission(pool, message_db_id, chat_db_id, user_id).await?;

    if !can_delete {
        return Err(ServiceError::forbidden("Cannot delete this message"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Create audit entry for the deletion
    sqlx::query(
        r#"
        INSERT INTO message_deletions (message_id, deleted_by_user_id, reason, deleted_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(message_db_id)
    .bind(user_id)
    .bind("User deleted message")
    .bind(&now)
    .execute(pool)
    .await?;

    let member_ids = fetch_chat_member_ids(pool, chat_db_id).await?;

    // Delete the message (cascade will handle related records)
    sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(message_db_id)
        .execute(pool)
        .await?;

    Ok((member_ids, message_public_id.to_string()))
}

pub async fn get_message_edits(
    pool: &SqlitePool,
    chat_id: &str,
    message_public_id: &str,
    user_id: i64,
) -> Result<Vec<MessageEdit>, ServiceError> {
    let chat_db_id = check_chat_membership(pool, chat_id, user_id).await?;

    // Get the message ID
    let message_db_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(message_public_id)
            .bind(chat_db_id)
            .fetch_optional(pool)
            .await?;

    let message_db_id = message_db_id.ok_or_else(|| ServiceError::not_found("Message not found"))?;

    // Get edit history
    let edits = sqlx::query_as::<_, MessageEdit>(
        r#"
        SELECT id, message_id, edited_by_user_id, old_content, new_content, edited_at
        FROM message_edits
        WHERE message_id = ?
        ORDER BY edited_at DESC
        "#,
    )
    .bind(message_db_id)
    .fetch_all(pool)
    .await?;

    Ok(edits)
}

async fn fetch_chat_member_ids(pool: &SqlitePool, chat_db_id: i64) -> Result<Vec<i64>, ServiceError> {
    let member_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT user_id FROM chat_members
        WHERE chat_id = ?
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(pool)
    .await?;

    Ok(member_ids)
}

async fn check_message_edit_permission(
    pool: &SqlitePool,
    message_db_id: i64,
    chat_db_id: i64,
    user_id: i64,
) -> Result<bool, ServiceError> {
    // Check if user owns the message
    let message_owner: i64 = sqlx::query_scalar("SELECT user_id FROM messages WHERE id = ?")
        .bind(message_db_id)
        .fetch_one(pool)
        .await?;

    if message_owner == user_id {
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