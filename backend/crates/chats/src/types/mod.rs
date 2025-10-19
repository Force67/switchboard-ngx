//! Shared types and interfaces for the chat system.
//!
//! This module contains common types, error definitions, and interfaces
//! that are used across multiple modules in the crate.

pub mod errors;
pub mod requests;
pub mod responses;
pub mod events;

// Re-export common types
pub use errors::{ChatError, ChatResult};
pub use requests::*;
pub use responses::*;
pub use events::*;

// Common type aliases
pub type ChatId = String;
pub type MessageId = String;
pub type AttachmentId = String;
pub type InviteId = String;
pub type UserId = i64;

// Common enums
#[derive(Debug, Clone, PartialEq)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentType {
    Image,
    Document,
    Audio,
    Video,
    File,
}