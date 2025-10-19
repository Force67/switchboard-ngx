//! Request types for the chat system.

use serde::{Deserialize, Serialize};

// Re-export request types from database
pub use switchboard_database::{
    CreateChatRequest, UpdateChatRequest, CreateMessageRequest, UpdateMessageRequest,
    CreateAttachmentRequest, CreateInviteRequest,
};

/// Request for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Chat ID to generate completion for
    pub chat_id: String,
    /// User prompt
    pub prompt: String,
    /// AI model to use (optional)
    pub model: Option<String>,
    /// Attachment IDs to include in context
    pub attachments: Vec<String>,
    /// Stream response if true
    pub stream: bool,
    /// Temperature for generation (optional)
    pub temperature: Option<f32>,
    /// Max tokens to generate (optional)
    pub max_tokens: Option<u32>,
}

impl CompletionRequest {
    /// Validate the completion request
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        if self.prompt.len() > 100_000 {
            return Err("Prompt too long (max 100,000 characters)".to_string());
        }

        // Validate temperature if provided
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }

        // Validate max_tokens if provided
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 || max_tokens > 32_768 {
                return Err("Max tokens must be between 1 and 32768".to_string());
            }
        }

        Ok(())
    }
}

/// Request for listing chats with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListChatsRequest {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page
    pub limit: Option<u32>,
    /// Filter by chat type
    pub chat_type: Option<String>,
    /// Search query
    pub search: Option<String>,
}

impl Default for ListChatsRequest {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            chat_type: None,
            search: None,
        }
    }
}

/// Request for listing messages with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesRequest {
    /// Chat ID
    pub chat_id: String,
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Items per page
    pub limit: Option<u32>,
    /// Include message edits
    pub include_edits: Option<bool>,
}

impl Default for ListMessagesRequest {
    fn default() -> Self {
        Self {
            chat_id: String::new(), // Must be provided
            page: Some(1),
            limit: Some(50),
            include_edits: Some(false),
        }
    }
}