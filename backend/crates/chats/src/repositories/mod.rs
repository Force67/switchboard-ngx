//! Data access layer for the chat system.
//!
//! This module contains repository implementations that handle all
//! database operations. Repositories provide a clean interface
//! between the business logic and the database.

pub mod chat_repository;
pub mod message_repository;
pub mod attachment_repository;
pub mod member_repository;
pub mod invite_repository;

// Re-export all repositories
pub use chat_repository::ChatRepository;
pub use message_repository::MessageRepository;
pub use attachment_repository::AttachmentRepository;
pub use member_repository::MemberRepository;
pub use invite_repository::InviteRepository;