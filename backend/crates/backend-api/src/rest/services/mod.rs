//! Service trait implementations
//!
//! This module contains implementations of the generated service traits.
//! Each service struct holds a reference to the GatewayState and implements
//! the corresponding trait from `crate::generated::rest::traits`.

mod members;
mod chats;
mod messages;
mod folders;
mod attachments;
mod invites;
mod notifications;
mod permissions;
mod auth;
mod health;
mod models;

pub use members::MembersServiceImpl;
pub use chats::ChatsServiceImpl;
pub use messages::MessagesServiceImpl;
pub use folders::FoldersServiceImpl;
pub use attachments::AttachmentsServiceImpl;
pub use invites::InvitesServiceImpl;
pub use notifications::NotificationsServiceImpl;
pub use permissions::PermissionsServiceImpl;
pub use auth::AuthServiceImpl;
pub use health::HealthServiceImpl;
pub use models::ModelsServiceImpl;
