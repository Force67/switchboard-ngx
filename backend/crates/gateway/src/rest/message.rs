//! Message REST endpoints

use axum::{
    extract::{Path, Query, State, Request},
    Json,
    response::IntoResponse,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::state::GatewayState;
use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: Option<String>,
    pub message_type: String,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub edited: bool,
    pub deleted: bool,
    pub sender: MessageSenderResponse,
    pub attachments: Vec<AttachmentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageSenderResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub id: String,
    pub message_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: Option<String>,
    pub message_type: Option<String>,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub before: Option<String>, // Message ID to get messages before
    pub after: Option<String>,  // Message ID to get messages after
    pub thread_id: Option<String>, // Filter by thread
}


impl From<switchboard_database::ChatMessage> for MessageResponse {
    fn from(message: switchboard_database::ChatMessage) -> Self {
        Self {
            id: message.public_id,
            chat_id: message.chat_public_id,
            sender_id: message.sender_public_id.clone(),
            content: message.content.clone(),
            message_type: message.message_type.to_string(),
            reply_to: message.reply_to_public_id.clone(),
            thread_id: message.thread_public_id.clone(),
            created_at: message.created_at.clone(),
            updated_at: message.updated_at.clone(),
            edited: message.updated_at.is_some(),
            deleted: message.deleted_at.is_some(),
            sender: MessageSenderResponse {
                id: message.sender_public_id,
                display_name: message.sender_display_name,
                avatar_url: message.sender_avatar_url,
            },
            attachments: vec![], // Will be populated by the service
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Create message routes
pub fn create_message_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/chats/:chat_id/messages", axum::routing::get(list_messages).post(create_message))
        .route("/chats/:chat_id/messages/:message_id", axum::routing::get(get_message).put(update_message).delete(delete_message))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListMessagesQuery
    ),
    responses(
        (status = 200, description = "List of messages in chat", body = Vec<MessageResponse>),
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
        .message_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let messages = state
        .message_service
        .list_by_chat(&chat_id, params.limit, params.offset, params.before.as_deref(), params.after.as_deref())
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list messages: {}", e)))?;

    let message_responses: Vec<MessageResponse> = messages.into_iter().map(|message| message.into()).collect();
    Ok(Json(message_responses))
}

#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/messages",
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
    Json(payload): Json<CreateMessageRequest>,
) -> GatewayResult<impl IntoResponse> {
    // For now, use a placeholder user_id since we can't extract it without Request
    let user_id = 1; // TODO: Fix authentication

    // Check chat membership
    state
        .message_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message_type = match payload.message_type.as_deref() {
        Some("text") => "text".to_string(),
        Some("image") => "image".to_string(),
        Some("file") => "file".to_string(),
        Some("system") => "system".to_string(),
        _ => "text".to_string(),
    };

    let create_req = switchboard_database::CreateMessageRequest {
        chat_id: 0, // Will be set by service after resolving public_id to internal ID
        chat_public_id: chat_id,
        sender_id: user_id,
        sender_public_id: user_id.to_string(),
        content: payload.content,
        message_type: switchboard_database::MessageType::Text, // Default to text for now
        reply_to_public_id: payload.reply_to,
        thread_public_id: payload.thread_id,
    };

    let message = state
        .message_service
        .create(&create_req, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create message: {}", e)))?;

    let response = MessageResponse::from(message);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages/{message_id}",
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
        .message_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(GatewayError::NotFound("Message does not belong to specified chat".to_string()));
    }

    Ok(Json(MessageResponse::from(message)))
}

#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}/messages/{message_id}",
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
    Json(payload): Json<UpdateMessageRequest>,
) -> GatewayResult<Json<MessageResponse>> {
    // For now, use a placeholder user_id since we can't extract it without Request
    let user_id = 1; // TODO: Fix authentication

    // Check chat membership
    state
        .message_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Check if user can edit (owner or admin)
    if message.sender_public_id != user_id.to_string() {
        state
            .message_service
            .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    let update_req = switchboard_database::UpdateMessageRequest {
        content: payload.content,
        status: None, // Status updates not allowed via REST API currently
    };

    let updated_message = state
        .message_service
        .update(message.id, &update_req, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update message: {}", e)))?;

    Ok(Json(MessageResponse::from(updated_message)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/messages/{message_id}",
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
        .message_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let message = state
        .message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
        .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

    // Check if user can delete (owner or admin)
    if message.sender_public_id != user_id.to_string() {
        state
            .message_service
            .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    state
        .message_service
        .delete(message.id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete message: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

