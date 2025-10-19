use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use crate::{
    routes::models::{CreateFolderRequest, Folder, UpdateFolderRequest},
    services::folder as folder_service,
    state::ServerEvent,
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

    let folders = folder_service::list_folders(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch folders: {}", e);
            ApiError::from(e)
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

    let folder = folder_service::create_folder(state.db_pool(), user.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create folder: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::FolderCreated {
        folder: folder.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

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

    let folder = folder_service::get_folder(state.db_pool(), user.id, &folder_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch folder: {}", e);
            ApiError::from(e)
        })?;

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

    let folder = folder_service::update_folder(state.db_pool(), user.id, &folder_id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update folder: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::FolderUpdated {
        folder: folder.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

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

    folder_service::delete_folder(state.db_pool(), user.id, &folder_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete folder: {}", e);
            ApiError::from(e)
        })?;

    let event = ServerEvent::FolderDeleted {
        folder_id: folder_id.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

    Ok(())
}
