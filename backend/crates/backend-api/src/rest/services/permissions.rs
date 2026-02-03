//! Permissions service implementation

use crate::error::GatewayResult;
use crate::generated::rest::traits::PermissionsServiceTrait;
use crate::rest::models::{
    CreatePermissionRequest, Permission, PermissionResponse, PermissionsResponse,
};
use crate::state::GatewayState;

/// Implementation of PermissionsServiceTrait
pub struct PermissionsServiceImpl<'a> {
    #[allow(dead_code)]
    state: &'a GatewayState,
}

impl<'a> PermissionsServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl PermissionsServiceTrait for PermissionsServiceImpl<'_> {
    async fn get_user_permissions(
        &self,
        _user_id: i64,           // authenticated user
        _target_user_id: String, // user whose permissions to fetch
    ) -> GatewayResult<PermissionsResponse> {
        // TODO: Implement permission fetching from database
        // For now, return empty permissions list
        let permissions: Vec<Permission> = vec![];

        Ok(PermissionsResponse { permissions })
    }

    async fn get_resource_permissions(
        &self,
        _user_id: i64,
        _resource_type: String,
        _resource_id: String,
    ) -> GatewayResult<PermissionsResponse> {
        // TODO: Implement permission fetching from database
        // For now, return empty permissions list
        let permissions: Vec<Permission> = vec![];

        Ok(PermissionsResponse { permissions })
    }

    async fn create_permission(
        &self,
        user_id: i64,
        _resource_type: String,
        _resource_id: String,
        req: CreatePermissionRequest,
    ) -> GatewayResult<PermissionResponse> {
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

        Ok(PermissionResponse { permission })
    }

    async fn delete_permission(&self, _user_id: i64, _permission_id: String) -> GatewayResult<()> {
        // TODO: Implement permission deletion

        Ok(())
    }
}
