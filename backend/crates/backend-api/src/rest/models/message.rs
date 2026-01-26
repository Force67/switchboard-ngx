//! Message-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::attachment::MessageAttachmentResponse;

/// Response containing a list of messages
#[derive(Debug, Serialize, ToSchema)]
pub struct MessagesResponse {
    pub messages: Vec<MessageResponse>,
}

/// Response containing message details
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: Option<String>,
    pub message_type: String,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub edited: bool,
    pub deleted: bool,
    pub sender: MessageSenderResponse,
    pub attachments: Vec<MessageAttachmentResponse>,
}

/// Sender information embedded in message response
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageSenderResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for creating a message
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: Option<String>,
    pub message_type: Option<String>,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
}

/// Request body for updating a message
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: Option<String>,
}

/// Query parameters for listing messages
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// Message ID to get messages before
    pub before: Option<String>,
    /// Message ID to get messages after
    pub after: Option<String>,
    /// Filter by thread
    pub thread_id: Option<String>,
}

impl From<switchboard_database::ChatMessage> for MessageResponse {
    fn from(message: switchboard_database::ChatMessage) -> Self {
        Self {
            id: message.public_id,
            chat_id: message.chat_public_id,
            sender_id: message.sender_public_id.clone(),
            content: message.content.clone(),
            message_type: message.message_type.to_string(),
            reply_to: message.reply_to_public_id.clone(),
            thread_id: message.thread_public_id.clone(),
            created_at: message.created_at.clone(),
            updated_at: message.updated_at.clone(),
            edited: message.updated_at.is_some(),
            deleted: message.deleted_at.is_some(),
            sender: MessageSenderResponse {
                id: message.sender_public_id,
                display_name: message.sender_display_name,
                avatar_url: message.sender_avatar_url,
            },
            attachments: vec![], // Will be populated by the service
        }
    }
}
