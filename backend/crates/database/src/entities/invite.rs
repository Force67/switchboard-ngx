//! Invite entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvite {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub chat_public_id: String,
    pub chat_title: String,
    pub inviter_id: i64,
    pub invited_by_public_id: String,
    pub inviter_display_name: Option<String>,
    pub inviter_avatar_url: Option<String>,
    pub invited_email: String,
    pub invite_code: String,
    pub status: InviteStatus,
    pub expires_at: String,
    pub created_at: String,
    pub accepted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInviteRequest {
    pub chat_id: i64,
    pub chat_public_id: String,
    pub invited_by_public_id: String,
    pub invited_email: String,
    pub expires_in_hours: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum InviteStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

impl InviteStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InviteStatus::Pending => "pending",
            InviteStatus::Accepted => "accepted",
            InviteStatus::Rejected => "rejected",
            InviteStatus::Expired => "expired",
        }
    }
}

impl From<&str> for InviteStatus {
    fn from(s: &str) -> Self {
        match s {
            "accepted" => InviteStatus::Accepted,
            "rejected" => InviteStatus::Rejected,
            "expired" => InviteStatus::Expired,
            _ => InviteStatus::Pending,
        }
    }
}

impl ToString for InviteStatus {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}