//! # Switchboard Gateway Crate
//!
//! This crate provides the API gateway layer for Switchboard, handling HTTP REST and WebSocket
//! connections and routing them to the appropriate domain services (users and chats).
//!
//! ## Architecture
//!
//! - **REST**: HTTP API endpoints with OpenAPI documentation
//! - **WebSocket**: Real-time communication handlers
//! - **State**: Shared application state for managing connections and services
//! - **Middleware**: Authentication, CORS, logging, and other cross-cutting concerns
//!
//! ## Usage
//!
//! ```rust
//! use switchboard_gateway::{GatewayState, create_router};
//!
//! let state = GatewayState::new(pool, jwt_config);
//! let app = create_router(state);
//!
//! axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
//!     .serve(app.into_make_service())
//!     .await
//!     .unwrap();
//! ```

pub mod rest;
pub mod websocket;
pub mod state;
pub mod middleware;
pub mod error;

// Re-export main types for convenience
pub use state::{GatewayState, create_gateway_state};
pub use error::{GatewayError, GatewayResult};
pub use middleware::auth_middleware;

// Legacy exports for compatibility
pub use create_router as build_router;
pub use GatewayState as AppState;

use axum::{
    Router,
    http::Method,
    middleware as axum_middleware,
};
use tower_http::cors::{CorsLayer, Any};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

/// Create the main application router with all routes
pub fn create_router(state: GatewayState) -> Router {
    let arc_state = Arc::new(state);
    let mut router = Router::new()
        // REST API routes
        .merge(rest::create_rest_routes().with_state(arc_state.clone()))
        // WebSocket routes
        .merge(websocket::create_websocket_routes().with_state(arc_state))
        // CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
                .allow_headers(Any)
                .allow_credentials(true)
        )
        // Logging middleware
        .layer(axum_middleware::from_fn(middleware::logging_middleware));

    // Add Swagger UI if in debug mode
    #[cfg(debug_assertions)]
    {
        #[derive(OpenApi)]
        #[openapi(
            paths(
                rest::auth::github_login,
                rest::auth::github_callback,
                rest::auth::dev_token,
                rest::auth::logout,
                rest::auth::me,
                rest::chat::list_chats,
                rest::chat::create_chat,
                rest::chat::get_chat,
                rest::chat::update_chat,
                rest::chat::delete_chat,
                rest::message::list_messages,
                rest::message::create_message,
                rest::message::get_message,
                rest::message::update_message,
                rest::message::delete_message,
                                rest::invite::list_invites,
                rest::invite::list_user_invites,
                rest::invite::create_invite,
                rest::invite::get_invite,
                rest::invite::respond_to_invite,
                rest::invite::delete_invite,
                rest::member::list_members,
                rest::member::get_member,
                rest::member::update_member_role,
                rest::member::remove_member,
                rest::member::leave_chat,
                rest::attachment::list_attachments,
                rest::attachment::list_message_attachments,
                rest::attachment::create_attachment,
                rest::attachment::get_attachment,
                rest::attachment::download_attachment,
                rest::attachment::delete_attachment,
            ),
            components(
                schemas(
                    rest::auth::GithubLoginResponse,
                    rest::auth::GithubLoginQuery,
                    rest::auth::GithubCallbackRequest,
                    rest::auth::SessionResponse,
                    rest::auth::UserResponse,
                    rest::auth::ErrorResponse,
                    rest::chat::ChatResponse,
                    rest::chat::CreateChatRequest,
                    rest::chat::UpdateChatRequest,
                    rest::chat::ListChatsQuery,
                    rest::chat::ErrorResponse,
                    rest::message::MessageResponse,
                    rest::message::CreateMessageRequest,
                    rest::message::UpdateMessageRequest,
                    rest::message::ListMessagesQuery,
                                        rest::message::ErrorResponse,
                    rest::invite::InviteResponse,
                    rest::invite::CreateInviteRequest,
                    rest::invite::ListInvitesQuery,
                    rest::invite::RespondToInviteRequest,
                    rest::invite::ErrorResponse,
                    rest::member::MemberResponse,
                    rest::member::UpdateMemberRoleRequest,
                    rest::member::ListMembersQuery,
                    rest::member::ErrorResponse,
                    rest::attachment::AttachmentResponse,
                    rest::attachment::CreateAttachmentRequest,
                    rest::attachment::ListAttachmentsQuery,
                    rest::attachment::ErrorResponse,
                )
            ),
            tags(
                (name = "auth", description = "Authentication endpoints"),
                (name = "chats", description = "Chat management"),
                (name = "messages", description = "Message management"),
                (name = "invites", description = "Chat invitations"),
                (name = "members", description = "Chat member management"),
                (name = "attachments", description = "File attachments"),
            )
        )]
        struct ApiDoc;

        router = router
            .merge(SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
            );
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}