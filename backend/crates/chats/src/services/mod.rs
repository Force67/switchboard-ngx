//! Business logic services for the chat system.
//!
//! This module contains all the service layer components that implement
//! the core business logic for chat operations. Services coordinate
//! between repositories and handle business rules.

pub mod attachment_service;
pub mod chat_service;
pub mod completion_service;
pub mod invite_service;
pub mod member_service;
pub mod message_service;

// Re-export all services
pub use attachment_service::AttachmentService;
pub use chat_service::ChatService;
pub use completion_service::CompletionService;
pub use invite_service::InviteService;
pub use member_service::MemberService;
pub use message_service::MessageService;
