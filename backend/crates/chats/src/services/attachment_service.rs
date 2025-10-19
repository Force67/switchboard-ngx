//! Attachment service for managing file attachments.

use crate::entities::{MessageAttachment, CreateAttachmentRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Service for managing attachment operations
pub struct AttachmentService {
    pool: SqlitePool,
}

impl AttachmentService {
    /// Create a new attachment service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get attachments for a message
    pub async fn get_message_attachments(
        &self,
        chat_id: &str,
        message_id: &str,
        user_id: i64,
    ) -> ChatResult<Vec<MessageAttachment>> {
        todo!("Implement get_message_attachments")
    }

    /// Create a new attachment
    pub async fn create_attachment(
        &self,
        chat_id: &str,
        message_id: &str,
        user_id: i64,
        request: CreateAttachmentRequest,
    ) -> ChatResult<MessageAttachment> {
        todo!("Implement create_attachment")
    }

    /// Delete an attachment
    pub async fn delete_attachment(
        &self,
        chat_id: &str,
        message_id: &str,
        attachment_id: &str,
        user_id: i64,
    ) -> ChatResult<()> {
        todo!("Implement delete_attachment")
    }
}