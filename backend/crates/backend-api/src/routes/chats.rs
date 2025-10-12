use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use uuid::Uuid;

use crate::{
    routes::models::{Chat, ChatInvite, ChatMember, CreateChatRequest, CreateInviteRequest, InviteResponse, InvitesResponse, MemberResponse, MembersResponse, UpdateChatRequest, UpdateMemberRoleRequest, ChatMessage},
    util::require_bearer,
    ApiError, AppState,
};

#[derive(Debug, Serialize)]
pub struct ChatsResponse {
    pub chats: Vec<Chat>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub chat: Chat,
}

// Helper function to fetch messages for a chat as JSON
async fn fetch_chat_messages(chat_id: i64, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Option<String>, ApiError> {
    // For now, return empty array for all chats to fix the frontend parsing issue
    // TODO: Implement proper message fetching later
    Ok(Some("[]".to_string()))
}

pub async fn list_chats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChatsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chats = sqlx::query_as::<_, Chat>(
        r#"
        SELECT c.id, c.public_id, c.user_id, c.folder_id, c.title, c.is_group, NULL as messages, c.created_at, c.updated_at
        FROM chats c
        WHERE c.user_id = ? OR (c.is_group = 1 AND EXISTS (SELECT 1 FROM chat_members cm WHERE cm.chat_id = c.id AND cm.user_id = ?))
        ORDER BY c.created_at DESC
        "#
    )
    .bind(user.id)
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
        let messages_json = fetch_chat_messages(chat.id, state.db_pool()).await?;
        let chat_with_messages = Chat {
            messages: messages_json,
            ..chat
        };
        chats_with_messages.push(chat_with_messages);
    }

    Ok(Json(ChatsResponse { chats: chats_with_messages }))
}

pub async fn create_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let folder_db_id = if let Some(folder_public_id) = &req.folder_id {
        // Resolve folder ID from public_id
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM folders WHERE public_id = ? AND user_id = ?"
        )
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

    // Create the chat first
    let chat_db_id = sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, is_group, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(user.id)
    .bind(folder_db_id)
    .bind(&req.title)
    .bind(req.is_group)
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
        sqlx::query(
            r#"
            INSERT INTO messages (public_id, chat_id, user_id, content, message_type, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message_public_id)
        .bind(chat_db_id)
        .bind(user.id)
        .bind(&message.content)
        .bind("text")
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
        "#
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
    let messages_json = if req.messages.is_empty() {
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
        user_id: user.id,
        folder_id: folder_db_id,
        title: req.title.clone(),
        is_group: req.is_group,
        messages: messages_json,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    Ok(Json(ChatResponse { chat }))
}

pub async fn get_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ChatResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chat = sqlx::query_as::<_, Chat>(
        r#"
        SELECT id, public_id, user_id, folder_id, title, is_group, NULL as messages, created_at, updated_at
        FROM chats
        WHERE public_id = ? AND user_id = ?
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

    // Add messages to the chat
    let messages_json = fetch_chat_messages(chat.id, state.db_pool()).await?;
    let chat_with_messages = Chat {
        messages: messages_json,
        ..chat
    };

    Ok(Json(ChatResponse { chat: chat_with_messages }))
}

pub async fn update_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let now = chrono::Utc::now().to_rfc3339();

    let folder_db_id = if let Some(folder_public_id) = &req.folder_id {
        if folder_public_id.is_empty() {
            None
        } else {
            // Resolve folder ID from public_id
            sqlx::query_scalar::<_, i64>(
                "SELECT id FROM folders WHERE public_id = ? AND user_id = ?"
            )
            .bind(folder_public_id)
            .bind(user.id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder: {}", e);
                ApiError::internal_server_error("Failed to resolve folder")
            })?
        }
    } else {
        None
    };

    let messages_json = req.messages.as_ref().map(|msgs| {
        serde_json::to_string(msgs).unwrap_or_else(|_| "[]".to_string())
    });

    sqlx::query(
        r#"
        UPDATE chats
        SET title = COALESCE(?, title),
            messages = COALESCE(?, messages),
            folder_id = ?,
            updated_at = ?
        WHERE public_id = ? AND user_id = ?
        "#
    )
    .bind(&req.title)
    .bind(&messages_json)
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
        SELECT id, public_id, user_id, folder_id, title, is_group, created_at, updated_at
        FROM chats
        WHERE public_id = ? AND user_id = ?
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

    Ok(Json(ChatResponse { chat }))
}

pub async fn delete_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    sqlx::query(
        "DELETE FROM chats WHERE public_id = ? AND user_id = ?"
    )
    .bind(&chat_id)
    .bind(user.id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete chat: {}", e);
        ApiError::internal_server_error("Failed to delete chat")
    })?;

    Ok(())
}

pub async fn create_invite(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Check if chat exists and is a group chat, and user is a member
    let chat_db_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT c.id FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND c.is_group = 1 AND cm.user_id = ?
        "#
    )
    .bind(&chat_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to check chat: {}", e);
        ApiError::internal_server_error("Failed to check chat")
    })?;

    let chat_db_id = chat_db_id.ok_or_else(|| ApiError::not_found("Chat not found or not a group chat"))?;

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
        "#
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
        "#
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

pub async fn accept_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Get the invite and check if the email matches
    let invite: Option<(i64, String)> = sqlx::query_as(
        "SELECT chat_id, invitee_email FROM chat_invites WHERE id = ? AND status = 'pending'"
    )
    .bind(invite_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch invite: {}", e);
        ApiError::internal_server_error("Failed to fetch invite")
    })?;

    let (chat_db_id, invitee_email) = invite.ok_or_else(|| ApiError::not_found("Invite not found"))?;

    // Check if the user's email matches
    if user.email.as_ref() != Some(&invitee_email) {
        return Err(ApiError::forbidden("Invite not for this user"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Update invite status
    sqlx::query(
        "UPDATE chat_invites SET status = 'accepted', updated_at = ? WHERE id = ?"
    )
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
        "#
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

pub async fn reject_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // Get the invite and check if the email matches
    let invitee_email: Option<String> = sqlx::query_scalar(
        "SELECT invitee_email FROM chat_invites WHERE id = ? AND status = 'pending'"
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
    sqlx::query(
        "UPDATE chat_invites SET status = 'rejected', updated_at = ? WHERE id = ?"
    )
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
        "#
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
        "#
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
        "#
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
            "#
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

    let now = chrono::Utc::now().to_rfc3339();

    // Update the role
    sqlx::query(
        r#"
        UPDATE chat_members
        SET role = ?
        WHERE chat_id = (SELECT id FROM chats WHERE public_id = ?) AND user_id = ?
        "#
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
        "#
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
        "#
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
            "#
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
        "#
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