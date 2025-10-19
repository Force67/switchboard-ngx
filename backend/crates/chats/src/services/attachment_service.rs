//! Attachment service for managing message attachments.

use switchboard_database::{MessageAttachment, CreateAttachmentRequest, AttachmentRepository, ChatResult};
use sqlx::SqlitePool;

/// Service for managing attachment operations
pub struct AttachmentService {
    attachment_repository: AttachmentRepository,
}

impl AttachmentService {
    /// Create a new attachment service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            attachment_repository: AttachmentRepository::new(pool),
        }
    }

    /// Get attachment by ID
    pub async fn get_attachment(&self, attachment_id: &str, user_id: i64) -> ChatResult<MessageAttachment> {
        todo!("Implement get_attachment")
    }

    /// Create a new attachment
    pub async fn create_attachment(
        &self,
        message_id: &str,
        user_id: i64,
        request: CreateAttachmentRequest,
    ) -> ChatResult<MessageAttachment> {
        todo!("Implement create_attachment")
    }

    /// Delete an attachment
    pub async fn delete_attachment(&self, attachment_id: &str, user_id: i64) -> ChatResult<()> {
        todo!("Implement delete_attachment")
    }
}