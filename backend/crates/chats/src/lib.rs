//! # Switchboard Chats Crate
//!
//! This crate provides types and utilities for chat functionality in Switchboard.
//! It re-exports database entities and provides event types.
//!
//! ## Architecture
//!
//! - **Types**: Request/Response types and events
//! - **Utils**: Internal utilities
//! - **Entities**: Imported from database crate
//!
//! ## Usage
//!
//! ```rust
//! use switchboard_chats::{ChatEvent, CreateChatRequest};
//! ```

pub mod types {
    pub mod events;
}
pub mod utils;

// Re-export database types and repositories
pub use switchboard_database::{
    AttachmentRepository, AuthProvider, Chat, ChatError, ChatInvite, ChatMember, ChatMessage,
    ChatRepository, ChatResult, ChatStatus, ChatType, CreateAttachmentRequest, CreateChatRequest,
    CreateInviteRequest, CreateMemberRequest, CreateMessageRequest, InviteRepository, InviteStatus,
    MemberRepository, MemberRole, MessageAttachment, MessageRepository, MessageStatus,
    UpdateChatRequest, UpdateMessageRequest,
};

// Re-export sqlx for pool access
pub use sqlx::SqlitePool;

// Re-export event types
pub use types::events::ChatEvent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}
