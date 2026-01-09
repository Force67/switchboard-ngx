//! Completion service for AI chat completions.
//!
//! Note: AI completions are primarily handled directly by the gateway layer
//! using the orchestrator. This service provides utility methods for
//! message-based completion workflows.

use sqlx::SqlitePool;
use switchboard_database::{
    ChatError, ChatMessage, ChatResult, CreateMessageRequest, MessageRepository, MessageType,
};

/// Service for managing AI chat completion operations
pub struct CompletionService {
    message_repository: MessageRepository,
}

impl CompletionService {
    /// Create a new completion service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            message_repository: MessageRepository::new(pool),
        }
    }

    /// Store a user prompt as a message in the chat
    pub async fn store_user_prompt(
        &self,
        chat_id: i64,
        chat_public_id: &str,
        user_id: i64,
        sender_public_id: &str,
        prompt: &str,
    ) -> ChatResult<ChatMessage> {
        let request = CreateMessageRequest {
            chat_id,
            chat_public_id: chat_public_id.to_string(),
            sender_id: user_id,
            sender_public_id: sender_public_id.to_string(),
            content: Some(prompt.to_string()),
            message_type: MessageType::Text,
            reply_to_public_id: None,
            thread_public_id: None,
        };
        self.message_repository.create(user_id, &request).await
    }

    /// Store an AI completion response as a message in the chat
    pub async fn store_completion_response(
        &self,
        chat_id: i64,
        chat_public_id: &str,
        assistant_user_id: i64,
        assistant_public_id: &str,
        response: &str,
    ) -> ChatResult<ChatMessage> {
        let request = CreateMessageRequest {
            chat_id,
            chat_public_id: chat_public_id.to_string(),
            sender_id: assistant_user_id,
            sender_public_id: assistant_public_id.to_string(),
            content: Some(response.to_string()),
            message_type: MessageType::Text,
            reply_to_public_id: None,
            thread_public_id: None,
        };
        self.message_repository
            .create(assistant_user_id, &request)
            .await
    }

    /// Generate AI completion for a chat
    /// Note: Actual completion should be done via the gateway's /api/v1/chat endpoint.
    /// This method is a placeholder for future service-layer completion handling.
    pub async fn generate_completion(
        &self,
        _chat_id: &str,
        _user_id: i64,
        _prompt: String,
        _model: Option<String>,
        _attachments: Vec<String>,
    ) -> ChatResult<ChatMessage> {
        Err(ChatError::ServiceUnavailable(
            "Completions should be requested via the gateway API".to_string(),
        ))
    }

    /// Stream AI completion response
    /// Note: Streaming completions are handled by the gateway layer.
    pub async fn stream_completion(
        &self,
        _chat_id: &str,
        _user_id: i64,
        _prompt: String,
        _model: Option<String>,
        _attachments: Vec<String>,
    ) -> ChatResult<futures::stream::Empty<ChatResult<String>>> {
        Err(ChatError::ServiceUnavailable(
            "Streaming completions should be requested via the gateway API".to_string(),
        ))
    }
}
