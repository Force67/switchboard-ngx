//! Domain entities for the database layer
//!
//! Simplified entity definitions for use by the repository layer

pub mod attachment;
pub mod chat;
pub mod invite;
pub mod member;
pub mod message;
pub mod notification;
pub mod session;
pub mod settings;
pub mod user;

// Re-export all entity types
pub use attachment::{AttachmentType, CreateAttachmentRequest, MessageAttachment};
pub use chat::{Chat, ChatStatus, ChatType, CreateChatRequest, UpdateChatRequest};
pub use invite::{ChatInvite, CreateInviteRequest, InviteStatus};
pub use member::{ChatMember, CreateMemberRequest, MemberRole};
pub use message::{
    ChatMessage, CreateMessageRequest, MessageStatus, MessageType, UpdateMessageRequest,
};
pub use notification::{
    CreateNotificationRequest, Notification, NotificationPriority, NotificationType,
};
pub use session::{AuthProvider, AuthSession, CreateSessionRequest, LoginRequest, RegisterRequest};
pub use settings::{UserPreferences, UserSettings};
pub use user::{CreateUserRequest, UpdateUserRequest, User, UserRole, UserStatus};
