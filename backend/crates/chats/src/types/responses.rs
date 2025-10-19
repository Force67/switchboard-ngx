//! Response types for the chat system.

use serde::{Deserialize, Serialize};

// Re-export response types from entities
pub use crate::entities::chat::ChatWithMessages;
pub use crate::entities::member::MemberWithUser;
pub use crate::entities::invite::InviteWithDetails;
pub use crate::entities::message::TokenUsage;

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
    pub chat: crate::entities::Chat,
    /// Initial messages if any
    pub messages: Vec<crate::entities::ChatMessage>,
}

/// Response for message creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageResponse {
    /// Created message
    pub message: crate::entities::ChatMessage,
    /// Updated chat information
    pub chat: crate::entities::Chat,
}

/// Response for attachment creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentResponse {
    /// Created attachment
    pub attachment: crate::entities::MessageAttachment,
    /// Message the attachment belongs to
    pub message: crate::entities::ChatMessage,
}

/// Response for invitation creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInviteResponse {
    /// Created invitation
    pub invite: crate::entities::ChatInvite,
    /// Chat the invitation is for
    pub chat: crate::entities::Chat,
}

/// Response for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated message
    pub message: crate::entities::ChatMessage,
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
    pub member: crate::entities::ChatMember,
    /// Chat information
    pub chat: crate::entities::Chat,
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