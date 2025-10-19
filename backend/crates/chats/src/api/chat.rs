//! Chat API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use switchboard_database::{
    Chat, CreateChatRequest, UpdateChatRequest, ChatService, RepositoryError,
    ChatError, ChatResult,
};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: i64,
    pub message_count: i64,
    pub last_message_at: Option<String>,
    pub members: Vec<ChatMemberResponse>,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatMemberResponse {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

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
    pub sender: ChatMemberResponse,
    pub attachments: Vec<AttachmentResponse>,
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
pub struct CreateChatRequest {
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
    pub initial_message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListChatsQuery {
    pub folder_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<Chat> for ChatResponse {
    fn from(chat: Chat) -> Self {
        Self {
            id: chat.public_id,
            title: chat.title,
            description: chat.description,
            avatar_url: chat.avatar_url,
            folder_id: chat.folder_id,
            created_by: chat.created_by,
            created_at: chat.created_at.to_rfc3339(),
            updated_at: chat.updated_at.to_rfc3339(),
            member_count: chat.member_count,
            message_count: chat.message_count,
            last_message_at: chat.last_message_at.map(|dt| dt.to_rfc3339()),
            members: vec![], // Will be populated by the service
            messages: vec![], // Will be populated by the service
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/chats",
    tag = "Chats",
    params(ListChatsQuery),
    responses(
        (status = 200, description = "List of user's chats", body = Vec<ChatResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_chats(
    Query(params): Query<ListChatsQuery>,
    State(chat_service): State<ChatService<switchboard_database::ChatRepository>>,
    user_id: String,
) -> Result<Json<Vec<ChatResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let chats = chat_service
        .list_user_chats(user_internal_id, params.folder_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let chat_responses: Vec<ChatResponse> = chats.into_iter().map(|chat| chat.into()).collect();
    Ok(Json(chat_responses))
}

#[utoipa::path(
    post,
    path = "/api/chats",
    tag = "Chats",
    request_body = CreateChatRequest,
    responses(
        (status = 201, description = "Chat created successfully", body = ChatResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_chat(
    State(chat_service): State<ChatService<switchboard_database::ChatRepository>>,
    Json(payload): Json<CreateChatRequest>,
    user_id: String,
) -> Result<Json<ChatResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let create_req = switchboard_database::CreateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
        created_by: user_internal_id,
    };

    let chat = chat_service
        .create(&create_req)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(ChatResponse::from(chat)))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    responses(
        (status = 200, description = "Chat details", body = ChatResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_chat(
    Path(chat_id): Path<String>,
    State(chat_service): State<ChatService<switchboard_database::ChatRepository>>,
    user_id: String,
) -> Result<Json<ChatResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let chat = chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "CHAT_NOT_FOUND".to_string(),
            message: "Chat not found".to_string(),
        })?;

    // Check if user is a member
    chat_service
        .check_membership(chat.id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(ChatResponse::from(chat)))
}

#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    request_body = UpdateChatRequest,
    responses(
        (status = 200, description = "Chat updated successfully", body = ChatResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn update_chat(
    Path(chat_id): Path<String>,
    State(chat_service): State<ChatService<switchboard_database::ChatRepository>>,
    Json(payload): Json<UpdateChatRequest>,
    user_id: String,
) -> Result<Json<ChatResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let chat = chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "CHAT_NOT_FOUND".to_string(),
            message: "Chat not found".to_string(),
        })?;

    // Check if user is owner or admin
    chat_service
        .check_role(chat.id, user_internal_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let update_req = switchboard_database::UpdateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
    };

    let updated_chat = chat_service
        .update(chat.id, &update_req)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(ChatResponse::from(updated_chat)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    responses(
        (status = 204, description = "Chat deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_chat(
    Path(chat_id): Path<String>,
    State(chat_service): State<ChatService<switchboard_database::ChatRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let chat = chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "CHAT_NOT_FOUND".to_string(),
            message: "Chat not found".to_string(),
        })?;

    // Check if user is owner
    chat_service
        .check_role(chat.id, user_internal_id, switchboard_database::ChatRole::Owner)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    chat_service
        .delete(chat.id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

impl From<&ChatError> for ErrorResponse {
    fn from(error: &ChatError) -> Self {
        match error {
            ChatError::NotFound => Self {
                error: "CHAT_NOT_FOUND".to_string(),
                message: "Chat not found".to_string(),
            },
            ChatError::AccessDenied => Self {
                error: "ACCESS_DENIED".to_string(),
                message: "Access denied".to_string(),
            },
            ChatError::InvalidInput(msg) => Self {
                error: "INVALID_INPUT".to_string(),
                message: format!("Invalid input: {}", msg),
            },
            ChatError::RepositoryError(_) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: "Internal server error".to_string(),
            },
            ChatError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}

impl From<RepositoryError> for ErrorResponse {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound => Self {
                error: "NOT_FOUND".to_string(),
                message: "Resource not found".to_string(),
            },
            RepositoryError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
            RepositoryError::ValidationError(msg) => Self {
                error: "VALIDATION_ERROR".to_string(),
                message: format!("Validation error: {}", msg),
            },
            RepositoryError::DuplicateError(msg) => Self {
                error: "DUPLICATE_ERROR".to_string(),
                message: format!("Duplicate error: {}", msg),
            },
        }
    }
}