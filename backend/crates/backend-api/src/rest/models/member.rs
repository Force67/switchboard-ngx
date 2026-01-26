//! Chat member-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Response containing a list of members
#[derive(Debug, Serialize, ToSchema)]
pub struct MembersResponse {
    pub members: Vec<MemberResponse>,
}

/// Response containing chat member details (used in chat and message responses)
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct ChatMemberResponse {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Full member response including chat ID
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub chat_id: String,
    pub role: String,
    pub joined_at: String,
    pub user: MemberUserResponse,
}

/// User information embedded in member response
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberUserResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

/// Request body for updating a member's role
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRoleRequest {
    /// Role must be "member", "admin", or "owner"
    pub role: String,
}

/// Query parameters for listing members
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListMembersQuery {
    /// Filter by role
    pub role: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<switchboard_database::ChatMember> for MemberResponse {
    fn from(member: switchboard_database::ChatMember) -> Self {
        Self {
            id: member.public_id,
            user_id: member.user_public_id.clone(),
            chat_id: member.chat_public_id,
            role: member.role.to_string(),
            joined_at: member.joined_at,
            user: MemberUserResponse {
                id: member.user_public_id,
                display_name: member.user_display_name,
                avatar_url: member.user_avatar_url,
                email: member.user_email,
            },
        }
    }
}
