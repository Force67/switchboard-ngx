//! Chat REST endpoints

use axum::{
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::rest::models::{ChatResponse, CreateChatRequest, ListChatsQuery, UpdateChatRequest};
use crate::state::GatewayState;

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
    path = "/api/v1/chats",
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
        .chat_repo
        .find_by_user_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list chats: {}", e)))?;

    // Filter by folder if specified
    let chats = if let Some(folder_id) = params.folder_id {
        chats
            .into_iter()
            .filter(|chat| chat.folder_id.as_ref() == Some(&folder_id))
            .collect()
    } else {
        chats
    };

    let chat_responses: Vec<ChatResponse> = chats.into_iter().map(|chat| chat.into()).collect();
    Ok(Json(chat_responses))
}

#[utoipa::path(
    post,
    path = "/api/v1/chats",
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
        .chat_repo
        .create(user_id, &create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create chat: {}", e)))?;

    // Add the creator as an owner member
    let member_req = switchboard_database::CreateMemberRequest {
        chat_id: chat.id,
        user_id,
        role: switchboard_database::MemberRole::Owner,
    };
    let _ = state.member_repo.create(&member_req).await;

    let response = ChatResponse::from(chat);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}",
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
        .chat_repo
        .find_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    // Check if user is a member or creator
    if chat.created_by != user_id.to_string() {
        state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    Ok(Json(ChatResponse::from(chat)))
}

#[utoipa::path(
    put,
    path = "/api/v1/chats/{chat_id}",
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
        .chat_repo
        .update(&chat_id, user_id, &update_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update chat: {}", e)))?;

    Ok(Json(ChatResponse::from(updated_chat)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/chats/{chat_id}",
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
        .chat_repo
        .find_by_public_id(&chat_id)
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
        .permissions()
        .require_role(&chat_id, user_id, switchboard_database::MemberRole::Owner)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    state
        .chat_repo
        .delete(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete chat: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
