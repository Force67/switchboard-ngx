//! Chat entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub public_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
    pub status: ChatStatus,
    pub created_by: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub chat_type: ChatType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ChatStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatType {
    Direct,
    Group,
    Channel,
}

impl ChatType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatType::Direct => "direct",
            ChatType::Group => "group",
            ChatType::Channel => "channel",
        }
    }
}

impl From<&str> for ChatType {
    fn from(s: &str) -> Self {
        match s {
            "group" => ChatType::Group,
            "channel" => ChatType::Channel,
            _ => ChatType::Direct,
        }
    }
}

impl ToString for ChatType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatStatus {
    Active,
    Archived,
    Deleted,
}

impl ChatStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatStatus::Active => "active",
            ChatStatus::Archived => "archived",
            ChatStatus::Deleted => "deleted",
        }
    }
}

impl From<&str> for ChatStatus {
    fn from(s: &str) -> Self {
        match s {
            "archived" => ChatStatus::Archived,
            "deleted" => ChatStatus::Deleted,
            _ => ChatStatus::Active,
        }
    }
}

impl ToString for ChatStatus {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}