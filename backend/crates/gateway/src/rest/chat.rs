//! Chat REST endpoints

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

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListChatsQuery {
    pub folder_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<switchboard_database::Chat> for ChatResponse {
    fn from(chat: switchboard_database::Chat) -> Self {
        Self {
            id: chat.public_id,
            title: chat.title,
            description: chat.description,
            avatar_url: chat.avatar_url,
            folder_id: chat.folder_id,
            created_by: chat.created_by,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
            member_count: chat.member_count,
            message_count: chat.message_count,
            last_message_at: chat.last_message_at,
            members: vec![], // Will be populated by the service
            messages: vec![], // Will be populated by the service
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Create chat routes
pub fn create_chat_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/chats", axum::routing::get(list_chats).post(create_chat))
        .route("/chats/:chat_id", axum::routing::get(get_chat).put(update_chat).delete(delete_chat))
}

#[utoipa::path(
    get,
    path = "/api/chats",
    tag = "Chats",
    params(ListChatsQuery),
    responses(
        (status = 200, description = "List of user's chats", body = Vec<ChatResponse>),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_chats(
    Query(params): Query<ListChatsQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<ChatResponse>>> {
    let user_id = extract_user_id(&request)?;

    let chats = state
        .chat_service
        .list_user_chats(user_id, params.folder_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list chats: {}", e)))?;

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
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
#[axum::debug_handler]
pub async fn create_chat(
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<CreateChatRequest>,
) -> GatewayResult<impl IntoResponse> {
    // For now, use a placeholder user_id since we can't extract it without Request
    let user_id = 1; // TODO: Fix authentication

    let create_req = switchboard_database::CreateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
        chat_type: switchboard_database::ChatType::Group, // Default to group chat
        created_by: user_id.to_string(),
    };

    let chat = state
        .chat_service
        .create(&create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create chat: {}", e)))?;

    let response = ChatResponse::from(chat);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn get_chat(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<ChatResponse>> {
    let user_id = extract_user_id(&request)?;

    let chat = state
        .chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    // Check if user is a member
    state
        .chat_service
        .check_membership(chat.id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

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
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn update_chat(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<UpdateChatRequest>,
) -> GatewayResult<Json<ChatResponse>> {
    // For now, use a placeholder user_id since we can't extract it without Request
    let user_id = 1; // TODO: Fix authentication

    let chat = state
        .chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    // Check if user is owner or admin
    state
        .chat_service
        .check_role(chat.id, user_id, switchboard_database::MemberRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let update_req = switchboard_database::UpdateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
        status: None, // Status updates not allowed via REST API currently
    };

    let updated_chat = state
        .chat_service
        .update(chat.id, &update_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update chat: {}", e)))?;

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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn delete_chat(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    let chat = state
        .chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    // Check if user is owner
    state
        .chat_service
        .check_role(chat.id, user_id, switchboard_database::MemberRole::Owner)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    state
        .chat_service
        .delete(chat.id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete chat: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}