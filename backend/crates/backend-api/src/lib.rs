//! # Switchboard Backend API
//!
//! This crate provides the HTTP REST and WebSocket API layer for Switchboard,
//! handling connections and routing them to the appropriate domain services (users and chats).
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
//! ```rust,ignore
//! use switchboard_backend_api::{ApiState, create_router};
//!
//! let state = ApiState::new(pool, authenticator, jwt_config, None);
//! let app = create_router(state);
//!
//! axum::serve(listener, app).await.unwrap();
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

use axum::{middleware as axum_middleware, Router};
use std::sync::Arc;
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
        // Logging middleware
        .layer(axum_middleware::from_fn(middleware::logging_middleware));

    // Dev-only CORS: allow all localhost ports to hit the API from a browser.
    //
    // Production should generally be same-origin behind the reverse proxy / frontend host.
    #[cfg(debug_assertions)]
    {
        router = router.layer(middleware::create_dev_cors_middleware());
    }

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
                rest::model::list_models,
                rest::notifications::get_notifications,
                rest::notifications::get_unread_count,
                rest::notifications::mark_notification_read,
                rest::notifications::mark_all_read,
                rest::notifications::delete_notification,
                rest::permissions::get_user_permissions,
                rest::permissions::get_resource_permissions,
                rest::permissions::create_permission,
                rest::permissions::delete_permission,
                rest::folders::list_folders,
                rest::folders::create_folder,
                rest::folders::get_folder,
                rest::folders::update_folder,
                rest::folders::delete_folder,
                rest::health::health_check,
                rest::chat_completion::chat_completion,
            ),
            components(
                schemas(
                    // Auth models
                    rest::models::GithubLoginResponse,
                    rest::models::GithubLoginQuery,
                    rest::models::GithubCallbackRequest,
                    rest::models::SessionResponse,
                    rest::models::UserResponse,
                    // Chat models
                    rest::models::ChatResponse,
                    rest::models::ChatsResponse,
                    rest::models::CreateChatRequest,
                    rest::models::UpdateChatRequest,
                    rest::models::ListChatsQuery,
                    // Message models
                    rest::models::MessageResponse,
                    rest::models::MessagesResponse,
                    rest::models::MessageSenderResponse,
                    rest::models::CreateMessageRequest,
                    rest::models::UpdateMessageRequest,
                    rest::models::ListMessagesQuery,
                    // Invite models
                    rest::models::InviteResponse,
                    rest::models::InvitesResponse,
                    rest::models::CreateInviteRequest,
                    rest::models::ListInvitesQuery,
                    rest::models::RespondToInviteRequest,
                    // Member models
                    rest::models::MemberResponse,
                    rest::models::MembersResponse,
                    rest::models::ChatMemberResponse,
                    rest::models::MemberUserResponse,
                    rest::models::UpdateMemberRoleRequest,
                    rest::models::ListMembersQuery,
                    // Attachment models
                    rest::models::AttachmentResponse,
                    rest::models::AttachmentsResponse,
                    rest::models::MessageAttachmentResponse,
                    rest::models::AttachmentUploaderResponse,
                    rest::models::CreateAttachmentRequest,
                    rest::models::ListAttachmentsQuery,
                    // Folder models
                    rest::models::Folder,
                    rest::models::FolderResponse,
                    rest::models::FoldersResponse,
                    rest::models::CreateFolderRequest,
                    rest::models::UpdateFolderRequest,
                    // Model models
                    rest::models::ModelsResponse,
                    rest::models::ModelSummary,
                    // Notification models
                    rest::models::Notification,
                    rest::models::NotificationResponse,
                    rest::models::NotificationsResponse,
                    rest::models::UnreadCountResponse,
                    rest::models::BulkUpdateResponse,
                    rest::models::MarkNotificationReadRequest,
                    rest::models::ListNotificationsQuery,
                    // Permission models
                    rest::models::Permission,
                    rest::models::PermissionResponse,
                    rest::models::PermissionsResponse,
                    rest::models::CreatePermissionRequest,
                    // Chat completion models
                    rest::models::ChatCompletionResponse,
                    rest::models::ChatCompletionForm,
                    rest::models::TokenUsage,
                    rest::models::WebSearchSource,
                    // Health models
                    rest::models::HealthResponse,
                    // Common
                    rest::models::PaginationQuery,
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
