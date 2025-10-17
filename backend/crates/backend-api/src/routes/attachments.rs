use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use crate::{
    routes::models::{
        AttachmentResponse, AttachmentsResponse, CreateAttachmentRequest,
    },
    services::attachment as attachment_service,
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

    let attachments = attachment_service::get_message_attachments(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch attachments: {}", e);
        ApiError::from(e)
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

    let attachment = attachment_service::create_message_attachment(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id,
        req
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create attachment: {}", e);
        ApiError::from(e)
    })?;

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

    attachment_service::delete_attachment(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        attachment_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete attachment: {}", e);
        ApiError::from(e)
    })?;

    Ok(())
}
