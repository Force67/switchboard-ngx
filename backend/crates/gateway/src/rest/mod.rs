//! REST API endpoints for the gateway

pub mod auth;
pub mod chat;
pub mod message;
pub mod invite;
pub mod member;
pub mod attachment;

use axum::Router;
use crate::state::GatewayState;
use std::sync::Arc;

/// Create all REST API routes
pub fn create_rest_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        // Authentication routes
        .merge(auth::create_auth_routes())
        // Chat routes
        .merge(chat::create_chat_routes())
        // Message routes
        .merge(message::create_message_routes())
        // Invite routes
        .merge(invite::create_invite_routes())
        // Member routes
        .merge(member::create_member_routes())
        // Attachment routes
        .merge(attachment::create_attachment_routes())
}

// Re-export for convenience
pub use auth::*;
pub use chat::*;
pub use message::*;
pub use invite::*;
pub use member::*;
pub use attachment::*;