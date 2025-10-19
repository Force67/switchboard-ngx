//! Domain entities for the database layer
//!
//! Simplified entity definitions for use by the repository layer

pub mod user;
pub mod chat;
pub mod message;
pub mod attachment;
pub mod member;
pub mod invite;
pub mod notification;
pub mod session;
pub mod settings;

// Re-export all entity types
pub use user::{User, CreateUserRequest, UpdateUserRequest, UserStatus, UserRole};
pub use chat::{Chat, CreateChatRequest, UpdateChatRequest, ChatType, ChatStatus};
pub use message::{ChatMessage, CreateMessageRequest, UpdateMessageRequest, MessageStatus};
pub use attachment::{MessageAttachment, CreateAttachmentRequest};
pub use member::{ChatMember, CreateMemberRequest, MemberRole};
pub use invite::{ChatInvite, CreateInviteRequest, InviteStatus};
pub use notification::{Notification, CreateNotificationRequest, NotificationType, NotificationPriority};
pub use session::{AuthSession, CreateSessionRequest, LoginRequest, RegisterRequest, AuthProvider};
pub use settings::{UserSettings, UserPreferences};