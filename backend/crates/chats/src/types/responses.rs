//! Response types for the chat system.

use serde::{Deserialize, Serialize};

// Chat with messages response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWithMessages {
    pub id: i64,
    pub public_id: String,
    pub user_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub title: String,
    pub chat_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_group: bool,
    pub messages: Option<String>,
}

// Member with user response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithUser {
    pub id: i64,
    pub chat_id: String,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
    pub user: Option<UserSummary>,
}

// Invite with details response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteWithDetails {
    pub id: i64,
    pub chat_id: String,
    pub inviter_user_id: i64,
    pub invitee_email: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub chat: Option<ChatSummary>,
}

// Token usage for AI completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// User summary for nested responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: i64,
    pub public_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

// Chat summary for nested responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSummary {
    pub id: i64,
    pub public_id: String,
    pub title: String,
    pub chat_type: String,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// List of items
    pub items: Vec<T>,
    /// Current page number
    pub page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Total number of items
    pub total_items: u64,
    /// Items per page
    pub per_page: u32,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(
        items: Vec<T>,
        page: u32,
        per_page: u32,
        total_items: u64,
    ) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        Self {
            items,
            page,
            total_pages,
            total_items,
            per_page,
        }
    }

    /// Create an empty paginated response
    pub fn empty(page: u32, per_page: u32) -> Self {
        Self {
            items: vec![],
            page,
            total_pages: 0,
            total_items: 0,
            per_page,
        }
    }
}

/// Response for chat creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatResponse {
    /// Created chat
    pub chat: switchboard_database::Chat,
    /// Initial messages if any
    pub messages: Vec<switchboard_database::ChatMessage>,
}

/// Response for message creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageResponse {
    /// Created message
    pub message: switchboard_database::ChatMessage,
    /// Updated chat information
    pub chat: switchboard_database::Chat,
}

/// Response for attachment creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentResponse {
    /// Created attachment
    pub attachment: switchboard_database::MessageAttachment,
    /// Message the attachment belongs to
    pub message: switchboard_database::ChatMessage,
}

/// Response for invitation creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInviteResponse {
    /// Created invitation
    pub invite: switchboard_database::ChatInvite,
    /// Chat the invitation is for
    pub chat: switchboard_database::Chat,
}

/// Response for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated message
    pub message: switchboard_database::ChatMessage,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// Model used
    pub model: String,
    /// Generation duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Response for member role update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemberRoleResponse {
    /// Updated member
    pub member: switchboard_database::ChatMember,
    /// Chat information
    pub chat: switchboard_database::Chat,
}

/// Response statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStats {
    /// Total number of chats
    pub total_chats: u64,
    /// Total number of messages
    pub total_messages: u64,
    /// Total number of attachments
    pub total_attachments: u64,
    /// Number of active chats
    pub active_chats: u64,
    /// Storage used in bytes
    pub storage_used_bytes: u64,
}