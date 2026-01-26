//! Message REST endpoints

use axum::{
    extract::{Extension, Path, Query, Request, State},
    response::IntoResponse,
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::rest::models::{
    CreateMessageRequest, ListMessagesQuery, MessageResponse, UpdateMessageRequest,
};
use crate::state::GatewayState;

/// Create message routes
pub fn create_message_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route(
            "/chats/:chat_id/messages",
            axum::routing::get(list_messages).post(create_message),
        )
        .route(
            "/chats/:chat_id/messages/:message_id",
            axum::routing::get(get_message)
                .put(update_message)
                .delete(delete_message),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/messages",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListMessagesQuery
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_messages(
    Path(chat_id): Path<String>,
    Query(params): Query<ListMessagesQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<MessageResponse>>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Get chat to resolve numeric ID
    let chat = state
        .chat_repo
        .find_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    let messages = state
        .message_repo
        .find_by_chat_id(chat.id, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list messages: {}", e)))?;

    let message_responses: Vec<MessageResponse> =
        messages.into_iter().map(|msg| msg.into()).collect();
    Ok(Json(message_responses))
}

#[utoipa::path(
    post,
    path = "/api/v1/chats/{chat_id}/messages",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "Message created successfully", body = MessageResponse),
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn create_message(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateMessageRequest>,
) -> GatewayResult<impl IntoResponse> {
    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message_type = match payload.message_type.as_deref() {
        Some("system") => switchboard_database::MessageType::System,
        _ => switchboard_database::MessageType::Text,
    };

    // Get user's public_id
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    // Get chat to resolve IDs
    let chat = state
        .chat_repo
        .find_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    let create_req = switchboard_database::CreateMessageRequest {
        chat_id: chat.id,
        chat_public_id: chat_id.clone(),
        sender_id: user_id,
        sender_public_id: user.public_id,
        content: payload.content.clone(),
        message_type,
        reply_to_public_id: payload.reply_to.clone(),
        thread_public_id: payload.thread_id.clone(),
    };

    let message = state
        .message_repo
        .create(user_id, &create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create message: {}", e)))?;

    let response = MessageResponse::from(message);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/messages/{message_id}",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    responses(
        (status = 200, description = "Message details", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Message not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn get_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<MessageResponse>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_repo
        .find_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(GatewayError::NotFound(
            "Message does not belong to specified chat".to_string(),
        ));
    }

    Ok(Json(MessageResponse::from(message)))
}

#[utoipa::path(
    put,
    path = "/api/v1/chats/{chat_id}/messages/{message_id}",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    request_body = UpdateMessageRequest,
    responses(
        (status = 200, description = "Message updated successfully", body = MessageResponse),
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Message not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn update_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateMessageRequest>,
) -> GatewayResult<Json<MessageResponse>> {
    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_repo
        .find_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(GatewayError::NotFound(
            "Message does not belong to specified chat".to_string(),
        ));
    }

    let update_req = switchboard_database::UpdateMessageRequest {
        content: payload.content.clone(),
        status: None,
    };

    // The repository already checks if user is the sender
    let updated_message = state
        .message_repo
        .update(&message_id, user_id, &update_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update message: {}", e)))?;

    Ok(Json(MessageResponse::from(updated_message)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/chats/{chat_id}/messages/{message_id}",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    responses(
        (status = 204, description = "Message deleted successfully"),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Message not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn delete_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_repo
        .find_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(GatewayError::NotFound(
            "Message does not belong to specified chat".to_string(),
        ));
    }

    // User can delete their own message, or chat admins can delete any message
    if message.sender_id != user_id {
        state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|_| {
                GatewayError::AuthorizationFailed(
                    "Cannot delete another user's message without admin permissions".to_string(),
                )
            })?;
    }

    state
        .message_repo
        .delete(&message_id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete message: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
