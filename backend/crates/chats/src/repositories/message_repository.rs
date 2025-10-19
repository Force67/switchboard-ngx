//! Repository for message data access operations.

use crate::entities::ChatMessage;
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Repository for message database operations
pub struct MessageRepository {
    pool: SqlitePool,
}

impl MessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_chat_id(&self, chat_id: i64) -> ChatResult<Vec<ChatMessage>> {
        todo!("Implement find_by_chat_id")
    }

    pub async fn create(&self, message: &ChatMessage) -> ChatResult<ChatMessage> {
        todo!("Implement create")
    }

    pub async fn update(&self, message: &ChatMessage) -> ChatResult<ChatMessage> {
        todo!("Implement update")
    }

    pub async fn delete(&self, public_id: &str) -> ChatResult<()> {
        todo!("Implement delete")
    }
}