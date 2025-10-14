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
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FolderResponse {
    pub folder: Folder,
}

#[utoipa::path(
    get,
    path = "/api/folders",
    tag = "Folders",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List folders for the current user", body = FoldersResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch folders", body = crate::error::ErrorResponse)
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/folders",
    tag = "Folders",
    security(("bearerAuth" = [])),
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created", body = FolderResponse),
        (status = 400, description = "Invalid folder payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create folder", body = crate::error::ErrorResponse)
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/folders/{folder_id}",
    tag = "Folders",
    security(("bearerAuth" = [])),
    params(
        ("folder_id" = String, Path, description = "Folder public identifier")
    ),
    responses(
        (status = 200, description = "Folder fetched", body = FolderResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Folder not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch folder", body = crate::error::ErrorResponse)
    )
)]
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

#[utoipa::path(
    put,
    path = "/api/folders/{folder_id}",
    tag = "Folders",
    security(("bearerAuth" = [])),
    params(
        ("folder_id" = String, Path, description = "Folder public identifier")
    ),
    request_body = UpdateFolderRequest,
    responses(
        (status = 200, description = "Folder updated", body = FolderResponse),
        (status = 400, description = "Invalid update payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Folder not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update folder", body = crate::error::ErrorResponse)
    )
)]
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

#[utoipa::path(
    delete,
    path = "/api/folders/{folder_id}",
    tag = "Folders",
    security(("bearerAuth" = [])),
    params(
        ("folder_id" = String, Path, description = "Folder public identifier")
    ),
    responses(
        (status = 200, description = "Folder deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Folder not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete folder", body = crate::error::ErrorResponse)
    )
)]
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
