use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::{GatewayError, GatewayResult};
use crate::state::GatewayState;

#[derive(Debug, Serialize, FromRow, ToSchema, Clone)]
pub struct Folder {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<i64>,
    pub collapsed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FolderResponse {
    pub folder: Folder,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>, // parent public_id
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub collapsed: Option<bool>,
    pub parent_id: Option<String>, // parent public_id
}

pub fn create_folder_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/folders", get(list_folders).post(create_folder))
        .route(
            "/folders/:folder_id",
            get(get_folder).put(update_folder).delete(delete_folder),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/folders",
    tag = "Folders",
    responses(
        (status = 200, description = "List folders for the current user", body = FoldersResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch folders", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_folders(
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
) -> GatewayResult<Json<FoldersResponse>> {
    let folders = sqlx::query_as::<_, Folder>(
        "SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
         FROM folders
         WHERE user_id = ?
         ORDER BY created_at ASC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(map_db_error)?;

    Ok(Json(FoldersResponse { folders }))
}

#[utoipa::path(
    post,
    path = "/api/v1/folders",
    tag = "Folders",
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created", body = FolderResponse),
        (status = 400, description = "Invalid folder payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Parent folder not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create folder", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_folder(
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<CreateFolderRequest>,
) -> GatewayResult<Json<FolderResponse>> {
    let public_id = cuid2::cuid();
    let now = Utc::now().to_rfc3339();

    let parent_id = if let Some(parent_public_id) = req.parent_id.as_deref() {
        Some(resolve_parent_id(&state, user_id, parent_public_id).await?)
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
    )
    .bind(&public_id)
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.color)
    .bind(parent_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(map_db_error)?;

    let folder = fetch_folder(&state, user_id, &public_id).await?;
    Ok(Json(FolderResponse { folder }))
}

#[utoipa::path(
    get,
    path = "/api/v1/folders/{folder_id}",
    tag = "Folders",
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
    State(state): State<Arc<GatewayState>>,
    Path(folder_id): Path<String>,
    Extension(user_id): Extension<i64>,
) -> GatewayResult<Json<FolderResponse>> {
    let folder = fetch_folder(&state, user_id, &folder_id).await?;
    Ok(Json(FolderResponse { folder }))
}

#[utoipa::path(
    put,
    path = "/api/v1/folders/{folder_id}",
    tag = "Folders",
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
    State(state): State<Arc<GatewayState>>,
    Path(folder_id): Path<String>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<UpdateFolderRequest>,
) -> GatewayResult<Json<FolderResponse>> {
    let current = fetch_folder(&state, user_id, &folder_id).await?;
    let parent_id = if let Some(parent_public_id) = req.parent_id.as_deref() {
        let resolved = resolve_parent_id(&state, user_id, parent_public_id).await?;
        if resolved == current.id {
            return Err(GatewayError::InvalidRequest(
                "Folder cannot be its own parent".to_string(),
            ));
        }
        Some(resolved)
    } else {
        None
    };

    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "UPDATE folders
         SET name = COALESCE(?, name),
             color = COALESCE(?, color),
             collapsed = COALESCE(?, collapsed),
             parent_id = COALESCE(?, parent_id),
             updated_at = ?
         WHERE public_id = ? AND user_id = ?",
    )
    .bind(req.name.as_ref())
    .bind(req.color.as_ref())
    .bind(req.collapsed)
    .bind(parent_id)
    .bind(&now)
    .bind(&folder_id)
    .bind(user_id)
    .execute(&state.pool)
    .await
    .map_err(map_db_error)?;

    let folder = fetch_folder(&state, user_id, &folder_id).await?;
    Ok(Json(FolderResponse { folder }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/folders/{folder_id}",
    tag = "Folders",
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
    State(state): State<Arc<GatewayState>>,
    Path(folder_id): Path<String>,
    Extension(user_id): Extension<i64>,
) -> GatewayResult<()> {
    let folder = fetch_folder(&state, user_id, &folder_id).await?;

    sqlx::query("UPDATE chats SET folder_id = NULL WHERE folder_id = ? AND created_by = ?")
        .bind(&folder.public_id)
        .bind(user_id.to_string())
        .execute(&state.pool)
        .await
        .map_err(map_db_error)?;

    let result = sqlx::query("DELETE FROM folders WHERE public_id = ? AND user_id = ?")
        .bind(&folder_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(map_db_error)?;

    if result.rows_affected() == 0 {
        return Err(GatewayError::NotFound("Folder not found".to_string()));
    }

    Ok(())
}

async fn resolve_parent_id(
    state: &GatewayState,
    user_id: i64,
    parent_public_id: &str,
) -> GatewayResult<i64> {
    let parent = fetch_folder(state, user_id, parent_public_id).await?;
    Ok(parent.id)
}

async fn fetch_folder(
    state: &GatewayState,
    user_id: i64,
    folder_public_id: &str,
) -> GatewayResult<Folder> {
    sqlx::query_as::<_, Folder>(
        "SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
         FROM folders
         WHERE public_id = ? AND user_id = ?",
    )
    .bind(folder_public_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| GatewayError::NotFound("Folder not found".to_string()))
}

fn map_db_error(error: sqlx::Error) -> GatewayError {
    GatewayError::DatabaseError(error.to_string())
}
