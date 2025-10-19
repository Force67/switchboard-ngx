//! Message API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use switchboard_database::{
    ChatMessage, CreateMessageRequest, UpdateMessageRequest, MessageService, RepositoryError,
    MessageError, MessageResult, MessageType, AttachmentType,
};

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

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageEditResponse {
    pub id: String,
    pub message_id: String,
    pub old_content: String,
    pub new_content: String,
    pub edited_by: String,
    pub edited_at: String,
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

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub before: Option<String>, // Message ID to get messages before
    pub after: Option<String>,  // Message ID to get messages after
    pub thread_id: Option<String>, // Filter by thread
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetMessageEditsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<ChatMessage> for MessageResponse {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.public_id,
            chat_id: message.chat_public_id,
            sender_id: message.sender_public_id,
            content: message.content,
            message_type: message.message_type.to_string(),
            reply_to: message.reply_to_public_id,
            thread_id: message.thread_public_id,
            created_at: message.created_at.to_rfc3339(),
            updated_at: message.updated_at.map(|dt| dt.to_rfc3339()),
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_messages(
    Path(chat_id): Path<String>,
    Query(params): Query<ListMessagesQuery>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    user_id: String,
) -> Result<Json<Vec<MessageResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let messages = message_service
        .list_by_chat(&chat_id, params.limit, params.offset, params.before.as_deref(), params.after.as_deref())
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

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
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_message(
    Path(chat_id): Path<String>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    Json(payload): Json<CreateMessageRequest>,
    user_id: String,
) -> Result<Json<MessageResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let message_type = match payload.message_type.as_deref() {
        Some("text") => MessageType::Text,
        Some("image") => MessageType::Image,
        Some("file") => MessageType::File,
        Some("system") => MessageType::System,
        _ => MessageType::Text,
    };

    let create_req = switchboard_database::CreateMessageRequest {
        chat_public_id: chat_id,
        sender_public_id: user_id,
        content: payload.content,
        message_type,
        reply_to_public_id: payload.reply_to,
        thread_public_id: payload.thread_id,
    };

    let message = message_service
        .create(&create_req, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(MessageResponse::from(message)))
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    user_id: String,
) -> Result<Json<MessageResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let message = message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MESSAGE_NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
        })?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(ErrorResponse {
            error: "MESSAGE_NOT_IN_CHAT".to_string(),
            message: "Message does not belong to specified chat".to_string(),
        });
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
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn update_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    Json(payload): Json<UpdateMessageRequest>,
    user_id: String,
) -> Result<Json<MessageResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let message = message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MESSAGE_NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
        })?;

    // Check if user can edit (owner or admin)
    if message.sender_public_id != user_id {
        message_service
            .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    let update_req = switchboard_database::UpdateMessageRequest {
        content: payload.content,
    };

    let updated_message = message_service
        .update(message.id, &update_req, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_message(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let message = message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MESSAGE_NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
        })?;

    // Check if user can delete (owner or admin)
    if message.sender_public_id != user_id {
        message_service
            .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    message_service
        .delete(message.id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages/{message_id}/edits",
    tag = "Messages",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID"),
        GetMessageEditsQuery
    ),
    responses(
        (status = 200, description = "Message edit history", body = Vec<MessageEditResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_message_edits(
    Path((chat_id, message_id)): Path<(String, String)>,
    Query(params): Query<GetMessageEditsQuery>,
    State(message_service): State<MessageService<switchboard_database::MessageRepository>>,
    user_id: String,
) -> Result<Json<Vec<MessageEditResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    message_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let message = message_service
        .get_by_public_id(&message_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MESSAGE_NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
        })?;

    // Verify message belongs to the specified chat
    if message.chat_public_id != chat_id {
        return Err(ErrorResponse {
            error: "MESSAGE_NOT_IN_CHAT".to_string(),
            message: "Message does not belong to specified chat".to_string(),
        });
    }

    let edits = message_service
        .get_message_edits(message.id, params.limit, params.offset)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let edit_responses: Vec<MessageEditResponse> = edits.into_iter().map(|edit| MessageEditResponse {
        id: edit.public_id,
        message_id: message_id,
        old_content: edit.old_content,
        new_content: edit.new_content,
        edited_by: edit.edited_by_public_id,
        edited_at: edit.edited_at.to_rfc3339(),
    }).collect();

    Ok(Json(edit_responses))
}

impl From<&MessageError> for ErrorResponse {
    fn from(error: &MessageError) -> Self {
        match error {
            MessageError::NotFound => Self {
                error: "MESSAGE_NOT_FOUND".to_string(),
                message: "Message not found".to_string(),
            },
            MessageError::AccessDenied => Self {
                error: "ACCESS_DENIED".to_string(),
                message: "Access denied".to_string(),
            },
            MessageError::InvalidInput(msg) => Self {
                error: "INVALID_INPUT".to_string(),
                message: format!("Invalid input: {}", msg),
            },
            MessageError::RepositoryError(_) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: "Internal server error".to_string(),
            },
            MessageError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}