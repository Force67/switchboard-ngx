use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use uuid::Uuid;

use crate::{
    routes::models::{Chat, ChatMessage, CreateChatRequest, UpdateChatRequest},
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

pub async fn list_chats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChatsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chats = sqlx::query_as::<_, Chat>(
        r#"
        SELECT id, public_id, user_id, folder_id, title, messages, created_at, updated_at
        FROM chats
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#
    )
    .bind(user.id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch chats: {}", e);
        ApiError::internal_server_error("Failed to fetch chats")
    })?;

    Ok(Json(ChatsResponse { chats }))
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
    let messages_json = serde_json::to_string(&req.messages)
        .map_err(|e| {
            tracing::error!("Failed to serialize messages: {}", e);
            ApiError::bad_request("Invalid messages format")
        })?;

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

    sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, folder_id, title, messages, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(user.id)
    .bind(folder_db_id)
    .bind(&req.title)
    .bind(&messages_json)
    .bind(&now)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create chat: {}", e);
        ApiError::internal_server_error("Failed to create chat")
    })?;

    let chat_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
        .fetch_one(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get last insert ID: {}", e);
            ApiError::internal_server_error("Failed to create chat")
        })?;

    let chat = Chat {
        id: chat_id,
        public_id,
        user_id: user.id,
        folder_id: folder_db_id,
        title: req.title.clone(),
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
        SELECT id, public_id, user_id, folder_id, title, messages, created_at, updated_at
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

    Ok(Json(ChatResponse { chat }))
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
        SELECT id, public_id, user_id, folder_id, title, messages, created_at, updated_at
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