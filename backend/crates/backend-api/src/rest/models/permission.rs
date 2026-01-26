//! Permission-related request and response types

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Permission entity
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Permission {
    pub id: i64,
    pub user_id: i64,
    pub resource_type: String,
    pub resource_id: i64,
    pub permission_level: String,
    pub granted_at: String,
}

/// Request body for creating a permission
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    /// User public_id
    pub user_id: String,
    pub resource_type: String,
    /// Resource public_id
    pub resource_id: String,
    pub permission_level: String,
}

/// Response containing multiple permissions
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionsResponse {
    pub permissions: Vec<Permission>,
}

/// Response containing a single permission
#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub permission: Permission,
}
