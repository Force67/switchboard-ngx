//! WebSocket endpoints for the gateway

pub mod user;
pub mod chat;

use axum::{
    extract::State,
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::state::GatewayState;

/// Create all WebSocket routes
pub fn create_websocket_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        // User WebSocket endpoints
        .route("/ws/user", get(user::user_websocket_handler))
        // Chat WebSocket endpoints
        .route("/ws/chat", get(chat::chat_websocket_handler))
}

// Re-export for convenience
pub use user::*;
pub use chat::*;