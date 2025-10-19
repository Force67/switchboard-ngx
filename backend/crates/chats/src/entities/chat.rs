use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::message::ChatMessage;

/// Represents a chat conversation in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// Owner user ID (nullable for backwards compatibility)
    pub user_id: Option<i64>,
    /// Optional folder for organization
    pub folder_id: Option<i64>,
    /// Chat title
    pub title: String,
    /// Type of chat (direct, group, system)
    pub chat_type: ChatType,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Chat type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatType {
    Direct,
    Group,
    System,
}

impl From<&str> for ChatType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "group" => ChatType::Group,
            "system" => ChatType::System,
            _ => ChatType::Direct,
        }
    }
}

impl From<ChatType> for String {
    fn from(chat_type: ChatType) -> Self {
        match chat_type {
            ChatType::Direct => "direct".to_string(),
            ChatType::Group => "group".to_string(),
            ChatType::System => "system".to_string(),
        }
    }
}

/// Request to create a new chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatRequest {
    /// Chat title
    pub title: String,
    /// Chat type
    pub chat_type: String,
    /// Optional folder ID
    pub folder_id: Option<String>,
    /// Initial messages (optional)
    pub messages: Vec<ChatMessage>,
}

/// Request to update a chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChatRequest {
    /// New title (optional)
    pub title: Option<String>,
    /// New folder ID (optional, empty string to remove)
    pub folder_id: Option<String>,
    /// Messages to add/update (optional)
    pub messages: Option<Vec<ChatMessage>>,
}

/// Chat with messages included (for UI hydration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWithMessages {
    pub id: i64,
    pub public_id: String,
    pub user_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub title: String,
    pub chat_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_group: bool,
    pub messages: Option<String>, // JSON string of messages
}

impl Chat {
    /// Create a new chat instance
    pub fn new(
        title: String,
        chat_type: ChatType,
        user_id: Option<i64>,
        folder_id: Option<i64>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            user_id,
            folder_id,
            title,
            chat_type,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Check if this is a group chat
    pub fn is_group(&self) -> bool {
        matches!(self.chat_type, ChatType::Group)
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validate chat data
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Chat title cannot be empty".to_string());
        }

        if self.title.len() > 255 {
            return Err("Chat title too long (max 255 characters)".to_string());
        }

        Ok(())
    }
}

impl CreateChatRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Chat title cannot be empty".to_string());
        }

        if self.title.len() > 255 {
            return Err("Chat title too long (max 255 characters)".to_string());
        }

        // Validate initial messages
        for (i, message) in self.messages.iter().enumerate() {
            if let Err(e) = message.validate() {
                return Err(format!("Invalid message at index {}: {}", i, e));
            }
        }

        Ok(())
    }
}

impl UpdateChatRequest {
    /// Validate the update request
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err("Chat title cannot be empty".to_string());
            }

            if title.len() > 255 {
                return Err("Chat title too long (max 255 characters)".to_string());
            }
        }

        // Validate messages if provided
        if let Some(ref messages) = self.messages {
            for (i, message) in messages.iter().enumerate() {
                if let Err(e) = message.validate() {
                    return Err(format!("Invalid message at index {}: {}", i, e));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_creation() {
        let chat = Chat::new(
            "Test Chat".to_string(),
            ChatType::Direct,
            Some(1),
            None,
        );

        assert_eq!(chat.title, "Test Chat");
        assert_eq!(chat.chat_type, ChatType::Direct);
        assert_eq!(chat.user_id, Some(1));
        assert!(!chat.is_group());
        assert!(chat.public_id.len() > 0);
    }

    #[test]
    fn test_chat_type_conversion() {
        assert_eq!(ChatType::from("direct"), ChatType::Direct);
        assert_eq!(ChatType::from("group"), ChatType::Group);
        assert_eq!(ChatType::from("system"), ChatType::System);
        assert_eq!(ChatType::from("unknown"), ChatType::Direct);

        assert_eq!(String::from(ChatType::Direct), "direct");
        assert_eq!(String::from(ChatType::Group), "group");
        assert_eq!(String::from(ChatType::System), "system");
    }

    #[test]
    fn test_chat_validation() {
        let mut chat = Chat::new(
            "Valid Chat".to_string(),
            ChatType::Direct,
            Some(1),
            None,
        );

        assert!(chat.validate().is_ok());

        chat.title = "".to_string();
        assert!(chat.validate().is_err());

        chat.title = "a".repeat(256);
        assert!(chat.validate().is_err());
    }

    #[test]
    fn test_create_chat_request_validation() {
        let valid_request = CreateChatRequest {
            title: "Valid Chat".to_string(),
            chat_type: "direct".to_string(),
            folder_id: None,
            messages: vec![],
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateChatRequest {
            title: "".to_string(),
            chat_type: "direct".to_string(),
            folder_id: None,
            messages: vec![],
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_update_chat_request_validation() {
        let valid_update = UpdateChatRequest {
            title: Some("New Title".to_string()),
            folder_id: None,
            messages: None,
        };
        assert!(valid_update.validate().is_ok());

        let invalid_update = UpdateChatRequest {
            title: Some("".to_string()),
            folder_id: None,
            messages: None,
        };
        assert!(invalid_update.validate().is_err());
    }
}