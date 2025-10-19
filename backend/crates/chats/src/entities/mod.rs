//! Domain entities for the chat system.
//!
//! This module contains all the core domain entities that represent the
//! chat system's data models. These are pure domain objects without
//! API-specific concerns.

pub mod chat;
pub mod message;
pub mod attachment;
pub mod member;
pub mod invite;

// Re-export all entity types
pub use chat::{Chat, ChatType, CreateChatRequest, UpdateChatRequest};
pub use message::{ChatMessage, MessageRole, CreateMessageRequest, UpdateMessageRequest};
pub use attachment::{MessageAttachment, AttachmentType, CreateAttachmentRequest};
pub use member::{ChatMember, MemberRole};
pub use invite::{ChatInvite, InviteStatus, CreateInviteRequest};