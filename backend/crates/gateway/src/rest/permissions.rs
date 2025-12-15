use axum::{
    extract::{Path, State},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::state::GatewayState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    pub user_id: String, // public_id
    pub resource_type: String,
    pub resource_id: String, // public_id
    pub permission_level: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionsResponse {
    pub permissions: Vec<crate::rest::models::Permission>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub permission: crate::rest::models::Permission,
}

// Get user permissions
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/permissions",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("user_id" = String, Path, description = "User public identifier")
    ),
    responses(
        (status = 200, description = "Permissions for specified user", body = PermissionsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch permissions", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_user_permissions(
    State(state): State<Arc<GatewayState>>,
    Path(user_public_id): Path<String>,
    request: axum::http::Request<()>,
) -> GatewayResult<Json<PermissionsResponse>> {
    let current_user_id = extract_user_id(&request)?;

    // TODO: Implement actual permission fetching
    tracing::info!(
        "User {} requesting permissions for user {}",
        current_user_id,
        user_public_id
    );

    Ok(Json(PermissionsResponse {
        permissions: vec![],
    }))
}

// Get resource permissions
#[utoipa::path(
    get,
    path = "/api/v1/permissions/{resource_type}/{resource_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Resource type (e.g., 'chat', 'folder')"),
        ("resource_id" = String, Path, description = "Resource public identifier")
    ),
    responses(
        (status = 200, description = "Permissions for specified resource", body = PermissionsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch permissions", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_resource_permissions(
    State(state): State<Arc<GatewayState>>,
    Path((resource_type, resource_id)): Path<(String, String)>,
    request: axum::http::Request<()>,
) -> GatewayResult<Json<PermissionsResponse>> {
    let user_id = extract_user_id(&request)?;

    // TODO: Implement actual permission fetching
    tracing::info!(
        "User {} requesting permissions for {}/{}",
        user_id,
        resource_type,
        resource_id
    );

    Ok(Json(PermissionsResponse {
        permissions: vec![],
    }))
}

// Create permission
#[utoipa::path(
    post,
    path = "/api/v1/permissions/{resource_type}/{resource_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Resource type"),
        ("resource_id" = String, Path, description = "Resource public identifier")
    ),
    request_body = CreatePermissionRequest,
    responses(
        (status = 201, description = "Permission created", body = PermissionResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_permission(
    State(state): State<Arc<GatewayState>>,
    Path((resource_type, resource_id)): Path<(String, String)>,
    Json(payload): Json<CreatePermissionRequest>,
    request: axum::http::Request<()>,
) -> GatewayResult<Json<PermissionResponse>> {
    let user_id = extract_user_id(&request)?;

    // TODO: Implement actual permission creation
    tracing::info!(
        "User {} creating permission: user={}, resource={}/{}, level={}",
        user_id,
        payload.user_id,
        resource_type,
        resource_id,
        payload.permission_level
    );

    // Return mock response for now
    Ok(Json(PermissionResponse {
        permission: crate::rest::models::Permission {
            id: 1,
            user_id: 1,
            resource_type,
            resource_id: 1,
            permission_level: payload.permission_level,
            granted_at: chrono::Utc::now().to_rfc3339(),
        },
    }))
}

// Delete permission
#[utoipa::path(
    delete,
    path = "/api/v1/permissions/{resource_type}/{resource_id}/{user_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Resource type"),
        ("resource_id" = String, Path, description = "Resource public identifier"),
        ("user_id" = String, Path, description = "User public identifier")
    ),
    responses(
        (status = 204, description = "Permission deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = crate::error::ErrorResponse),
        (status = 404, description = "Permission not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_permission(
    State(state): State<Arc<GatewayState>>,
    Path((resource_type, resource_id, user_id)): Path<(String, String, String)>,
    request: axum::http::Request<()>,
) -> GatewayResult<()> {
    let current_user_id = extract_user_id(&request)?;

    // TODO: Implement actual permission deletion
    tracing::info!(
        "User {} deleting permission: user={}, resource={}/{}",
        current_user_id,
        user_id,
        resource_type,
        resource_id
    );

    Ok(())
}

/// Create permission routes
pub fn create_permission_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/users/:user_id/permissions", axum::routing::get(get_user_permissions))
        .route("/permissions/:resource_type/:resource_id", axum::routing::get(get_resource_permissions))
        .route("/permissions/:resource_type/:resource_id", axum::routing::post(create_permission))
        .route("/permissions/:resource_type/:resource_id/:user_id", axum::routing::delete(delete_permission))
}