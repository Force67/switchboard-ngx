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
pub mod api;
pub mod types {
    pub mod events;
    pub mod requests;
    pub mod responses;
}
pub mod utils;

// Re-export database types and repositories
pub use switchboard_database::{
    ChatRepository, MessageRepository, AttachmentRepository, MemberRepository, InviteRepository,
    ChatResult, ChatError,
    Chat, ChatMessage, MessageAttachment, ChatMember, ChatInvite,
    CreateChatRequest, UpdateChatRequest, CreateMessageRequest, UpdateMessageRequest,
    CreateAttachmentRequest, CreateMemberRequest, CreateInviteRequest,
    ChatType, ChatStatus, MessageStatus, MemberRole, InviteStatus,
    AuthProvider,
};

// Re-export sqlx for pool access
pub use sqlx::SqlitePool;

// Re-export main types for convenience
pub use services::{
    ChatService, MessageService, AttachmentService, MemberService, InviteService, CompletionService,
};
pub use types::events::ChatEvent;
pub use api::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}