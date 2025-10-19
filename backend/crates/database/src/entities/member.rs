//! Member entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMember {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub chat_public_id: String,
    pub user_id: i64,
    pub user_public_id: String,
    pub role: MemberRole,
    pub joined_at: String,
    pub user_display_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub user_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub chat_id: i64,
    pub user_id: i64,
    pub role: MemberRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: MemberRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Owner => "owner",
            MemberRole::Admin => "admin",
            MemberRole::Member => "member",
        }
    }
}

impl From<&str> for MemberRole {
    fn from(s: &str) -> Self {
        match s {
            "owner" => MemberRole::Owner,
            "admin" => MemberRole::Admin,
            _ => MemberRole::Member,
        }
    }
}

impl ToString for MemberRole {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl MemberRole {
    /// Check if this role is equal to or higher than the required role
    pub fn is_higher_or_equal(&self, required_role: &MemberRole) -> bool {
        match (self, required_role) {
            (MemberRole::Owner, _) => true, // Owner is higher than all
            (MemberRole::Admin, MemberRole::Admin | MemberRole::Member) => true,
            (MemberRole::Member, MemberRole::Member) => true,
            (MemberRole::Member, MemberRole::Owner | MemberRole::Admin) => false,
            (MemberRole::Admin, MemberRole::Owner) => false,
        }
    }
}

impl ChatMember {
    /// Check if this member has the required role or higher
    pub fn has_role_or_higher(&self, required_role: &MemberRole) -> bool {
        self.role.is_higher_or_equal(required_role)
    }
}