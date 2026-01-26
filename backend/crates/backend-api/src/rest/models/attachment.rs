//! Attachment-related request and response types

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Response containing a list of attachments
#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentsResponse {
    pub attachments: Vec<AttachmentResponse>,
}

/// Full attachment response with uploader information
#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub id: String,
    pub message_id: String,
    pub chat_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
    pub uploader: AttachmentUploaderResponse,
}

/// Simplified attachment response for embedding in messages
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct MessageAttachmentResponse {
    pub id: String,
    pub message_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
}

/// Uploader information embedded in attachment response
#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentUploaderResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for creating an attachment
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    /// Base64 encoded file data
    pub file_data: String,
}

/// Query parameters for listing attachments
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListAttachmentsQuery {
    /// Filter by message ID
    pub message_id: Option<String>,
    /// Filter by file type
    pub file_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<switchboard_database::MessageAttachment> for AttachmentResponse {
    fn from(attachment: switchboard_database::MessageAttachment) -> Self {
        Self {
            id: attachment.public_id,
            message_id: attachment.message_public_id,
            chat_id: attachment.chat_public_id,
            file_name: attachment.file_name,
            file_type: attachment.file_type.to_string(),
            file_size: attachment.file_size,
            file_url: attachment.file_url,
            created_at: attachment.created_at,
            uploader: AttachmentUploaderResponse {
                id: attachment.uploader_public_id,
                display_name: attachment.uploader_display_name,
                avatar_url: attachment.uploader_avatar_url,
            },
        }
    }
}
