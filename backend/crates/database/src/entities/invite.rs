//! Invite entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvite {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub inviter_id: i64,
    pub invitee_email: Option<String>,
    pub invite_code: String,
    pub status: InviteStatus,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInviteRequest {
    pub chat_id: i64,
    pub invitee_email: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InviteStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}

impl InviteStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InviteStatus::Pending => "pending",
            InviteStatus::Accepted => "accepted",
            InviteStatus::Declined => "declined",
            InviteStatus::Expired => "expired",
        }
    }
}

impl From<&str> for InviteStatus {
    fn from(s: &str) -> Self {
        match s {
            "accepted" => InviteStatus::Accepted,
            "declined" => InviteStatus::Declined,
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