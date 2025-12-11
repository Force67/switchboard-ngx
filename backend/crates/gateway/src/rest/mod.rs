//! REST API endpoints for the gateway

pub mod attachment;
pub mod auth;
pub mod chat;
pub mod health;
pub mod invite;
pub mod member;
pub mod message;

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
pub use attachment::*;
pub use auth::*;
pub use chat::*;
pub use invite::*;
pub use member::*;
pub use message::*;
