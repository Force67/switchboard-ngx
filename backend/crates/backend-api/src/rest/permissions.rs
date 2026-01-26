//! Permission REST endpoints

use axum::{
    extract::{Extension, Path, Request, State},
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::rest::models::{
    CreatePermissionRequest, Permission, PermissionResponse, PermissionsResponse,
};
use crate::state::GatewayState;

pub fn create_permission_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route(
            "/users/:user_id/permissions",
            axum::routing::get(get_user_permissions),
        )
        .route(
            "/resources/:resource_type/:resource_id/permissions",
            axum::routing::get(get_resource_permissions).post(create_permission),
        )
        .route(
            "/permissions/:permission_id",
            axum::routing::delete(delete_permission),
        )
}

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
    State(_state): State<Arc<GatewayState>>,
    Path(user_public_id): Path<String>,
    request: Request,
) -> GatewayResult<Json<PermissionsResponse>> {
    let _user_id = extract_user_id(&request)?;

    // TODO: Implement permission fetching from database
    // For now, return empty permissions list
    let permissions: Vec<Permission> = vec![];

    Ok(Json(PermissionsResponse { permissions }))
}

#[utoipa::path(
    get,
    path = "/api/v1/resources/{resource_type}/{resource_id}/permissions",
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
    State(_state): State<Arc<GatewayState>>,
    Path((resource_type, resource_id)): Path<(String, String)>,
    request: Request,
) -> GatewayResult<Json<PermissionsResponse>> {
    let _user_id = extract_user_id(&request)?;

    // TODO: Implement permission fetching from database
    // For now, return empty permissions list
    let permissions: Vec<Permission> = vec![];

    Ok(Json(PermissionsResponse { permissions }))
}

#[utoipa::path(
    post,
    path = "/api/v1/resources/{resource_type}/{resource_id}/permissions",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("resource_type" = String, Path, description = "Resource type (e.g., 'chat', 'folder')"),
        ("resource_id" = String, Path, description = "Resource public identifier")
    ),
    request_body = CreatePermissionRequest,
    responses(
        (status = 201, description = "Permission created", body = PermissionResponse),
        (status = 400, description = "Invalid permission payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Resource not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to create permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn create_permission(
    State(_state): State<Arc<GatewayState>>,
    Path((resource_type, resource_id)): Path<(String, String)>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<CreatePermissionRequest>,
) -> GatewayResult<Json<PermissionResponse>> {
    // TODO: Implement permission creation
    // For now, return a placeholder permission
    let permission = Permission {
        id: 0,
        user_id,
        resource_type: req.resource_type,
        resource_id: 0,
        permission_level: req.permission_level,
        granted_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(PermissionResponse { permission }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/permissions/{permission_id}",
    tag = "Permissions",
    security(("bearerAuth" = [])),
    params(
        ("permission_id" = i64, Path, description = "Permission ID")
    ),
    responses(
        (status = 200, description = "Permission deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::error::ErrorResponse),
        (status = 404, description = "Permission not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete permission", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_permission(
    State(_state): State<Arc<GatewayState>>,
    Path(permission_id): Path<i64>,
    request: Request,
) -> GatewayResult<()> {
    let _user_id = extract_user_id(&request)?;

    // TODO: Implement permission deletion

    Ok(())
}
