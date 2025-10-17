use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use crate::{
    routes::models::{
        CreatePermissionRequest, PermissionResponse, PermissionsResponse,
    },
    services::permission as permission_service,
    util::require_bearer,
    ApiError, AppState,
};

// API Handlers

// Get user permissions
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/permissions",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("user_id" = String, Path, description = "User public identifier")
    ),
    responses(
        (status = 200, description = "Permissions for the specified user", body = PermissionsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch permissions", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_user_permissions(
    State(state): State<AppState>,
    Path(user_public_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<PermissionsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (current_user, _) = state.authenticate(&token).await?;

    let permissions = permission_service::get_user_permissions_with_auth(
        state.db_pool(),
        current_user.id,
        &user_public_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get user permissions: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(PermissionsResponse { permissions }))
}

// Get resource permissions
#[utoipa::path(
    get,
    path = "/api/permissions/{resource_type}/{resource_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Type of resource (chat, folder, workspace)"),
        ("resource_id" = String, Path, description = "Resource public identifier")
    ),
    responses(
        (status = 200, description = "Permissions on the resource", body = PermissionsResponse),
        (status = 400, description = "Invalid resource type", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Admin permission required", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch permissions", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_resource_permissions(
    State(state): State<AppState>,
    Path((resource_type, resource_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<PermissionsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let permissions = permission_service::get_resource_permissions_with_auth(
        state.db_pool(),
        user.id,
        &resource_type,
        &resource_public_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get resource permissions: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(PermissionsResponse { permissions }))
}

// Grant permission to user
#[utoipa::path(
    post,
    path = "/api/permissions/{resource_type}/{resource_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Type of resource (chat, folder, workspace)"),
        ("resource_id" = String, Path, description = "Resource public identifier")
    ),
    request_body = CreatePermissionRequest,
    responses(
        (status = 200, description = "Permission granted", body = PermissionResponse),
        (status = 400, description = "Invalid request payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Admin permission required", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource or user not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to grant permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn grant_permission(
    State(state): State<AppState>,
    Path((resource_type, resource_public_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<CreatePermissionRequest>,
) -> Result<Json<PermissionResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let permission = permission_service::grant_permission_with_auth(
        state.db_pool(),
        user.id,
        &resource_type,
        &resource_public_id,
        &req.user_id,
        &req.permission_level,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to grant permission: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(PermissionResponse { permission }))
}

// Revoke permission from user
#[utoipa::path(
    delete,
    path = "/api/permissions/{resource_type}/{resource_id}/{user_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Type of resource (chat, folder, workspace)"),
        ("resource_id" = String, Path, description = "Resource public identifier"),
        ("user_id" = String, Path, description = "User public identifier")
    ),
    responses(
        (status = 200, description = "Permission revoked"),
        (status = 400, description = "Invalid resource type", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Admin permission required", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource or user not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to revoke permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn revoke_permission(
    State(state): State<AppState>,
    Path((resource_type, resource_public_id, user_public_id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    permission_service::revoke_permission_with_auth(
        state.db_pool(),
        user.id,
        &resource_type,
        &resource_public_id,
        &user_public_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to revoke permission: {}", e);
        ApiError::from(e)
    })?;

    Ok(())
}
