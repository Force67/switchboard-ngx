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
//! let state = GatewayState::new(pool, authenticator, jwt_config, None);
//! let app = create_router(state);
//!
//! axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
//!     .serve(app.into_make_service())
//!     .await
//!     .unwrap();
//! ```

pub mod error;
pub mod middleware;
pub mod rest;
pub mod services;
pub mod state;
pub mod websocket;

// Re-export main types for convenience
pub use error::{GatewayError, GatewayResult};
pub use middleware::auth_middleware;
pub use state::{create_gateway_state, GatewayState};

// Legacy exports for compatibility
pub use create_router as build_router;
pub use GatewayState as AppState;

use axum::{
    http::{header, Method},
    middleware as axum_middleware, Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main application router with all routes
pub fn create_router(state: GatewayState) -> Router {
    let arc_state = Arc::new(state);
    let api_routes = rest::create_rest_routes()
        .route_layer(axum_middleware::from_fn_with_state(
            arc_state.clone(),
            middleware::auth_middleware,
        ))
        .with_state(arc_state.clone());
    let mut router = Router::new()
        // REST API routes with versioning
        .nest("/api/v1", api_routes)
        // WebSocket routes
        .merge(websocket::create_websocket_routes().with_state(arc_state))
        // CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                ])
                .allow_headers([
                    header::ACCEPT,
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::ORIGIN,
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    header::ACCESS_CONTROL_REQUEST_METHOD,
                ])
                .allow_credentials(true),
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
                rest::models::list_models,
                rest::notifications::get_notifications,
                rest::notifications::get_unread_count,
                rest::notifications::mark_notification_read,
                rest::notifications::mark_all_read,
                rest::notifications::delete_notification,
                rest::permissions::get_user_permissions,
                rest::permissions::get_resource_permissions,
                rest::permissions::create_permission,
                rest::permissions::delete_permission,
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
                    rest::models::ModelsResponse,
                    rest::notifications::NotificationsResponse,
                    rest::notifications::UnreadCountResponse,
                    rest::notifications::BulkUpdateResponse,
                    rest::notifications::MarkNotificationReadRequest,
                    rest::permissions::PermissionsResponse,
                    rest::permissions::PermissionResponse,
                    rest::permissions::CreatePermissionRequest,
                    error::ErrorResponse,
                )
            ),
            tags(
                (name = "auth", description = "Authentication endpoints"),
                (name = "chats", description = "Chat management"),
                (name = "messages", description = "Message management"),
                (name = "invites", description = "Chat invitations"),
                (name = "members", description = "Chat member management"),
                (name = "attachments", description = "File attachments"),
                (name = "models", description = "AI model information"),
                (name = "notifications", description = "User notifications"),
                (name = "permissions", description = "Resource permissions"),
            )
        )]
        struct ApiDoc;

        router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
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
