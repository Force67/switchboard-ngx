//! Chat-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::member::ChatMemberResponse;

/// Response containing a list of chats
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatsResponse {
    pub chats: Vec<ChatResponse>,
}

/// Response containing chat details
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatResponse {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub messages: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub member_count: i64,
    pub message_count: i64,
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub members: Vec<ChatMemberResponse>,
}

/// Request body for creating a new chat
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChatRequest {
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub is_group: bool,
    pub initial_message: Option<String>,
}

/// Request body for updating a chat
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub folder_id: Option<String>,
}

/// Query parameters for listing chats
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListChatsQuery {
    pub folder_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<switchboard_database::Chat> for ChatResponse {
    fn from(chat: switchboard_database::Chat) -> Self {
        Self {
            id: chat.id,
            public_id: chat.public_id.clone(),
            user_id: chat.created_by.parse().unwrap_or(0),
            title: chat.title,
            description: chat.description,
            avatar_url: chat.avatar_url,
            folder_id: chat.folder_id,
            is_group: matches!(chat.chat_type, switchboard_database::ChatType::Group),
            messages: Some("[]".to_string()),
            created_by: chat.created_by,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
            member_count: chat.member_count,
            message_count: chat.message_count,
            last_message_at: chat.last_message_at,
            members: vec![],
        }
    }
}
