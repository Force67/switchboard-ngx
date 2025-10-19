//! Message entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub chat_public_id: String,
    pub sender_id: i64,
    pub sender_public_id: String,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub reply_to_id: Option<i64>,
    pub reply_to_public_id: Option<String>,
    pub thread_id: Option<i64>,
    pub thread_public_id: Option<String>,
    pub status: MessageStatus,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
    pub sender_display_name: Option<String>,
    pub sender_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub chat_id: i64,
    pub chat_public_id: String,
    pub sender_id: i64,
    pub sender_public_id: String,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub reply_to_public_id: Option<String>,
    pub thread_public_id: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
            MessageType::System => "system",
        }
    }
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        match s {
            "image" => MessageType::Image,
            "file" => MessageType::File,
            "system" => MessageType::System,
            _ => MessageType::Text,
        }
    }
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}