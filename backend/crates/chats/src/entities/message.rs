use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a message within a chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// Chat ID this message belongs to
    pub chat_id: i64,
    /// User ID who sent the message
    pub user_id: i64,
    /// Message content
    pub content: String,
    /// Role of the message sender
    pub role: MessageRole,
    /// AI model used (if applicable)
    pub model: Option<String>,
    /// Type of message
    pub message_type: String,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// Reasoning content (for AI responses)
    pub reasoning: Option<String>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Message role enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl From<&str> for MessageRole {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "tool" => MessageRole::Tool,
            _ => MessageRole::User,
        }
    }
}

impl From<MessageRole> for String {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::User => "user".to_string(),
            MessageRole::Assistant => "assistant".to_string(),
            MessageRole::System => "system".to_string(),
            MessageRole::Tool => "tool".to_string(),
        }
    }
}

/// Token usage information for AI responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Request to create a new message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    /// Message content
    pub content: String,
    /// Message role
    pub role: String,
    /// AI model (if applicable)
    pub model: Option<String>,
    /// Token usage (for AI responses)
    pub usage: Option<TokenUsage>,
    /// Reasoning content (for AI responses)
    pub reasoning: Option<String>,
}

/// Request to update a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessageRequest {
    /// Updated content
    pub content: String,
    /// Updated role
    pub role: Option<String>,
    /// Updated model
    pub model: Option<String>,
    /// Updated reasoning
    pub reasoning: Option<String>,
}

impl ChatMessage {
    /// Create a new message instance
    pub fn new(
        chat_id: i64,
        user_id: i64,
        content: String,
        role: MessageRole,
        model: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            chat_id,
            user_id,
            content,
            role,
            model,
            message_type: if matches!(role, MessageRole::System) {
                "system".to_string()
            } else {
                "text".to_string()
            },
            usage: None,
            reasoning: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Update the message content and timestamp
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Check if this is a system message
    pub fn is_system(&self) -> bool {
        matches!(self.role, MessageRole::System)
    }

    /// Check if this is an AI assistant message
    pub fn is_assistant(&self) -> bool {
        matches!(self.role, MessageRole::Assistant)
    }

    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        matches!(self.role, MessageRole::User)
    }

    /// Get the effective message length (content + reasoning)
    pub fn total_length(&self) -> usize {
        let content_len = self.content.len();
        let reasoning_len = self.reasoning.as_ref().map_or(0, |r| r.len());
        content_len + reasoning_len
    }

    /// Validate message data
    pub fn validate(&self) -> Result<(), String> {
        if self.content.trim().is_empty() && !self.is_system() {
            return Err("Message content cannot be empty".to_string());
        }

        if self.content.len() > 100_000 {
            return Err("Message content too long (max 100,000 characters)".to_string());
        }

        // Validate reasoning length if present
        if let Some(ref reasoning) = self.reasoning {
            if reasoning.len() > 50_000 {
                return Err("Reasoning content too long (max 50,000 characters)".to_string());
            }
        }

        Ok(())
    }
}

impl CreateMessageRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), String> {
        if self.content.trim().is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if self.content.len() > 100_000 {
            return Err("Message content too long (max 100,000 characters)".to_string());
        }

        // Validate role
        let _role: MessageRole = self.role.as_str().into();

        // Validate reasoning length if present
        if let Some(ref reasoning) = self.reasoning {
            if reasoning.len() > 50_000 {
                return Err("Reasoning content too long (max 50,000 characters)".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateMessageRequest {
    /// Validate the update request
    pub fn validate(&self) -> Result<(), String> {
        if self.content.trim().is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if self.content.len() > 100_000 {
            return Err("Message content too long (max 100,000 characters)".to_string());
        }

        // Validate reasoning length if present
        if let Some(ref reasoning) = self.reasoning {
            if reasoning.len() > 50_000 {
                return Err("Reasoning content too long (max 50,000 characters)".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = ChatMessage::new(
            1,
            1,
            "Hello, world!".to_string(),
            MessageRole::User,
            None,
        );

        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.chat_id, 1);
        assert_eq!(message.user_id, 1);
        assert!(!message.is_system());
        assert!(message.is_user());
        assert!(!message.is_assistant());
    }

    #[test]
    fn test_system_message_creation() {
        let message = ChatMessage::new(
            1,
            1,
            "You are a helpful assistant".to_string(),
            MessageRole::System,
            None,
        );

        assert_eq!(message.message_type, "system");
        assert!(message.is_system());
        assert!(!message.is_user());
        assert!(!message.is_assistant());
    }

    #[test]
    fn test_message_role_conversion() {
        assert_eq!(MessageRole::from("user"), MessageRole::User);
        assert_eq!(MessageRole::from("assistant"), MessageRole::Assistant);
        assert_eq!(MessageRole::from("system"), MessageRole::System);
        assert_eq!(MessageRole::from("tool"), MessageRole::Tool);
        assert_eq!(MessageRole::from("unknown"), MessageRole::User);

        assert_eq!(String::from(MessageRole::User), "user");
        assert_eq!(String::from(MessageRole::Assistant), "assistant");
        assert_eq!(String::from(MessageRole::System), "system");
        assert_eq!(String::from(MessageRole::Tool), "tool");
    }

    #[test]
    fn test_message_validation() {
        let mut message = ChatMessage::new(
            1,
            1,
            "Valid message".to_string(),
            MessageRole::User,
            None,
        );

        assert!(message.validate().is_ok());

        message.content = "".to_string();
        assert!(message.validate().is_err());

        message.content = "a".repeat(100_001);
        assert!(message.validate().is_err());
    }

    #[test]
    fn test_system_message_validation() {
        let message = ChatMessage::new(
            1,
            1,
            "".to_string(), // Empty content allowed for system messages
            MessageRole::System,
            None,
        );

        assert!(message.validate().is_ok());
    }

    #[test]
    fn test_create_message_request_validation() {
        let valid_request = CreateMessageRequest {
            content: "Hello".to_string(),
            role: "user".to_string(),
            model: None,
            usage: None,
            reasoning: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateMessageRequest {
            content: "".to_string(),
            role: "user".to_string(),
            model: None,
            usage: None,
            reasoning: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_total_length() {
        let mut message = ChatMessage::new(
            1,
            1,
            "Hello".to_string(),
            MessageRole::Assistant,
            None,
        );

        assert_eq!(message.total_length(), 5);

        message.reasoning = Some("Thinking...".to_string());
        assert_eq!(message.total_length(), 5 + 11); // 5 content + 11 reasoning
    }

    #[test]
    fn test_update_content() {
        let mut message = ChatMessage::new(
            1,
            1,
            "Original".to_string(),
            MessageRole::User,
            None,
        );

        let original_updated_at = message.updated_at.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        message.update_content("Updated".to_string());

        assert_eq!(message.content, "Updated");
        assert_ne!(message.updated_at, original_updated_at);
    }
}