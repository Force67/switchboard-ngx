mod docs;
mod error;
mod state;
mod util;

pub mod routes;

pub use docs::ApiDoc;
pub use error::ApiError;
pub use state::{AppState, OAuthStateStore};

use axum::{
    http::header::{AUTHORIZATION, CONTENT_TYPE},
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn build_router(state: AppState) -> Router {
    let docs = SwaggerUi::new("/docs").url("/docs/openapi.json", docs::ApiDoc::openapi());

    Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/api/auth/github/login", get(routes::auth::github_login))
        .route(
            "/api/auth/github/callback",
            post(routes::auth::github_callback),
        )
        .route("/api/auth/dev/token", get(routes::auth::dev_token))
        .route("/api/models", get(routes::models::list_models))
        .route("/api/chat", post(routes::chat::chat_completion))
        // Folder routes
        .route("/api/folders", get(routes::folders::list_folders))
        .route("/api/folders", post(routes::folders::create_folder))
        .route("/api/folders/:folder_id", get(routes::folders::get_folder))
        .route(
            "/api/folders/:folder_id",
            put(routes::folders::update_folder),
        )
        .route(
            "/api/folders/:folder_id",
            delete(routes::folders::delete_folder),
        )
        // Chat routes
        .route("/api/chats", get(routes::chats::list_chats))
        .route("/api/chats", post(routes::chats::create_chat))
        .route("/api/chats/:chat_id", get(routes::chats::get_chat))
        .route("/api/chats/:chat_id", put(routes::chats::update_chat))
        .route("/api/chats/:chat_id", delete(routes::chats::delete_chat))
        // Invite routes
        .route(
            "/api/chats/:chat_id/invites",
            get(routes::chats::list_invites),
        )
        .route(
            "/api/chats/:chat_id/invites",
            post(routes::chats::create_invite),
        )
        .route(
            "/api/invites/:invite_id/accept",
            post(routes::chats::accept_invite),
        )
        .route(
            "/api/invites/:invite_id/reject",
            post(routes::chats::reject_invite),
        )
        // Member routes
        .route(
            "/api/chats/:chat_id/members",
            get(routes::chats::list_members),
        )
        .route(
            "/api/chats/:chat_id/members/:member_user_id",
            put(routes::chats::update_member_role),
        )
        .route(
            "/api/chats/:chat_id/members/:member_user_id",
            delete(routes::chats::remove_member),
        )
        // Message routes
        .route(
            "/api/chats/:chat_id/messages",
            get(routes::messages::get_messages),
        )
        .route(
            "/api/chats/:chat_id/messages",
            post(routes::messages::create_message),
        )
        .route(
            "/api/chats/:chat_id/messages/:message_id",
            put(routes::messages::update_message),
        )
        .route(
            "/api/chats/:chat_id/messages/:message_id",
            delete(routes::messages::delete_message),
        )
        .route(
            "/api/chats/:chat_id/messages/:message_id/edits",
            get(routes::messages::get_message_edits),
        )
        // Attachment routes
        .route(
            "/api/chats/:chat_id/messages/:message_id/attachments",
            get(routes::attachments::get_message_attachments),
        )
        .route(
            "/api/chats/:chat_id/messages/:message_id/attachments",
            post(routes::attachments::create_message_attachment),
        )
        .route(
            "/api/chats/:chat_id/messages/:message_id/attachments/:attachment_id",
            delete(routes::attachments::delete_attachment),
        )
        // Notification routes
        .route(
            "/api/notifications",
            get(routes::notifications::get_notifications),
        )
        .route(
            "/api/notifications/unread-count",
            get(routes::notifications::get_unread_count),
        )
        .route(
            "/api/notifications/mark-all-read",
            post(routes::notifications::mark_all_read),
        )
        .route(
            "/api/notifications/:notification_id",
            put(routes::notifications::mark_notification_read),
        )
        .route(
            "/api/notifications/:notification_id",
            delete(routes::notifications::delete_notification),
        )
        // Permission routes
        .route(
            "/api/users/:user_id/permissions",
            get(routes::permissions::get_user_permissions),
        )
        .route(
            "/api/permissions/:resource_type/:resource_id",
            get(routes::permissions::get_resource_permissions),
        )
        .route(
            "/api/permissions/:resource_type/:resource_id",
            post(routes::permissions::grant_permission),
        )
        .route(
            "/api/permissions/:resource_type/:resource_id/:user_id",
            delete(routes::permissions::revoke_permission),
        )
        // WebSocket route
        .route("/ws", get(routes::websocket::websocket_handler))
        .merge(docs)
        .with_state(state)
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
}
