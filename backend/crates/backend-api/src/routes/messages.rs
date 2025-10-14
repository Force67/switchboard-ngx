use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    routes::models::{
        CreateMessageRequest, Message, MessageEdit, MessageEditsResponse, MessageResponse,
        MessagesResponse, UpdateMessageRequest,
    },
    state::ServerEvent,
    util::require_bearer,
    ApiError, AppState,
};

// Get messages for a chat
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages",
    tag = "Messages",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    responses(
        (status = 200, description = "List chat messages", body = MessagesResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch messages", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_messages(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<MessagesResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

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
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch messages: {}", e);
        ApiError::internal_server_error("Failed to fetch messages")
    })?;

    Ok(Json(MessagesResponse { messages }))
}

async fn fetch_chat_member_ids(state: &AppState, chat_db_id: i64) -> Result<Vec<i64>, ApiError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT user_id FROM chat_members
        WHERE chat_id = ?
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to fetch chat members for chat {}: {}",
            chat_db_id,
            e
        );
        ApiError::internal_server_error("Failed to fetch chat members")
    })
}

// Create a new message
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/messages",
    tag = "Messages",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    request_body = CreateMessageRequest,
    responses(
        (status = 200, description = "Message created", body = MessageResponse),
        (status = 400, description = "Invalid message payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create message", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Resolve reply_to_id if provided
    let reply_to_db_id = if let Some(reply_to_public_id) = &req.reply_to_id {
        sqlx::query_scalar::<_, i64>("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(reply_to_public_id)
            .bind(chat_db_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve reply_to message: {}", e);
                ApiError::internal_server_error("Failed to resolve reply_to message")
            })?
    } else {
        None
    };

    // Resolve thread_id if provided
    let thread_db_id = if let Some(thread_public_id) = &req.thread_id {
        sqlx::query_scalar::<_, i64>("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(thread_public_id)
            .bind(chat_db_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve thread message: {}", e);
                ApiError::internal_server_error("Failed to resolve thread message")
            })?
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
    .bind(user.id)
    .bind(&req.content)
    .bind(&message_type)
    .bind(&req.role)
    .bind(&req.model)
    .bind(thread_db_id)
    .bind(reply_to_db_id)
    .bind(&now)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create message: {}", e);
        ApiError::internal_server_error("Failed to create message")
    })?
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
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch created message: {}", e);
        ApiError::internal_server_error("Failed to fetch created message")
    })?
    .ok_or_else(|| ApiError::internal_server_error("Failed to fetch created message"))?;

    let member_ids = fetch_chat_member_ids(&state, chat_db_id).await?;
    let event = ServerEvent::Message {
        chat_id: chat_id.clone(),
        message_id: message.public_id.clone(),
        user_id: message.user_id,
        content: message.content.clone(),
        model: message.model.clone(),
        timestamp: message.created_at.clone(),
        message_type: message.message_type.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(MessageResponse { message }))
}

// Update a message (with audit trail)
#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}/messages/{message_id}",
    tag = "Messages",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated", body = MessageResponse),
        (status = 400, description = "Invalid update payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update message", body = crate::error::ErrorResponse)
    )
)]
pub async fn update_message(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    // Get the original message
    let original_message: Option<(i64, String)> =
        sqlx::query_as("SELECT id, content FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(&message_public_id)
            .bind(chat_db_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch original message: {}", e);
                ApiError::internal_server_error("Failed to fetch original message")
            })?;

    let (message_db_id, original_content) =
        original_message.ok_or_else(|| ApiError::not_found("Message not found"))?;

    // Check if user can edit this message (owner or admin)
    let can_edit: bool = if user.id
        == sqlx::query_scalar::<_, i64>("SELECT user_id FROM messages WHERE id = ?")
            .bind(message_db_id)
            .fetch_one(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to check message ownership: {}", e);
                ApiError::internal_server_error("Failed to check message ownership")
            })? {
        true
    } else {
        // Check if user is admin or owner of the chat
        let user_role: Option<String> = sqlx::query_scalar(
            "SELECT cm.role FROM chat_members cm WHERE cm.chat_id = ? AND cm.user_id = ?",
        )
        .bind(chat_db_id)
        .bind(user.id)
        .fetch_optional(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to check user role: {}", e);
            ApiError::internal_server_error("Failed to check user role")
        })?;

        matches!(user_role.as_deref(), Some("admin") | Some("owner"))
    };

    if !can_edit {
        return Err(ApiError::forbidden("Cannot edit this message"));
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
    .bind(user.id)
    .bind(&original_content)
    .bind(&req.content)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create message edit audit: {}", e);
        ApiError::internal_server_error("Failed to create message edit audit")
    })?;

    // Update the message
    sqlx::query(
        r#"
        UPDATE messages
        SET content = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&req.content)
    .bind(&now)
    .bind(message_db_id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update message: {}", e);
        ApiError::internal_server_error("Failed to update message")
    })?;

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
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated message: {}", e);
        ApiError::internal_server_error("Failed to fetch updated message")
    })?
    .ok_or_else(|| ApiError::internal_server_error("Failed to fetch updated message"))?;

    let member_ids = fetch_chat_member_ids(&state, chat_db_id).await?;
    let event = ServerEvent::MessageUpdated {
        chat_id: chat_id.clone(),
        message: message.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(MessageResponse { message }))
}

// Delete a message (with audit trail)
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/messages/{message_id}",
    tag = "Messages",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier")
    ),
    responses(
        (status = 200, description = "Message deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete message", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_message(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    // Get the message details
    let message_details: Option<(i64, i64)> =
        sqlx::query_as("SELECT id, user_id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(&message_public_id)
            .bind(chat_db_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch message details: {}", e);
                ApiError::internal_server_error("Failed to fetch message details")
            })?;

    let (message_db_id, message_user_id) =
        message_details.ok_or_else(|| ApiError::not_found("Message not found"))?;

    // Check if user can delete this message
    let can_delete = if user.id == message_user_id {
        true
    } else {
        // Check if user is admin or owner of the chat
        let user_role: Option<String> = sqlx::query_scalar(
            "SELECT cm.role FROM chat_members cm WHERE cm.chat_id = ? AND cm.user_id = ?",
        )
        .bind(chat_db_id)
        .bind(user.id)
        .fetch_optional(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to check user role: {}", e);
            ApiError::internal_server_error("Failed to check user role")
        })?;

        matches!(user_role.as_deref(), Some("admin") | Some("owner"))
    };

    if !can_delete {
        return Err(ApiError::forbidden("Cannot delete this message"));
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
    .bind(user.id)
    .bind("User deleted message")
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create message deletion audit: {}", e);
        ApiError::internal_server_error("Failed to create message deletion audit")
    })?;

    let member_ids = fetch_chat_member_ids(&state, chat_db_id).await?;

    // Delete the message (cascade will handle related records)
    sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(message_db_id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete message: {}", e);
            ApiError::internal_server_error("Failed to delete message")
        })?;

    let event = ServerEvent::MessageDeleted {
        chat_id: chat_id.clone(),
        message_id: message_public_id.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}

// Get message edit history
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages/{message_id}/edits",
    tag = "Messages",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier")
    ),
    responses(
        (status = 200, description = "Message edit history", body = MessageEditsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch message edits", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_message_edits(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<MessageEditsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    // Get the message ID
    let message_db_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM messages WHERE public_id = ? AND chat_id = ?")
            .bind(&message_public_id)
            .bind(chat_db_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to get message ID: {}", e);
                ApiError::internal_server_error("Failed to get message ID")
            })?;

    let message_db_id = message_db_id.ok_or_else(|| ApiError::not_found("Message not found"))?;

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
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch message edits: {}", e);
        ApiError::internal_server_error("Failed to fetch message edits")
    })?;

    Ok(Json(MessageEditsResponse { edits }))
}
