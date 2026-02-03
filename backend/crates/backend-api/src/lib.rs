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
pub mod generated;
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

use axum::{extract::DefaultBodyLimit, middleware as axum_middleware, Router};
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
        // Reduce DoS risk from unbounded request bodies (multipart uploads, large JSON, etc.).
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
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
                // Auth
                generated::rest::auth::github_login,
                generated::rest::auth::github_callback,
                rest::dev::dev_token,
                generated::rest::auth::logout,
                generated::rest::auth::me,
                // Chats
                generated::rest::chats::list_chats,
                generated::rest::chats::create_chat,
                generated::rest::chats::get_chat,
                generated::rest::chats::update_chat,
                generated::rest::chats::delete_chat,
                // Messages
                generated::rest::messages::list_messages,
                generated::rest::messages::create_message,
                generated::rest::messages::get_message,
                generated::rest::messages::update_message,
                generated::rest::messages::delete_message,
                // Invites
                generated::rest::invites::list_invites,
                generated::rest::invites::list_user_invites,
                generated::rest::invites::create_invite,
                generated::rest::invites::get_invite,
                generated::rest::invites::respond_to_invite,
                generated::rest::invites::delete_invite,
                // Members
                generated::rest::members::list_members,
                generated::rest::members::get_member,
                generated::rest::members::update_member_role,
                generated::rest::members::remove_member,
                generated::rest::members::leave_chat,
                // Attachments
                generated::rest::attachments::list_attachments,
                generated::rest::attachments::list_message_attachments,
                generated::rest::attachments::create_attachment,
                generated::rest::attachments::get_attachment,
                generated::rest::attachments::delete_attachment,
                // Models
                generated::rest::models::list_models,
                // Notifications
                generated::rest::notifications::get_notifications,
                generated::rest::notifications::get_unread_count,
                generated::rest::notifications::mark_notification_read,
                generated::rest::notifications::mark_all_read,
                generated::rest::notifications::delete_notification,
                // Permissions
                generated::rest::permissions::get_user_permissions,
                generated::rest::permissions::get_resource_permissions,
                generated::rest::permissions::create_permission,
                generated::rest::permissions::delete_permission,
                // Folders
                generated::rest::folders::list_folders,
                generated::rest::folders::create_folder,
                generated::rest::folders::get_folder,
                generated::rest::folders::update_folder,
                generated::rest::folders::delete_folder,
                // Health
                generated::rest::health::health_check,
                // Chat completion (non-generated)
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
