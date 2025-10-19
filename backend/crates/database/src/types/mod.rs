//! Shared types and result types for the database layer

pub mod errors;

// Re-export common types
pub use errors::{DatabaseError, UserError, ChatError, NotificationError, AuthError};

// Common result types
pub type DatabaseResult<T> = Result<T, DatabaseError>;
pub type UserResult<T> = Result<T, UserError>;
pub type ChatResult<T> = Result<T, ChatError>;
pub type NotificationResult<T> = Result<T, NotificationError>;

// Re-export request types from entities
pub use crate::entities::{
    CreateUserRequest, UpdateUserRequest, CreateNotificationRequest,
    CreateChatRequest, UpdateChatRequest, CreateMessageRequest, UpdateMessageRequest,
    CreateAttachmentRequest, CreateMemberRequest, CreateInviteRequest,
    CreateSessionRequest, LoginRequest, RegisterRequest
};

// Additional types that repositories expect
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateSettingsRequest {
    pub preferences: crate::entities::UserPreferences,
}

// Auth result type alias
pub type AuthResult<T> = Result<T, crate::types::errors::AuthError>;