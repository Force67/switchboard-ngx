//! Chat REST endpoints

use axum::{
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use crate::error::{GatewayError, GatewayResult};
use crate::state::GatewayState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatResponse {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub messages: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: i64,
    pub message_count: i64,
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub members: Vec<ChatMemberResponse>,
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
    #[serde(default)]
    pub is_group: bool,
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
            id: chat.id,
            public_id: chat.public_id.clone(),
            user_id: chat.created_by.parse().unwrap_or(0),
            title: chat.title,
            description: chat.description,
            avatar_url: chat.avatar_url,
            folder_id: chat.folder_id,
            is_group: matches!(chat.chat_type, switchboard_database::ChatType::Group),
            messages: Some("[]".to_string()),
            created_by: chat.created_by,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
            member_count: chat.member_count,
            message_count: chat.message_count,
            last_message_at: chat.last_message_at,
            members: vec![],
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
        .route(
            "/chats/:chat_id",
            axum::routing::get(get_chat)
                .put(update_chat)
                .delete(delete_chat),
        )
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
    Extension(user_id): Extension<i64>,
) -> GatewayResult<Json<Vec<ChatResponse>>> {
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
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateChatRequest>,
) -> GatewayResult<impl IntoResponse> {
    let chat_type = if payload.is_group {
        switchboard_database::ChatType::Group
    } else {
        switchboard_database::ChatType::Direct
    };

    let create_req = switchboard_database::CreateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
        chat_type,
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
    Extension(user_id): Extension<i64>,
) -> GatewayResult<Json<ChatResponse>> {
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
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateChatRequest>,
) -> GatewayResult<Json<ChatResponse>> {
    let update_req = switchboard_database::UpdateChatRequest {
        title: payload.title,
        description: payload.description,
        avatar_url: payload.avatar_url,
        folder_id: payload.folder_id,
        status: None, // Status updates not allowed via REST API currently
    };

    let updated_chat = state
        .chat_service
        .update_chat(&chat_id, user_id, update_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update chat: {}", e)))?
        .0;

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
    Extension(user_id): Extension<i64>,
) -> GatewayResult<impl IntoResponse> {
    let chat = state
        .chat_service
        .get_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    if chat.created_by != user_id.to_string() {
        return Err(GatewayError::AuthorizationFailed(
            "Access denied: only chat creators can delete chats".to_string(),
        ));
    }

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
