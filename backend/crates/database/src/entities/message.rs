//! Message entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub message_type: String,
    pub status: MessageStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub chat_id: i64,
    pub content: String,
    pub message_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
    pub status: Option<MessageStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
    Deleted,
}

impl MessageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
            MessageStatus::Deleted => "deleted",
        }
    }
}

impl From<&str> for MessageStatus {
    fn from(s: &str) -> Self {
        match s {
            "delivered" => MessageStatus::Delivered,
            "read" => MessageStatus::Read,
            "deleted" => MessageStatus::Deleted,
            _ => MessageStatus::Sent,
        }
    }
}

impl ToString for MessageStatus {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}