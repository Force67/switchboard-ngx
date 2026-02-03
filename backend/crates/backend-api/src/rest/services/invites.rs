//! Invites service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::InvitesServiceTrait;
use crate::rest::models::{
    CreateInviteRequest, InviteResponse, InvitesResponse, ListInvitesQuery, RespondToInviteRequest,
};
use crate::state::GatewayState;

/// Implementation of InvitesServiceTrait
pub struct InvitesServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> InvitesServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl InvitesServiceTrait for InvitesServiceImpl<'_> {
    async fn list_invites(
        &self,
        user_id: i64,
        chat_id: String,
        query: ListInvitesQuery,
    ) -> GatewayResult<InvitesResponse> {
        // Check if user is owner or admin
        self.state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        // Parse status filter
        let status_filter = query.status.as_ref().and_then(|s| {
            match s.to_lowercase().as_str() {
                "pending" => Some(switchboard_database::InviteStatus::Pending),
                "accepted" => Some(switchboard_database::InviteStatus::Accepted),
                "rejected" => Some(switchboard_database::InviteStatus::Rejected),
                "expired" => Some(switchboard_database::InviteStatus::Expired),
                _ => None,
            }
        });

        let invites = self
            .state
            .invite_repo
            .list_by_chat_public(&chat_id, status_filter, query.limit, query.offset)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list invites: {}", e)))?;

        let invites: Vec<InviteResponse> = invites.into_iter().map(|i| i.into()).collect();
        Ok(InvitesResponse { invites })
    }

    async fn create_invite(
        &self,
        user_id: i64,
        chat_id: String,
        req: CreateInviteRequest,
    ) -> GatewayResult<InviteResponse> {
        // Check if user is owner or admin
        self.state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        // Get chat to resolve ID
        let chat = self
            .state
            .chat_repo
            .find_by_public_id(&chat_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
            .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

        // Get user's public_id
        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        let create_req = switchboard_database::CreateInviteRequest {
            chat_id: chat.id,
            chat_public_id: chat_id.clone(),
            invited_by_public_id: user.public_id,
            invited_email: req.email,
            expires_in_hours: req.expires_in_hours.unwrap_or(24),
        };

        let invite = self
            .state
            .invite_repo
            .create(user_id, &create_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to create invite: {}", e)))?;

        Ok(InviteResponse::from(invite))
    }

    async fn list_user_invites(
        &self,
        user_id: i64,
        query: ListInvitesQuery,
    ) -> GatewayResult<InvitesResponse> {
        // Get user to find their email
        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        let email = user.email.ok_or(GatewayError::InvalidRequest(
            "User does not have an email address".to_string(),
        ))?;

        // Parse status filter
        let status_filter = query.status.as_ref().and_then(|s| {
            match s.to_lowercase().as_str() {
                "pending" => Some(switchboard_database::InviteStatus::Pending),
                "accepted" => Some(switchboard_database::InviteStatus::Accepted),
                "rejected" => Some(switchboard_database::InviteStatus::Rejected),
                "expired" => Some(switchboard_database::InviteStatus::Expired),
                _ => None,
            }
        });

        let invites = self
            .state
            .invite_repo
            .list_by_user_email(&email, status_filter, query.limit, query.offset)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list invites: {}", e)))?;

        let invites: Vec<InviteResponse> = invites.into_iter().map(|i| i.into()).collect();
        Ok(InvitesResponse { invites })
    }

    async fn get_invite(&self, user_id: i64, invite_id: String) -> GatewayResult<InviteResponse> {
        let invite = self
            .state
            .invite_repo
            .find_by_public_id(&invite_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
            .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

        // Get user email to check if they're the invitee
        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        let is_invitee = user
            .email
            .as_ref()
            .map_or(false, |email| email == &invite.invited_email);

        // User can view invite if they're the invitee or the inviter (or admin of the chat)
        let can_view = invite.inviter_id == user_id
            || is_invitee
            || self
                .state
                .permissions()
                .require_role(
                    &invite.chat_public_id,
                    user_id,
                    switchboard_database::MemberRole::Admin,
                )
                .await
                .is_ok();

        if !can_view {
            return Err(GatewayError::AuthorizationFailed(
                "Access denied".to_string(),
            ));
        }

        Ok(InviteResponse::from(invite))
    }

    async fn respond_to_invite(
        &self,
        user_id: i64,
        invite_id: String,
        req: RespondToInviteRequest,
    ) -> GatewayResult<InviteResponse> {
        // Get the invite first
        let invite = self
            .state
            .invite_repo
            .find_by_public_id(&invite_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
            .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

        // Verify user is the invitee by email
        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        let is_invitee = user
            .email
            .as_ref()
            .map_or(false, |email| email == &invite.invited_email);
        if !is_invitee {
            return Err(GatewayError::AuthorizationFailed(
                "Access denied: not the invitee".to_string(),
            ));
        }

        let invite = match req.action.as_str() {
            "accept" => self
                .state
                .invite_repo
                .accept_by_id(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to accept invite: {}", e)))?,
            "reject" => self
                .state
                .invite_repo
                .decline_by_id(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to reject invite: {}", e)))?,
            _ => {
                return Err(GatewayError::InvalidRequest(
                    "Action must be 'accept' or 'reject'".to_string(),
                ))
            }
        };

        Ok(InviteResponse::from(invite))
    }

    async fn delete_invite(&self, user_id: i64, invite_id: String) -> GatewayResult<()> {
        let invite = self
            .state
            .invite_repo
            .find_by_public_id(&invite_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
            .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

        // Only the inviter or a chat admin can delete the invite
        if invite.inviter_id != user_id {
            self.state
                .permissions()
                .require_role(
                    &invite.chat_public_id,
                    user_id,
                    switchboard_database::MemberRole::Admin,
                )
                .await
                .map_err(|_| {
                    GatewayError::AuthorizationFailed("Access denied: not authorized".to_string())
                })?;
        }

        self.state
            .invite_repo
            .delete_by_id(invite.id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to delete invite: {}", e)))?;

        Ok(())
    }
}
