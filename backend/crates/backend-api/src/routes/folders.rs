use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use uuid::Uuid;

use crate::{
    routes::models::{CreateFolderRequest, Folder, UpdateFolderRequest},
    util::require_bearer,
    ApiError, AppState,
};

#[derive(Debug, Serialize)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize)]
pub struct FolderResponse {
    pub folder: Folder,
}

pub async fn list_folders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<FoldersResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let folders = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        WHERE user_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(user.id)
    .fetch_all(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch folders: {}", e);
        ApiError::internal_server_error("Failed to fetch folders")
    })?;

    Ok(Json(FoldersResponse { folders }))
}

pub async fn create_folder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<FolderResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let parent_db_id = if let Some(parent_public_id) = &req.parent_id {
        // Resolve parent folder ID from public_id
        sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ? AND user_id = ?")
            .bind(parent_public_id)
            .bind(user.id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve parent folder: {}", e);
                ApiError::internal_server_error("Failed to resolve parent folder")
            })?
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(user.id)
    .bind(&req.name)
    .bind(&req.color)
    .bind(parent_db_id)
    .bind(false)
    .bind(&now)
    .bind(&now)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to create folder: {}", e);
        ApiError::internal_server_error("Failed to create folder")
    })?;

    let folder_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
        .fetch_one(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get last insert ID: {}", e);
            ApiError::internal_server_error("Failed to create folder")
        })?;

    let folder = Folder {
        id: folder_id,
        public_id,
        user_id: user.id,
        name: req.name.clone(),
        color: req.color.clone(),
        parent_id: parent_db_id,
        collapsed: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    Ok(Json(FolderResponse { folder }))
}

pub async fn get_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<FolderResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let folder = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&folder_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch folder: {}", e);
        ApiError::internal_server_error("Failed to fetch folder")
    })?
    .ok_or_else(|| ApiError::not_found("Folder not found"))?;

    Ok(Json(FolderResponse { folder }))
}

pub async fn update_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateFolderRequest>,
) -> Result<Json<FolderResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE folders
        SET name = COALESCE(?, name),
            color = COALESCE(?, color),
            collapsed = COALESCE(?, collapsed),
            updated_at = ?
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(req.collapsed)
    .bind(&now)
    .bind(&folder_id)
    .bind(user.id)
    .execute(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to update folder: {}", e);
        ApiError::internal_server_error("Failed to update folder")
    })?;

    let folder = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&folder_id)
    .bind(user.id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated folder: {}", e);
        ApiError::internal_server_error("Failed to fetch updated folder")
    })?
    .ok_or_else(|| ApiError::not_found("Folder not found"))?;

    Ok(Json(FolderResponse { folder }))
}

pub async fn delete_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    sqlx::query("DELETE FROM folders WHERE public_id = ? AND user_id = ?")
        .bind(&folder_id)
        .bind(user.id)
        .execute(state.db_pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete folder: {}", e);
            ApiError::internal_server_error("Failed to delete folder")
        })?;

    Ok(())
}
