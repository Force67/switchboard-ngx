use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};

use crate::{
    routes::models::{
        CreatePermissionRequest, Permission, PermissionResponse, PermissionsResponse,
    },
    util::require_bearer,
    ApiError, AppState,
};

// Permission levels and resource types
pub const PERMISSION_LEVELS: &[&str] = &["read", "write", "admin"];
pub const RESOURCE_TYPES: &[&str] = &["chat", "folder", "workspace"];

// Permissions service
pub struct PermissionsService;

impl PermissionsService {
    // Check if user has permission for a resource
    pub async fn check_permission(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        resource_type: &str,
        resource_id: i64,
        required_permission: &str,
    ) -> Result<bool, ApiError> {
        let permission_level = sqlx::query_scalar::<_, String>(
            r#"
            SELECT permission_level
            FROM permissions
            WHERE user_id = ? AND resource_type = ? AND resource_id = ?
            ORDER BY
                CASE permission_level
                    WHEN 'admin' THEN 1
                    WHEN 'write' THEN 2
                    WHEN 'read' THEN 3
                    ELSE 4
                END
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(resource_type)
        .bind(resource_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check permission: {}", e);
            ApiError::internal_server_error("Failed to check permission")
        })?;

        if let Some(level) = permission_level {
            match required_permission {
                "read" => Ok(true), // All permission levels include read
                "write" => Ok(matches!(level.as_str(), "write" | "admin")),
                "admin" => Ok(level == "admin"),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    // Grant permission to user for a resource
    pub async fn grant_permission(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        resource_type: &str,
        resource_id: i64,
        permission_level: &str,
        granted_by_user_id: i64,
    ) -> Result<(), ApiError> {
        let now = chrono::Utc::now().to_rfc3339();

        // Use INSERT OR REPLACE to handle existing permissions
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO permissions (user_id, resource_type, resource_id, permission_level, granted_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(user_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(permission_level)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to grant permission: {}", e);
            ApiError::internal_server_error("Failed to grant permission")
        })?;

        Ok(())
    }

    // Revoke permission from user for a resource
    pub async fn revoke_permission(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        resource_type: &str,
        resource_id: i64,
    ) -> Result<(), ApiError> {
        let result = sqlx::query(
            "DELETE FROM permissions WHERE user_id = ? AND resource_type = ? AND resource_id = ?",
        )
        .bind(user_id)
        .bind(resource_type)
        .bind(resource_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke permission: {}", e);
            ApiError::internal_server_error("Failed to revoke permission")
        })?;

        if result.rows_affected() == 0 {
            return Err(ApiError::not_found("Permission not found"));
        }

        Ok(())
    }

    // Get user permissions for a resource
    pub async fn get_user_permissions(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        resource_type: Option<&str>,
        resource_id: Option<i64>,
    ) -> Result<Vec<Permission>, ApiError> {
        let permissions = match (resource_type, resource_id) {
            (Some(rt), Some(rid)) => sqlx::query_as::<_, Permission>(
                r#"
                    SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                    FROM permissions
                    WHERE user_id = ? AND resource_type = ? AND resource_id = ?
                    ORDER BY granted_at DESC
                    "#,
            )
            .bind(user_id)
            .bind(rt)
            .bind(rid)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user permissions: {}", e);
                ApiError::internal_server_error("Failed to fetch user permissions")
            })?,
            (Some(rt), None) => sqlx::query_as::<_, Permission>(
                r#"
                    SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                    FROM permissions
                    WHERE user_id = ? AND resource_type = ?
                    ORDER BY granted_at DESC
                    "#,
            )
            .bind(user_id)
            .bind(rt)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user permissions: {}", e);
                ApiError::internal_server_error("Failed to fetch user permissions")
            })?,
            (None, Some(rid)) => sqlx::query_as::<_, Permission>(
                r#"
                    SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                    FROM permissions
                    WHERE user_id = ? AND resource_id = ?
                    ORDER BY granted_at DESC
                    "#,
            )
            .bind(user_id)
            .bind(rid)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user permissions: {}", e);
                ApiError::internal_server_error("Failed to fetch user permissions")
            })?,
            (None, None) => sqlx::query_as::<_, Permission>(
                r#"
                    SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                    FROM permissions
                    WHERE user_id = ?
                    ORDER BY granted_at DESC
                    "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch user permissions: {}", e);
                ApiError::internal_server_error("Failed to fetch user permissions")
            })?,
        };

        Ok(permissions)
    }

    // Get all permissions for a resource
    pub async fn get_resource_permissions(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        resource_type: &str,
        resource_id: i64,
    ) -> Result<Vec<Permission>, ApiError> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
            FROM permissions
            WHERE resource_type = ? AND resource_id = ?
            ORDER BY granted_at DESC
            "#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch resource permissions: {}", e);
            ApiError::internal_server_error("Failed to fetch resource permissions")
        })?;

        Ok(permissions)
    }

    // Check chat membership permissions (chat_members table)
    pub async fn check_chat_permission(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        chat_id: i64,
        required_permission: &str,
    ) -> Result<bool, ApiError> {
        let role = sqlx::query_scalar::<_, String>(
            "SELECT role FROM chat_members WHERE chat_id = ? AND user_id = ?",
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check chat membership: {}", e);
            ApiError::internal_server_error("Failed to check chat membership")
        })?;

        if let Some(role) = role {
            match required_permission {
                "read" => Ok(true),  // All chat members can read
                "write" => Ok(true), // All chat members can write messages
                "admin" => Ok(matches!(role.as_str(), "admin" | "owner")),
                "owner" => Ok(role == "owner"),
                _ => Ok(false),
            }
        } else {
            // Check permissions table for explicit permission
            Self::check_permission(pool, user_id, "chat", chat_id, required_permission).await
        }
    }

    // Check folder ownership permissions
    pub async fn check_folder_permission(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        folder_id: i64,
        required_permission: &str,
    ) -> Result<bool, ApiError> {
        let folder_user_id =
            sqlx::query_scalar::<_, i64>("SELECT user_id FROM folders WHERE id = ?")
                .bind(folder_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check folder ownership: {}", e);
                    ApiError::internal_server_error("Failed to check folder ownership")
                })?;

        if let Some(owner_id) = folder_user_id {
            if owner_id == user_id {
                return Ok(true); // Owner has all permissions
            }
        }

        // Check permissions table for explicit permission
        Self::check_permission(pool, user_id, "folder", folder_id, required_permission).await
    }
}

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

    // Get the target user
    let target_user_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE public_id = ?")
            .bind(&user_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to get user ID: {}", e);
                ApiError::internal_server_error("Failed to get user ID")
            })?;

    let target_user_id = target_user_id.ok_or_else(|| ApiError::not_found("User not found"))?;

    // Users can only see their own permissions unless they're admin
    if current_user.id != target_user_id {
        // Check if current user is admin (you might want to implement admin check)
        return Err(ApiError::forbidden("Cannot view other users' permissions"));
    }

    let permissions =
        PermissionsService::get_user_permissions(state.db_pool(), target_user_id, None, None)
            .await?;

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

    // Resolve resource ID from public ID
    let resource_id = match resource_type.as_str() {
        "chat" => sqlx::query_scalar::<_, i64>("SELECT id FROM chats WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve chat ID: {}", e);
                ApiError::internal_server_error("Failed to resolve chat ID")
            })?,
        "folder" => sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder ID: {}", e);
                ApiError::internal_server_error("Failed to resolve folder ID")
            })?,
        _ => return Err(ApiError::bad_request("Invalid resource type")),
    };

    let resource_id = resource_id.ok_or_else(|| ApiError::not_found("Resource not found"))?;

    // Check if user has admin permission for this resource
    if !PermissionsService::check_permission(
        state.db_pool(),
        user.id,
        &resource_type,
        resource_id,
        "admin",
    )
    .await?
    {
        return Err(ApiError::forbidden("Admin permission required"));
    }

    let permissions =
        PermissionsService::get_resource_permissions(state.db_pool(), &resource_type, resource_id)
            .await?;

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

    // Resolve resource ID
    let resource_id = match resource_type.as_str() {
        "chat" => sqlx::query_scalar::<_, i64>("SELECT id FROM chats WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve chat ID: {}", e);
                ApiError::internal_server_error("Failed to resolve chat ID")
            })?,
        "folder" => sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder ID: {}", e);
                ApiError::internal_server_error("Failed to resolve folder ID")
            })?,
        _ => return Err(ApiError::bad_request("Invalid resource type")),
    };

    let resource_id = resource_id.ok_or_else(|| ApiError::not_found("Resource not found"))?;

    // Check if user has admin permission for this resource
    if !PermissionsService::check_permission(
        state.db_pool(),
        user.id,
        &resource_type,
        resource_id,
        "admin",
    )
    .await?
    {
        return Err(ApiError::forbidden("Admin permission required"));
    }

    // Resolve target user ID
    let target_user_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE public_id = ?")
            .bind(&req.user_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve target user ID: {}", e);
                ApiError::internal_server_error("Failed to resolve target user ID")
            })?;

    let target_user_id =
        target_user_id.ok_or_else(|| ApiError::not_found("Target user not found"))?;

    // Grant the permission
    PermissionsService::grant_permission(
        state.db_pool(),
        target_user_id,
        &req.resource_type,
        resource_id,
        &req.permission_level,
        user.id,
    )
    .await?;

    // Fetch the created/updated permission
    let permission = sqlx::query_as::<_, Permission>(
        r#"
        SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
        FROM permissions
        WHERE user_id = ? AND resource_type = ? AND resource_id = ?
        "#,
    )
    .bind(target_user_id)
    .bind(&req.resource_type)
    .bind(resource_id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch created permission: {}", e);
        ApiError::internal_server_error("Failed to fetch created permission")
    })?
    .ok_or_else(|| ApiError::internal_server_error("Failed to fetch created permission"))?;

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

    // Resolve resource ID
    let resource_id = match resource_type.as_str() {
        "chat" => sqlx::query_scalar::<_, i64>("SELECT id FROM chats WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve chat ID: {}", e);
                ApiError::internal_server_error("Failed to resolve chat ID")
            })?,
        "folder" => sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ?")
            .bind(&resource_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve folder ID: {}", e);
                ApiError::internal_server_error("Failed to resolve folder ID")
            })?,
        _ => return Err(ApiError::bad_request("Invalid resource type")),
    };

    let resource_id = resource_id.ok_or_else(|| ApiError::not_found("Resource not found"))?;

    // Check if user has admin permission for this resource
    if !PermissionsService::check_permission(
        state.db_pool(),
        user.id,
        &resource_type,
        resource_id,
        "admin",
    )
    .await?
    {
        return Err(ApiError::forbidden("Admin permission required"));
    }

    // Resolve target user ID
    let target_user_id: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE public_id = ?")
            .bind(&user_public_id)
            .fetch_optional(state.db_pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve target user ID: {}", e);
                ApiError::internal_server_error("Failed to resolve target user ID")
            })?;

    let target_user_id =
        target_user_id.ok_or_else(|| ApiError::not_found("Target user not found"))?;

    // Revoke the permission
    PermissionsService::revoke_permission(
        state.db_pool(),
        target_user_id,
        &resource_type,
        resource_id,
    )
    .await?;

    Ok(())
}
