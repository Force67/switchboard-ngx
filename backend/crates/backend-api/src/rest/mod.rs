//! REST API endpoints for the backend

// Centralized DTOs - import these in handlers
pub mod models;

// Handler modules
pub mod attachment;
pub mod auth;
pub mod chat;
pub mod chat_completion;
pub mod folders;
pub mod health;
pub mod invite;
pub mod member;
pub mod message;
pub mod model;
pub mod notifications;
pub mod permissions;

use crate::state::GatewayState;
use axum::Router;
use std::sync::Arc;

/// Create all REST API routes
pub fn create_rest_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        // Authentication routes
        .merge(auth::create_auth_routes())
        // Health
        .merge(health::create_health_routes())
        // Protected routes
        .merge(
            Router::new()
                // Simple chat completion endpoint
                .merge(chat_completion::create_chat_completion_routes())
                // Chat routes
                .merge(chat::create_chat_routes())
                // Message routes
                .merge(message::create_message_routes())
                // Invite routes
                .merge(invite::create_invite_routes())
                // Folder routes
                .merge(folders::create_folder_routes())
                // Member routes
                .merge(member::create_member_routes())
                // Attachment routes
                .merge(attachment::create_attachment_routes())
                // Models routes
                .merge(model::create_models_routes())
                // Notifications routes
                .merge(notifications::create_notification_routes())
                // Permissions routes
                .merge(permissions::create_permission_routes()),
        )
}

// Re-export handler types for backwards compatibility with lib.rs OpenAPI registration
pub use attachment::{
    create_attachment, delete_attachment, download_attachment, get_attachment, list_attachments,
    list_message_attachments,
};
pub use auth::{dev_token, github_callback, github_login, logout, me};
pub use chat::{create_chat, delete_chat, get_chat, list_chats, update_chat};
pub use invite::{
    create_invite, delete_invite, get_invite, list_invites, list_user_invites, respond_to_invite,
};
pub use member::{get_member, leave_chat, list_members, remove_member, update_member_role};
pub use message::{create_message, delete_message, get_message, list_messages, update_message};
pub use model::list_models;
pub use notifications::{
    delete_notification, get_notifications, get_unread_count, mark_all_read, mark_notification_read,
};
pub use permissions::{
    create_permission, delete_permission, get_resource_permissions, get_user_permissions,
};
