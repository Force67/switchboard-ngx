//! Attachment entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: i64,
    pub public_id: String,
    pub message_id: i64,
    pub filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentRequest {
    pub message_id: i64,
    pub filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_path: String,
}