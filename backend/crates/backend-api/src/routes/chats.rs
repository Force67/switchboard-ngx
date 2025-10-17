use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    routes::models::{
        Chat, ChatInvite, ChatMember, CreateChatRequest, CreateInviteRequest, InviteResponse,
        InvitesResponse, MemberResponse, MembersResponse, UpdateChatRequest,
        UpdateMemberRoleRequest,
    },
    services::{chat as chat_service, invite as invite_service, member as member_service},
    state::ServerEvent,
    util::require_bearer,
    ApiError, AppState,
};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatsResponse {
    #[schema(value_type = Vec<ChatWithMessages>)]
    pub chats: Vec<ChatWithMessages>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatDetailResponse {
    pub chat: Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatWithMessages {
    pub id: i64,
    pub public_id: String,
    #[schema(nullable)]
    pub user_id: Option<i64>,
    #[schema(nullable)]
    pub folder_id: Option<i64>,
    pub title: String,
    pub chat_type: String,
    pub created_at: String,
    pub updated_at: String,
    #[schema(default)]
    pub is_group: bool,
    #[schema(nullable)]
    pub messages: Option<String>,
}



#[utoipa::path(
    get,
    path = "/api/chats",
    tag = "Chats",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List chats for the authenticated user", body = ChatsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch chats", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_chats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChatsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chats = chat_service::list_chats(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch chats: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(ChatsResponse { chats }))
}

#[utoipa::path(
    post,
    path = "/api/chats",
    tag = "Chats",
    security(("bearerAuth" = [])),
    request_body = CreateChatRequest,
    responses(
        (status = 200, description = "Chat created", body = ChatDetailResponse),
        (status = 400, description = "Invalid chat payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create chat", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateChatRequest>,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chat = chat_service::create_chat(state.db_pool(), user.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create chat: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::ChatCreated { chat: chat.clone() };
    state.broadcast_to_user(user.id, &event).await;

    Ok(Json(ChatDetailResponse { chat }))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    responses(
        (status = 200, description = "Chat retrieved", body = ChatDetailResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch chat", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chat = chat_service::get_chat(state.db_pool(), &chat_id, user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch chat: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(ChatDetailResponse { chat }))
}

#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    request_body = UpdateChatRequest,
    responses(
        (status = 200, description = "Chat updated", body = ChatDetailResponse),
        (status = 400, description = "Invalid update payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update chat", body = crate::error::ErrorResponse)
    )
)]
pub async fn update_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateChatRequest>,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (chat, member_ids) = chat_service::update_chat(state.db_pool(), &chat_id, user.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update chat: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::ChatUpdated { chat: chat.clone() };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(ChatDetailResponse { chat }))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}",
    tag = "Chats",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    responses(
        (status = 200, description = "Chat deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete chat", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let member_ids = chat_service::delete_chat(state.db_pool(), &chat_id, user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete chat: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::ChatDeleted {
        chat_id: chat_id.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    {
        let mut broadcasters = state.chat_broadcasters.lock().await;
        broadcasters.remove(&chat_id);
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/invites",
    tag = "Chat Invites",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    request_body = CreateInviteRequest,
    responses(
        (status = 200, description = "Invite created", body = InviteResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create invite", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_invite(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (invite, member_ids) = invite_service::create_invite(
        state.db_pool(),
        &chat_id,
        user.id,
        req
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create invite: {}", e);
        ApiError::from(e)
    })?;

    let event = ServerEvent::InviteCreated {
        chat_id: chat_id.clone(),
        invite: invite.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(InviteResponse { invite }))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/invites",
    tag = "Chat Invites",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    responses(
        (status = 200, description = "List pending chat invites", body = InvitesResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch invites", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_invites(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<InvitesResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let invites = invite_service::list_invites(
        state.db_pool(),
        &chat_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch invites: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(InvitesResponse { invites }))
}

#[utoipa::path(
    post,
    path = "/api/invites/{invite_id}/accept",
    tag = "Chat Invites",
    security(("bearerAuth" = [])),
    params(
        ("invite_id" = i64, Path, description = "Invite identifier")
    ),
    responses(
        (status = 200, description = "Invite accepted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Invite not valid for user", body = crate::error::ErrorResponse),
        (status = 404, description = "Invite not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to accept invite", body = crate::error::ErrorResponse)
    )
)]
pub async fn accept_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (member, member_ids, chat_public_id) = invite_service::accept_invite(
        state.db_pool(),
        invite_id,
        user.id,
        user.email
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to accept invite: {}", e);
        ApiError::from(e)
    })?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_public_id.clone(),
        member: member.clone(),
    };
    state.broadcast_to_chat(&chat_public_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/invites/{invite_id}/reject",
    tag = "Chat Invites",
    security(("bearerAuth" = [])),
    params(
        ("invite_id" = i64, Path, description = "Invite identifier")
    ),
    responses(
        (status = 200, description = "Invite rejected"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Invite not valid for user", body = crate::error::ErrorResponse),
        (status = 404, description = "Invite not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to reject invite", body = crate::error::ErrorResponse)
    )
)]
pub async fn reject_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    invite_service::reject_invite(
        state.db_pool(),
        invite_id,
        user.email
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to reject invite: {}", e);
        ApiError::from(e)
    })?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/members",
    tag = "Chat Members",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier")
    ),
    responses(
        (status = 200, description = "List chat members", body = MembersResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Chat not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch members", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<MembersResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let members = member_service::list_members(
        state.db_pool(),
        &chat_id,
        user.id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch members: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(MembersResponse { members }))
}

#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}/members/{member_user_id}",
    tag = "Chat Members",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("member_user_id" = i64, Path, description = "Target user identifier")
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Member role updated", body = MemberResponse),
        (status = 400, description = "Invalid role update", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Member not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update member role", body = crate::error::ErrorResponse)
    )
)]
pub async fn update_member_role(
    State(state): State<AppState>,
    Path((chat_id, member_user_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<MemberResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (member, member_ids) = member_service::update_member_role(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id,
        req
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update member role: {}", e);
        ApiError::from(e)
    })?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_id.clone(),
        member: member.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(MemberResponse { member }))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/members/{member_user_id}",
    tag = "Chat Members",
    security(("bearerAuth" = [])),
    params(
        ("chat_id" = String, Path, description = "Chat public identifier"),
        ("member_user_id" = i64, Path, description = "Target user identifier")
    ),
    responses(
        (status = 200, description = "Member removed"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Member not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to remove member", body = crate::error::ErrorResponse)
    )
)]
pub async fn remove_member(
    State(state): State<AppState>,
    Path((chat_id, member_user_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let member_ids = member_service::remove_member(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to remove member: {}", e);
        ApiError::from(e)
    })?;

    let event = ServerEvent::MemberRemoved {
        chat_id: chat_id.clone(),
        user_id: member_user_id,
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}
