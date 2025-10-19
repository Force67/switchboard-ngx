//! Repository for attachment data access operations.

use crate::entities::MessageAttachment;
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Repository for attachment database operations
pub struct AttachmentRepository {
    pool: SqlitePool,
}

impl AttachmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_message_id(&self, message_id: i64) -> ChatResult<Vec<MessageAttachment>> {
        todo!("Implement find_by_message_id")
    }

    pub async fn create(&self, attachment: &MessageAttachment) -> ChatResult<MessageAttachment> {
        todo!("Implement create")
    }

    pub async fn delete(&self, public_id: &str) -> ChatResult<()> {
        todo!("Implement delete")
    }
}