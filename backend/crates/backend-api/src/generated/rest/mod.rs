//! Generated REST handlers from crudder. DO NOT EDIT.

pub mod attachments;
pub mod auth;
pub mod chats;
pub mod folders;
pub mod health;
pub mod invites;
pub mod members;
pub mod messages;
pub mod models;
pub mod notifications;
pub mod permissions;
pub mod traits;

// Re-export route constructors
pub use attachments::create_attachment_routes;
pub use auth::create_auth_routes;
pub use chats::create_chat_routes;
pub use folders::create_folder_routes;
pub use health::create_health_routes;
pub use invites::create_invite_routes;
pub use members::create_member_routes;
pub use messages::create_message_routes;
pub use models::create_models_routes;
pub use notifications::create_notification_routes;
pub use permissions::create_permission_routes;
