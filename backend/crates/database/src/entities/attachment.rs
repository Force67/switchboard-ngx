//! Attachment entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: i64,
    pub public_id: String,
    pub message_id: i64,
    pub message_public_id: String,
    pub chat_id: i64,
    pub chat_public_id: String,
    pub file_name: String,
    pub file_type: AttachmentType,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
    pub uploader_id: i64,
    pub uploader_public_id: String,
    pub uploader_display_name: Option<String>,
    pub uploader_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentRequest {
    pub message_id: i64,
    pub message_public_id: String,
    pub uploader_id: i64,
    pub uploader_public_id: String,
    pub file_name: String,
    pub file_type: AttachmentType,
    pub file_size: i64,
    pub file_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    Video,
    Audio,
    Document,
    Other,
}

impl AttachmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttachmentType::Image => "image",
            AttachmentType::Video => "video",
            AttachmentType::Audio => "audio",
            AttachmentType::Document => "document",
            AttachmentType::Other => "other",
        }
    }
}

impl From<&str> for AttachmentType {
    fn from(s: &str) -> Self {
        match s {
            "image" => AttachmentType::Image,
            "video" => AttachmentType::Video,
            "audio" => AttachmentType::Audio,
            "document" => AttachmentType::Document,
            _ => AttachmentType::Other,
        }
    }
}

impl ToString for AttachmentType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}