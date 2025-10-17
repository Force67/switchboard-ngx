use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use crate::{
    routes::models::{
        CreateMessageRequest, MessageEditsResponse, MessageResponse,
        MessagesResponse, UpdateMessageRequest,
    },
    services::message as message_service,
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

    let messages = message_service::get_messages(state.db_pool(), &chat_id, user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch messages: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(MessagesResponse { messages }))
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

    let (message, member_ids) = message_service::create_message(state.db_pool(), &chat_id, user.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create message: {}", e);
            ApiError::from(e)
        })?;

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

    let (message, member_ids) = message_service::update_message(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id,
        req.content
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update message: {}", e);
        ApiError::from(e)
    })?;

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

    let (member_ids, deleted_message_id) = message_service::delete_message(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete message: {}", e);
        ApiError::from(e)
    })?;

    let event = ServerEvent::MessageDeleted {
        chat_id: chat_id.clone(),
        message_id: deleted_message_id,
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

    let edits = message_service::get_message_edits(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch message edits: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(MessageEditsResponse { edits }))
}
