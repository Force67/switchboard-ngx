//! Chat invite-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Response containing a list of invites
#[derive(Debug, Serialize, ToSchema)]
pub struct InvitesResponse {
    pub invites: Vec<InviteResponse>,
}

/// Response containing invite details
#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub id: String,
    pub chat_id: String,
    pub chat_title: String,
    pub invited_by: String,
    pub invited_email: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
    pub inviter: InviteInviterResponse,
}

/// Inviter information embedded in invite response
#[derive(Debug, Serialize, ToSchema)]
pub struct InviteInviterResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for creating an invite
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    pub email: String,
    /// Will default to 'member'
    pub role: Option<String>,
    /// Optional custom expiration in hours
    pub expires_in_hours: Option<i64>,
}

/// Query parameters for listing invites
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListInvitesQuery {
    /// Filter by status: pending, accepted, rejected, expired
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request body for responding to an invite
#[derive(Debug, Deserialize, ToSchema)]
pub struct RespondToInviteRequest {
    /// Must be "accept" or "reject"
    pub action: String,
}

impl From<switchboard_database::ChatInvite> for InviteResponse {
    fn from(invite: switchboard_database::ChatInvite) -> Self {
        Self {
            id: invite.public_id,
            chat_id: invite.chat_public_id,
            chat_title: invite.chat_title,
            invited_by: invite.invited_by_public_id.clone(),
            invited_email: invite.invited_email.clone(),
            status: invite.status.to_string(),
            created_at: invite.created_at.clone(),
            expires_at: invite.expires_at.clone(),
            accepted_at: invite.accepted_at.clone(),
            inviter: InviteInviterResponse {
                id: invite.invited_by_public_id,
                display_name: invite.inviter_display_name,
                avatar_url: invite.inviter_avatar_url,
            },
        }
    }
}
