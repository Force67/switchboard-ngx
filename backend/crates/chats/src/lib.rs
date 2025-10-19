//! # Switchboard Chats Crate
//!
//! This crate provides the core business logic for chat functionality in Switchboard.
//! It contains domain entities, services, repositories, and types for managing
//! chats, messages, attachments, and real-time interactions.
//!
//! ## Architecture
//!
//! - **Entities**: Domain models (Chat, Message, Attachment, etc.)
//! - **Services**: Business logic layer
//! - **Repositories**: Data access layer
//! - **Types**: Shared types and interfaces
//! - **Utils**: Internal utilities
//!
//! ## Usage
//!
//! ```rust
//! use switchboard_chats::{ChatService, CreateChatRequest};
//!
//! let service = ChatService::new(pool);
//! let chat = service.create_chat(user_id, request).await?;
//! ```

pub mod entities;
pub mod repositories;
pub mod services;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use entities::{
    Chat, ChatType, ChatMessage, MessageAttachment, ChatMember, ChatInvite,
    CreateChatRequest, UpdateChatRequest, CreateMessageRequest, UpdateMessageRequest,
};
pub use services::{
    ChatService, MessageService, AttachmentService, MemberService, InviteService, CompletionService,
};
pub use types::{
    ChatError, ChatResult, ChatEvent, MemberRole, MessageRole, AttachmentType,
    ChatWithMessages, MemberWithUser, InviteWithDetails,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}