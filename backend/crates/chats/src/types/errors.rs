//! Error types for the chat system.

use thiserror::Error;

/// Result type alias for chat operations
pub type ChatResult<T> = Result<T, ChatError>;

/// Main error type for the chat system
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Chat not found: {id}")]
    ChatNotFound { id: String },

    #[error("Message not found: {id}")]
    MessageNotFound { id: String },

    #[error("Attachment not found: {id}")]
    AttachmentNotFound { id: String },

    #[error("Invite not found: {id}")]
    InviteNotFound { id: String },

    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Permission denied: {reason}")]
    PermissionDenied { reason: String },

    #[error("Invite expired")]
    InviteExpired,

    #[error("Invite already responded")]
    InviteAlreadyResponded,

    #[error("File upload error: {message}")]
    FileUpload { message: String },

    #[error("AI completion error: {message}")]
    CompletionError { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },
}

impl ChatError {
    /// Create a not found error for chats
    pub fn chat_not_found(id: impl Into<String>) -> Self {
        Self::ChatNotFound { id: id.into() }
    }

    /// Create a not found error for messages
    pub fn message_not_found(id: impl Into<String>) -> Self {
        Self::MessageNotFound { id: id.into() }
    }

    /// Create a not found error for attachments
    pub fn attachment_not_found(id: impl Into<String>) -> Self {
        Self::AttachmentNotFound { id: id.into() }
    }

    /// Create a not found error for invites
    pub fn invite_not_found(id: impl Into<String>) -> Self {
        Self::InviteNotFound { id: id.into() }
    }

    /// Create an access denied error
    pub fn access_denied(reason: impl Into<String>) -> Self {
        Self::AccessDenied { reason: reason.into() }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }

    /// Create a permission denied error
    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied { reason: reason.into() }
    }

    /// Create a file upload error
    pub fn file_upload(message: impl Into<String>) -> Self {
        Self::FileUpload { message: message.into() }
    }

    /// Create an AI completion error
    pub fn completion_error(message: impl Into<String>) -> Self {
        Self::CompletionError { message: message.into() }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration { message: message.into() }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }
}

// Conversion from common error types
impl From<uuid::Error> for ChatError {
    fn from(err: uuid::Error) -> Self {
        Self::Internal {
            message: format!("UUID error: {}", err),
        }
    }
}

impl From<chrono::ParseError> for ChatError {
    fn from(err: chrono::ParseError) -> Self {
        Self::Validation {
            message: format!("Date parsing error: {}", err),
        }
    }
}

impl From<serde_json::Error> for ChatError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal {
            message: format!("JSON serialization error: {}", err),
        }
    }
}