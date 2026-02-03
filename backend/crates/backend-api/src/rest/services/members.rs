//! Members service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::MembersServiceTrait;
use crate::rest::models::{
    ListMembersQuery, MemberResponse, MembersResponse, UpdateMemberRoleRequest,
};
use crate::state::GatewayState;

/// Implementation of MembersServiceTrait
pub struct MembersServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> MembersServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl MembersServiceTrait for MembersServiceImpl<'_> {
    async fn list_members(
        &self,
        user_id: i64,
        chat_id: String,
        query: ListMembersQuery,
    ) -> GatewayResult<MembersResponse> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let role_filter = match query.role.as_deref() {
            Some("owner") => Some(switchboard_database::MemberRole::Owner),
            Some("admin") => Some(switchboard_database::MemberRole::Admin),
            Some("member") => Some(switchboard_database::MemberRole::Member),
            _ => None,
        };

        let members = self
            .state
            .member_repo
            .list_by_chat_public(&chat_id, role_filter, query.limit, query.offset)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list members: {}", e)))?;

        let members: Vec<MemberResponse> = members.into_iter().map(|m| m.into()).collect();
        Ok(MembersResponse { members })
    }

    async fn get_member(
        &self,
        user_id: i64,
        chat_id: String,
        member_id: String,
    ) -> GatewayResult<MemberResponse> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let member = self
            .state
            .member_repo
            .find_by_public_id(&member_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
            .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

        if member.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Member does not belong to specified chat".to_string(),
            ));
        }

        Ok(MemberResponse::from(member))
    }

    async fn update_member_role(
        &self,
        user_id: i64,
        chat_id: String,
        member_id: String,
        req: UpdateMemberRoleRequest,
    ) -> GatewayResult<MemberResponse> {
        // Check if user is owner or admin
        self.state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let member = self
            .state
            .member_repo
            .find_by_public_id(&member_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
            .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

        if member.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Member does not belong to specified chat".to_string(),
            ));
        }

        let new_role = match req.role.as_str() {
            "owner" => switchboard_database::MemberRole::Owner,
            "admin" => switchboard_database::MemberRole::Admin,
            "member" => switchboard_database::MemberRole::Member,
            _ => {
                return Err(GatewayError::InvalidRequest(
                    "Role must be 'owner', 'admin', or 'member'".to_string(),
                ))
            }
        };

        // Only owners can promote others to owner
        if new_role == switchboard_database::MemberRole::Owner {
            self.state
                .permissions()
                .require_role(&chat_id, user_id, switchboard_database::MemberRole::Owner)
                .await
                .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
        }

        if member.user_id == user_id && new_role != switchboard_database::MemberRole::Owner {
            return Err(GatewayError::InvalidRequest(
                "You cannot demote yourself from owner role".to_string(),
            ));
        }

        let updated_member = self
            .state
            .member_repo
            .update_role_by_id(member.id, &new_role, user_id)
            .await
            .map_err(|e| {
                GatewayError::ServiceError(format!("Failed to update member role: {}", e))
            })?;

        Ok(MemberResponse::from(updated_member))
    }

    async fn remove_member(
        &self,
        user_id: i64,
        chat_id: String,
        member_id: String,
    ) -> GatewayResult<()> {
        // Check if user is owner or admin
        self.state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let member = self
            .state
            .member_repo
            .find_by_public_id(&member_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
            .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

        if member.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Member does not belong to specified chat".to_string(),
            ));
        }

        if member.role == switchboard_database::MemberRole::Owner {
            return Err(GatewayError::InvalidRequest(
                "Cannot remove the last owner from the chat".to_string(),
            ));
        }

        if member.user_id != user_id {
            self.state
                .permissions()
                .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
                .await
                .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
        }

        self.state
            .member_repo
            .delete_by_id(member.id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to remove member: {}", e)))?;

        Ok(())
    }

    async fn leave_chat(&self, user_id: i64, chat_id: String) -> GatewayResult<()> {
        // Check if user is a member
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        // Get list of owners
        let owners = self
            .state
            .member_repo
            .list_by_chat_public(
                &chat_id,
                Some(switchboard_database::MemberRole::Owner),
                None,
                None,
            )
            .await
            .map_err(|e| {
                GatewayError::ServiceError(format!("Failed to check membership: {}", e))
            })?;

        // Check if user is the last owner
        let is_owner = owners.iter().any(|m| m.user_id == user_id);
        if is_owner && owners.len() == 1 {
            return Err(GatewayError::InvalidRequest(
                "Cannot leave chat as the last owner".to_string(),
            ));
        }

        self.state
            .member_repo
            .delete_by_user_and_chat_public(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to leave chat: {}", e)))?;

        Ok(())
    }
}
