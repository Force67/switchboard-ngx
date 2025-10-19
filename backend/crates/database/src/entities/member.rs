//! Member entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMember {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub role: MemberRole,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub chat_id: i64,
    pub user_id: i64,
    pub role: MemberRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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