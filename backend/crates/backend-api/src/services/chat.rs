use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use crate::routes::models::{Chat, CreateChatRequest, UpdateChatRequest};
use crate::routes::chats::ChatWithMessages;
use super::error::ServiceError;

pub async fn list_chats(pool: &SqlitePool, user_id: i64) -> Result<Vec<ChatWithMessages>, ServiceError> {
    let chats = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        WHERE c.id IN (
            SELECT chat_id FROM chat_members WHERE user_id = ?
        )
        ORDER BY c.updated_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Add messages to each chat so the UI can hydrate its local stores on refresh
    let mut chats_with_messages = Vec::with_capacity(chats.len());
    for chat in chats {
        let messages_json = fetch_chat_messages(chat.id, pool).await?;
        let is_group = chat.chat_type.eq_ignore_ascii_case("group");
        let chat_with_messages = ChatWithMessages {
            id: chat.id,
            public_id: chat.public_id,
            user_id: chat.user_id,
            folder_id: chat.folder_id,
            title: chat.title,
            chat_type: chat.chat_type,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
            is_group,
            messages: messages_json,
        };
        chats_with_messages.push(chat_with_messages);
    }

    Ok(chats_with_messages)
}

pub async fn create_chat(
    pool: &SqlitePool,
    user_id: i64,
    req: CreateChatRequest,
) -> Result<Chat, ServiceError> {
    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let folder_db_id = if let Some(folder_public_id) = &req.folder_id {
        // Resolve folder ID from public_id
        sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ? AND user_id = ?")
            .bind(folder_public_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // Create the chat first (user_id is now nullable, managed through chat_members)
    let chat_db_id = sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, chat_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&public_id)
    .bind(user_id) // Set user_id for backwards compatibility
    .bind(folder_db_id)
    .bind(&req.title)
    .bind(&req.chat_type)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Insert initial messages if provided
    for message in &req.messages {
        let message_public_id = Uuid::new_v4().to_string();
        let message_type = if message.role.eq_ignore_ascii_case("system") {
            "system"
        } else {
            "text"
        };
        sqlx::query(
            r#"
            INSERT INTO messages (public_id, chat_id, user_id, content, message_type, role, model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message_public_id)
        .bind(chat_db_id)
        .bind(user_id)
        .bind(&message.content)
        .bind(message_type)
        .bind(message.role.as_str())
        .bind(message.model.clone())
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    // Add creator as owner of the chat (for both regular and group chats)
    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(chat_db_id)
    .bind(user_id)
    .bind("owner")
    .bind(&now)
    .execute(pool)
    .await?;

    let chat = Chat {
        id: chat_db_id,
        public_id: public_id.clone(),
        user_id: Some(user_id),
        folder_id: folder_db_id,
        title: req.title.clone(),
        chat_type: req.chat_type.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    Ok(chat)
}

pub async fn get_chat(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
) -> Result<Chat, ServiceError> {
    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::not_found("Chat not found"))?;

    Ok(chat)
}

pub async fn update_chat(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
    req: UpdateChatRequest,
) -> Result<(Chat, Vec<i64>), ServiceError> {
    let now = chrono::Utc::now().to_rfc3339();

    let mut folder_update_requested = false;
    let mut folder_set_null = false;
    let mut folder_db_id: Option<i64> = None;

    if let Some(folder_public_id) = &req.folder_id {
        folder_update_requested = true;
        if folder_public_id.is_empty() {
            folder_set_null = true;
        } else {
            folder_db_id = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM folders WHERE public_id = ? AND user_id = ?",
            )
            .bind(folder_public_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

            if folder_db_id.is_none() {
                return Err(ServiceError::not_found("Folder not found"));
            }
        }
    }

    let update_folder_flag: i32 = if folder_update_requested { 1 } else { 0 };
    let set_folder_null_flag: i32 = if folder_set_null { 1 } else { 0 };

    sqlx::query(
        r#"
        UPDATE chats
        SET title = COALESCE(?, title),
            folder_id = CASE
                WHEN ? = 0 THEN folder_id
                WHEN ? = 1 THEN NULL
                ELSE ?
            END,
            updated_at = ?
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&req.title)
    .bind(update_folder_flag)
    .bind(set_folder_null_flag)
    .bind(folder_db_id)
    .bind(&now)
    .bind(chat_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::not_found("Chat not found"))?;

    let member_ids = fetch_chat_member_ids(pool, chat.id).await?;

    Ok((chat, member_ids))
}

pub async fn delete_chat(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
) -> Result<Vec<i64>, ServiceError> {
    // Check if user is owner of the chat before deleting
    let chat_info: Option<(i64, String)> = sqlx::query_as(
        r#"
        SELECT c.id, cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (chat_db_id, chat_role) = chat_info.ok_or_else(|| ServiceError::not_found("Chat not found"))?;

    if chat_role != "owner" {
        return Err(ServiceError::forbidden("Only chat owners can delete chats"));
    }

    let member_ids = fetch_chat_member_ids(pool, chat_db_id).await?;

    sqlx::query("DELETE FROM chats WHERE public_id = ?")
        .bind(chat_id)
        .execute(pool)
        .await?;

    Ok(member_ids)
}

async fn fetch_chat_messages(
    chat_id: i64,
    pool: &SqlitePool,
) -> Result<Option<String>, ServiceError> {
    let rows = sqlx::query(
        r#"
        SELECT public_id, user_id, content, role, model, message_type, created_at
        FROM messages
        WHERE chat_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(Some("[]".to_string()));
    }

    let mut messages = Vec::with_capacity(rows.len());
    for row in rows {
        let role: String = row.get("role");
        let content: String = row.get("content");
        let model: Option<String> = row.try_get("model").unwrap_or(None);
        let message_json = serde_json::json!({
            "id": row.get::<String, _>("public_id"),
            "user_id": row.get::<i64, _>("user_id"),
            "role": role,
            "content": content,
            "model": model,
            "timestamp": row.get::<String, _>("created_at"),
            "message_type": row.get::<String, _>("message_type"),
        });
        messages.push(message_json);
    }

    serde_json::to_string(&messages).map(Some).map_err(|e| {
        tracing::error!("Failed to serialize messages for chat {}: {}", chat_id, e);
        ServiceError::internal("Failed to serialize chat messages")
    })
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

pub async fn fetch_chat_member_ids_by_id(pool: &SqlitePool, chat_db_id: i64) -> Result<Vec<i64>, ServiceError> {
    fetch_chat_member_ids(pool, chat_db_id).await
}