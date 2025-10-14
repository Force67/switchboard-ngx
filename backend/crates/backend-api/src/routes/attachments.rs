use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use crate::{
    routes::models::{
        AttachmentResponse, AttachmentsResponse, CreateAttachmentRequest, MessageAttachment,
    },
    util::require_bearer,
    ApiError, AppState,
};

// Get attachments for a message
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier")
    ),
    responses(
        (status = 200, description = "List message attachments", body = AttachmentsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch attachments", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_message_attachments(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<AttachmentsResponse>, ApiError> {
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

    // Get attachments
    let attachments = sqlx::query_as::<_, MessageAttachment>(
        r#"
        SELECT id, message_id, file_name, file_type, file_url, file_size_bytes, created_at
        FROM message_attachments
        WHERE message_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(message_db_id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch attachments: {}", e);
        ApiError::internal_server_error("Failed to fetch attachments")
    })?;

    Ok(Json(AttachmentsResponse { attachments }))
}

// Create attachment for a message
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier")
    ),
    request_body = CreateAttachmentRequest,
    responses(
        (status = 200, description = "Attachment created", body = AttachmentResponse),
        (status = 400, description = "Invalid attachment payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Message not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create attachment", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_message_attachment(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<CreateAttachmentRequest>,
) -> Result<Json<AttachmentResponse>, ApiError> {
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
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create attachment: {}", e);
        ApiError::internal_server_error("Failed to create attachment")
    })?
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
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch created attachment: {}", e);
        ApiError::internal_server_error("Failed to fetch created attachment")
    })?
    .ok_or_else(|| ApiError::internal_server_error("Failed to fetch created attachment"))?;

    Ok(Json(AttachmentResponse { attachment }))
}

// Delete an attachment
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/messages/{message_id}/attachments/{attachment_id}",
    tag = "Attachments",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("message_id" = String, Path, description = "Message public identifier"),
        ("attachment_id" = i64, Path, description = "Attachment identifier")
    ),
    responses(
        (status = 200, description = "Attachment deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Attachment not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete attachment", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_attachment(
    State(state): State<AppState>,
    Path((chat_id, message_public_id, attachment_id)): Path<(String, String, i64)>,
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

    // Get the message ID and verify attachment belongs to this message/chat
    let message_details: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT ma.message_id
        FROM message_attachments ma
        JOIN messages m ON ma.id = ? AND ma.message_id = m.id
        WHERE m.public_id = ? AND m.chat_id = ?
        "#,
    )
    .bind(attachment_id)
    .bind(&message_public_id)
    .bind(chat_db_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to verify attachment ownership: {}", e);
        ApiError::internal_server_error("Failed to verify attachment ownership")
    })?;

    message_details.ok_or_else(|| ApiError::not_found("Attachment not found"))?;

    // Check if user can delete attachments from this message
    let can_delete = {
        // Check if user owns the message
        let message_user_id: Option<i64> =
            sqlx::query_scalar("SELECT user_id FROM messages WHERE public_id = ? AND chat_id = ?")
                .bind(&message_public_id)
                .bind(chat_db_id)
                .fetch_optional(state.db_pool())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check message ownership: {}", e);
                    ApiError::internal_server_error("Failed to check message ownership")
                })?;

        if message_user_id == Some(user.id) {
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
        }
    };

    if !can_delete {
        return Err(ApiError::forbidden("Cannot delete this attachment"));
    }

    // Delete the attachment
    sqlx::query("DELETE FROM message_attachments WHERE id = ?")
        .bind(attachment_id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete attachment: {}", e);
            ApiError::internal_server_error("Failed to delete attachment")
        })?;

    Ok(())
}
