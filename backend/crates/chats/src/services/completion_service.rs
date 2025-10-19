//! Completion service for AI chat completions.

use crate::entities::{ChatMessage, CreateMessageRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Service for managing AI chat completion operations
pub struct CompletionService {
    pool: SqlitePool,
}

impl CompletionService {
    /// Create a new completion service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Generate AI completion for a chat
    pub async fn generate_completion(
        &self,
        chat_id: &str,
        user_id: i64,
        prompt: String,
        model: Option<String>,
        attachments: Vec<String>,
    ) -> ChatResult<ChatMessage> {
        todo!("Implement generate_completion")
    }

    /// Stream AI completion response
    pub async fn stream_completion(
        &self,
        chat_id: &str,
        user_id: i64,
        prompt: String,
        model: Option<String>,
        attachments: Vec<String>,
    ) -> ChatResult<futures::stream::Empty<ChatResult<String>>> {
        todo!("Implement stream_completion")
    }
}