use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;

use uuid::Uuid;

use crate::{
    routes::models::{
        Chat, ChatInvite, ChatMember, CreateChatRequest, CreateInviteRequest, InviteResponse,
        InvitesResponse, MemberResponse, MembersResponse, UpdateChatRequest,
        UpdateMemberRoleRequest,
    },
    util::require_bearer,
    ApiError, AppState,
};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatsResponse {
    pub chats: Vec<Chat>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatDetailResponse {
    pub chat: Chat,
}

// Helper function to fetch messages for a chat as JSON
async fn fetch_chat_messages(
    chat_id: i64,
    pool: &sqlx::Pool<sqlx::Sqlite>,
) -> Result<Option<String>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT public_id, user_id, content, role, model, message_type, created_at
        FROM messages
        WHERE chat_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch messages for chat {}: {}", chat_id, e);
        ApiError::internal_server_error("Failed to fetch chat messages")
    })?;

    if rows.is_empty() {
        return Ok(Some("[]".to_string()));
    }

    let mut messages = Vec::with_capacity(rows.len());
    for row in rows {
        let role: String = row.get("role");
        let content: String = row.get("content");
        let model: Option<String> = row.try_get("model").unwrap_or(None);
        let message_json = json!({
            "id": row.get::<String, _>("public_id"),
            "user_id": row.get::<i64, _>("user_id"),
            "role": role,
            "content": content,
            "model": model,
            "timestamp": row.get::<String, _>("created_at"),
            "message_type": row.get::<String, _>("message_type"),
        });
        messages.push(message_json);
    }

    serde_json::to_string(&messages).map(Some).map_err(|e| {
        tracing::error!("Failed to serialize messages for chat {}: {}", chat_id, e);
        ApiError::internal_server_error("Failed to serialize chat messages")
    })
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

    let chats = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        WHERE c.id IN (
            SELECT chat_id FROM chat_members WHERE user_id = ?
        )
        ORDER BY c.updated_at DESC
        "#
    )
    .bind(user.id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch chats: {}", e);
        ApiError::internal_server_error("Failed to fetch chats")
    })?;

    // Add messages to each chat
    let mut chats_with_messages = Vec::new();
    for chat in chats {
        let _messages_json = fetch_chat_messages(chat.id, state.db_pool()).await?;
        let chat_with_messages = Chat { ..chat };
        chats_with_messages.push(chat_with_messages);
    }

    Ok(Json(ChatsResponse {
        chats: chats_with_messages,
    }))
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

    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let folder_db_id = if let Some(folder_public_id) = &req.folder_id {
        // Resolve folder ID from public_id
        sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ? AND user_id = ?")
            .bind(folder_public_id)
            .bind(user.id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder: {}", e);
                ApiError::internal_server_error("Failed to resolve folder")
            })?
    } else {
        None
    };

    // Create the chat first (user_id is now nullable, managed through chat_members)
    let chat_db_id = sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, chat_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&public_id)
    .bind(user.id) // Set user_id for backwards compatibility
    .bind(folder_db_id)
    .bind(&req.title)
    .bind(&req.chat_type)
    .bind(&now)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create chat: {}", e);
        ApiError::internal_server_error("Failed to create chat")
    })?
    .last_insert_rowid();

    // Insert initial messages if provided
    for message in &req.messages {
        let message_public_id = Uuid::new_v4().to_string();
        let message_type = if message.role.eq_ignore_ascii_case("system") {
            "system"
        } else {
            "text"
        };
        sqlx::query(
            r#"
            INSERT INTO messages (public_id, chat_id, user_id, content, message_type, role, model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message_public_id)
        .bind(chat_db_id)
        .bind(user.id)
        .bind(&message.content)
        .bind(message_type)
        .bind(message.role.as_str())
        .bind(message.model.clone())
        .bind(&now)
        .bind(&now)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create initial message: {}", e);
            ApiError::internal_server_error("Failed to create initial message")
        })?;
    }

    // Add creator as owner of the chat (for both regular and group chats)
    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(chat_db_id)
    .bind(user.id)
    .bind("owner")
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to add chat owner: {}", e);
        ApiError::internal_server_error("Failed to add chat owner")
    })?;

    // Get messages for the newly created chat
    let _messages_json = if req.messages.is_empty() {
        Some("[]".to_string())
    } else {
        // Return the initial messages as JSON
        serde_json::to_string(&req.messages)
            .map(|s| Some(s))
            .unwrap_or_else(|_| Some("[]".to_string()))
    };

    let chat = Chat {
        id: chat_db_id,
        public_id: public_id.clone(),
        user_id: Some(user.id),
        folder_id: folder_db_id,
        title: req.title.clone(),
        chat_type: req.chat_type.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

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

    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch chat: {}", e);
        ApiError::internal_server_error("Failed to fetch chat")
    })?
    .ok_or_else(|| ApiError::not_found("Chat not found"))?;

    let chat_with_messages = Chat { ..chat };

    Ok(Json(ChatDetailResponse {
        chat: chat_with_messages,
    }))
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

    let now = chrono::Utc::now().to_rfc3339();

    let mut folder_update_requested = false;
    let mut folder_set_null = false;
    let mut folder_db_id: Option<i64> = None;

    if let Some(folder_public_id) = &req.folder_id {
        folder_update_requested = true;
        if folder_public_id.is_empty() {
            folder_set_null = true;
        } else {
            folder_db_id = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM folders WHERE public_id = ? AND user_id = ?",
            )
            .bind(folder_public_id)
            .bind(user.id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder: {}", e);
                ApiError::internal_server_error("Failed to resolve folder")
            })?;

            if folder_db_id.is_none() {
                return Err(ApiError::not_found("Folder not found"));
            }
        }
    }

    let update_folder_flag: i32 = if folder_update_requested { 1 } else { 0 };
    let set_folder_null_flag: i32 = if folder_set_null { 1 } else { 0 };

    sqlx::query(
        r#"
        UPDATE chats
        SET title = COALESCE(?, title),
            folder_id = CASE
                WHEN ? = 0 THEN folder_id
                WHEN ? = 1 THEN NULL
                ELSE ?
            END,
            updated_at = ?
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&req.title)
    .bind(update_folder_flag)
    .bind(set_folder_null_flag)
    .bind(folder_db_id)
    .bind(&now)
    .bind(&chat_id)
    .bind(user.id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update chat: {}", e);
        ApiError::internal_server_error("Failed to update chat")
    })?;

    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.chat_type, c.created_at, c.updated_at
        FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated chat: {}", e);
        ApiError::internal_server_error("Failed to fetch updated chat")
    })?
    .ok_or_else(|| ApiError::not_found("Chat not found"))?;

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

    // Check if user is owner of the chat before deleting
    let chat_role: Option<String> = sqlx::query_scalar(
        r#"
        SELECT cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat ownership: {}", e);
        ApiError::internal_server_error("Failed to delete chat")
    })?;

    let chat_role = chat_role.ok_or_else(|| ApiError::not_found("Chat not found"))?;

    if chat_role != "owner" {
        return Err(ApiError::forbidden("Only chat owners can delete chats"));
    }

    sqlx::query("DELETE FROM chats WHERE public_id = ?")
        .bind(&chat_id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete chat: {}", e);
            ApiError::internal_server_error("Failed to delete chat")
        })?;

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

    // Check if chat exists and is a group chat, and user is a member
    let result: Option<(i64, String)> = sqlx::query_as(
        r#"
        SELECT c.id, cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND c.chat_type = 'group' AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat: {}", e);
        ApiError::internal_server_error("Failed to check chat")
    })?;

    let (chat_db_id, user_role) =
        result.ok_or_else(|| ApiError::not_found("Chat not found or not a group chat"))?;

    // Check if user has permission to invite (owner or admin)
    if user_role != "owner" && user_role != "admin" {
        return Err(ApiError::forbidden(
            "Only owners and admins can invite members",
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO chat_invites (chat_id, inviter_id, invitee_email, status, created_at, updated_at)
        VALUES (?, ?, ?, 'pending', ?, ?)
        "#
    )
    .bind(chat_db_id)
    .bind(user.id)
    .bind(&req.email)
    .bind(&now)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create invite: {}", e);
        ApiError::internal_server_error("Failed to create invite")
    })?;

    let invite_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
        .fetch_one(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get last insert ID: {}", e);
            ApiError::internal_server_error("Failed to create invite")
        })?;

    let invite = ChatInvite {
        id: invite_id,
        chat_id: chat_db_id,
        inviter_id: user.id,
        invitee_email: req.email,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

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

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    let invites = sqlx::query_as::<_, ChatInvite>(
        r#"
        SELECT id, chat_id, inviter_id, invitee_email, status, created_at, updated_at
        FROM chat_invites
        WHERE chat_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch invites: {}", e);
        ApiError::internal_server_error("Failed to fetch invites")
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

    // Get the invite and check if the email matches
    let invite: Option<(i64, String)> = sqlx::query_as(
        "SELECT chat_id, invitee_email FROM chat_invites WHERE id = ? AND status = 'pending'",
    )
    .bind(invite_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch invite: {}", e);
        ApiError::internal_server_error("Failed to fetch invite")
    })?;

    let (chat_db_id, invitee_email) =
        invite.ok_or_else(|| ApiError::not_found("Invite not found"))?;

    // Check if the user's email matches
    if user.email.as_ref() != Some(&invitee_email) {
        return Err(ApiError::forbidden("Invite not for this user"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Update invite status
    sqlx::query("UPDATE chat_invites SET status = 'accepted', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(invite_id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update invite: {}", e);
            ApiError::internal_server_error("Failed to accept invite")
        })?;

    // Add user to chat members
    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, 'member', ?)
        "#,
    )
    .bind(chat_db_id)
    .bind(user.id)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to add user to chat: {}", e);
        ApiError::internal_server_error("Failed to accept invite")
    })?;

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

    // Get the invite and check if the email matches
    let invitee_email: Option<String> = sqlx::query_scalar(
        "SELECT invitee_email FROM chat_invites WHERE id = ? AND status = 'pending'",
    )
    .bind(invite_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch invite: {}", e);
        ApiError::internal_server_error("Failed to fetch invite")
    })?;

    let invitee_email = invitee_email.ok_or_else(|| ApiError::not_found("Invite not found"))?;

    // Check if the user's email matches
    if user.email.as_ref() != Some(&invitee_email) {
        return Err(ApiError::forbidden("Invite not for this user"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Update invite status
    sqlx::query("UPDATE chat_invites SET status = 'rejected', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(invite_id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update invite: {}", e);
            ApiError::internal_server_error("Failed to reject invite")
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

    // Check if user is a member of the chat
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat membership: {}", e);
        ApiError::internal_server_error("Failed to check chat membership")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    let members = sqlx::query_as::<_, ChatMember>(
        r#"
        SELECT id, chat_id, user_id, role, joined_at
        FROM chat_members
        WHERE chat_id = ?
        ORDER BY joined_at ASC
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch members: {}", e);
        ApiError::internal_server_error("Failed to fetch members")
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

    // Check if user is an owner/admin of the chat
    let user_role: Option<String> = sqlx::query_scalar(
        r#"
        SELECT cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check user role: {}", e);
        ApiError::internal_server_error("Failed to check user role")
    })?;

    let user_role = user_role.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    if user_role != "owner" && user_role != "admin" {
        return Err(ApiError::forbidden("Insufficient permissions"));
    }

    // Validate role
    if req.role != "member" && req.role != "admin" && req.role != "owner" {
        return Err(ApiError::bad_request("Invalid role"));
    }

    // Prevent demoting the last owner
    if req.role != "owner" {
        let owner_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM chat_members cm
            JOIN chats c ON c.id = cm.chat_id
            WHERE c.public_id = ? AND cm.role = 'owner'
            "#,
        )
        .bind(&chat_id)
        .fetch_one(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to count owners: {}", e);
            ApiError::internal_server_error("Failed to validate role change")
        })?;

        if owner_count <= 1 {
            let target_role: Option<String> = sqlx::query_scalar(
                "SELECT role FROM chat_members cm JOIN chats c ON c.id = cm.chat_id WHERE c.public_id = ? AND cm.user_id = ?"
            )
            .bind(&chat_id)
            .bind(member_user_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to get target role: {}", e);
                ApiError::internal_server_error("Failed to validate role change")
            })?;

            if target_role.as_deref() == Some("owner") {
                return Err(ApiError::bad_request("Cannot remove the last owner"));
            }
        }
    }

    // Update the role
    sqlx::query(
        r#"
        UPDATE chat_members
        SET role = ?
        WHERE chat_id = (SELECT id FROM chats WHERE public_id = ?) AND user_id = ?
        "#,
    )
    .bind(&req.role)
    .bind(&chat_id)
    .bind(member_user_id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update member role: {}", e);
        ApiError::internal_server_error("Failed to update member role")
    })?;

    // Return the updated member
    let member = sqlx::query_as::<_, ChatMember>(
        r#"
        SELECT cm.id, cm.chat_id, cm.user_id, cm.role, cm.joined_at
        FROM chat_members cm
        JOIN chats c ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(member_user_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated member: {}", e);
        ApiError::internal_server_error("Failed to fetch updated member")
    })?
    .ok_or_else(|| ApiError::not_found("Member not found"))?;

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

    // Check if user is an owner/admin of the chat
    let user_role: Option<String> = sqlx::query_scalar(
        r#"
        SELECT cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check user role: {}", e);
        ApiError::internal_server_error("Failed to check user role")
    })?;

    let user_role = user_role.ok_or_else(|| ApiError::forbidden("Not a member of this chat"))?;

    if user_role != "owner" && user_role != "admin" {
        return Err(ApiError::forbidden("Insufficient permissions"));
    }

    // Prevent removing the last owner
    if user_role == "owner" || user_role == "admin" {
        let owner_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM chat_members cm
            JOIN chats c ON c.id = cm.chat_id
            WHERE c.public_id = ? AND cm.role = 'owner'
            "#,
        )
        .bind(&chat_id)
        .fetch_one(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to count owners: {}", e);
            ApiError::internal_server_error("Failed to validate removal")
        })?;

        let target_role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM chat_members cm JOIN chats c ON c.id = cm.chat_id WHERE c.public_id = ? AND cm.user_id = ?"
        )
        .bind(&chat_id)
        .bind(member_user_id)
        .fetch_optional(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get target role: {}", e);
            ApiError::internal_server_error("Failed to validate removal")
        })?;

        if target_role.as_deref() == Some("owner") && owner_count <= 1 {
            return Err(ApiError::bad_request("Cannot remove the last owner"));
        }
    }

    // Remove the member
    sqlx::query(
        r#"
        DELETE FROM chat_members
        WHERE chat_id = (SELECT id FROM chats WHERE public_id = ?) AND user_id = ?
        "#,
    )
    .bind(&chat_id)
    .bind(member_user_id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to remove member: {}", e);
        ApiError::internal_server_error("Failed to remove member")
    })?;

    Ok(())
}
