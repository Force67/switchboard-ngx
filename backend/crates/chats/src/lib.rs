//! # Switchboard Chats Crate
//!
//! This crate provides the core business logic for chat functionality in Switchboard.
//! It contains services, types, and utilities for managing chats, messages, attachments,
//! and real-time interactions. All entities are now provided by the database crate.
//!
//! ## Architecture
//!
//! - **Services**: Business logic layer
//! - **Types**: Request/Response types and events
//! - **Utils**: Internal utilities
//! - **Entities**: Imported from database crate
//!
//! ## Usage
//!
//! ```rust
//! use switchboard_chats::{ChatService, CreateChatRequest};
//!
//! let service = ChatService::new(pool);
//! let chat = service.create_chat(user_id, request).await?;
//! ```

pub mod services;
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

// Re-export main types for convenience
pub use services::{
    AttachmentService, ChatService, CompletionService, InviteService, MemberService, MessageService,
};
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
