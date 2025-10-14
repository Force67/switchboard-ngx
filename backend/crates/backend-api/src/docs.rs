use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_check,
        crate::routes::auth::github_login,
        crate::routes::auth::github_callback,
        crate::routes::users::get_current_user,
        crate::routes::users::update_current_user,
        crate::routes::models::list_models,
        crate::routes::chat::chat_completion,
        crate::routes::folders::list_folders,
        crate::routes::folders::create_folder,
        crate::routes::folders::get_folder,
        crate::routes::folders::update_folder,
        crate::routes::folders::delete_folder,
        crate::routes::chats::list_chats,
        crate::routes::chats::create_chat,
        crate::routes::chats::get_chat,
        crate::routes::chats::update_chat,
        crate::routes::chats::delete_chat,
        crate::routes::chats::create_invite,
        crate::routes::chats::list_invites,
        crate::routes::chats::accept_invite,
        crate::routes::chats::reject_invite,
        crate::routes::chats::list_members,
        crate::routes::chats::update_member_role,
        crate::routes::chats::remove_member,
        crate::routes::messages::get_messages,
        crate::routes::messages::create_message,
        crate::routes::messages::update_message,
        crate::routes::messages::delete_message,
        crate::routes::messages::get_message_edits,
        crate::routes::attachments::get_message_attachments,
        crate::routes::attachments::create_message_attachment,
        crate::routes::attachments::delete_attachment,
        crate::routes::notifications::get_notifications,
        crate::routes::notifications::get_unread_count,
        crate::routes::notifications::mark_notification_read,
        crate::routes::notifications::mark_all_read,
        crate::routes::notifications::delete_notification,
        crate::routes::permissions::get_user_permissions,
        crate::routes::permissions::get_resource_permissions,
        crate::routes::permissions::grant_permission,
        crate::routes::permissions::revoke_permission,
        crate::routes::websocket::websocket_handler
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
            crate::routes::health::HealthResponse,
            crate::routes::auth::GithubLoginResponse,
            crate::routes::auth::GithubCallbackRequest,
            crate::routes::auth::SessionResponse,
            crate::routes::auth::UserResponse,
            crate::routes::users::UserProfileResponse,
            crate::routes::users::UpdateUserProfileRequest,
            crate::routes::chat::ChatCompletionForm,
            crate::routes::chat::ChatCompletionResponse,
            crate::routes::models::ModelsResponse,
            crate::routes::models::ModelSummary,
            crate::routes::models::ModelPricing,
            crate::routes::models::Folder,
            crate::routes::folders::FoldersResponse,
            crate::routes::folders::FolderResponse,
            crate::routes::models::Chat,
            crate::routes::models::User,
            crate::routes::models::Message,
            crate::routes::models::MessageEdit,
            crate::routes::models::MessageDeletion,
            crate::routes::models::MessageAttachment,
            crate::routes::models::Notification,
            crate::routes::models::Permission,
            crate::routes::models::CreateFolderRequest,
            crate::routes::models::UpdateFolderRequest,
            crate::routes::models::CreateChatRequest,
            crate::routes::models::UpdateChatRequest,
            crate::routes::models::ChatMessage,
            crate::routes::models::TokenUsage,
            crate::routes::models::ChatInvite,
            crate::routes::models::CreateInviteRequest,
            crate::routes::models::InvitesResponse,
            crate::routes::models::InviteResponse,
            crate::routes::models::ChatMember,
            crate::routes::models::UpdateMemberRoleRequest,
            crate::routes::models::MembersResponse,
            crate::routes::models::MemberResponse,
            crate::routes::models::CreateMessageRequest,
            crate::routes::models::UpdateMessageRequest,
            crate::routes::models::CreateAttachmentRequest,
            crate::routes::models::CreateNotificationRequest,
            crate::routes::models::MarkNotificationReadRequest,
            crate::routes::models::NotificationsResponse,
            crate::routes::models::NotificationResponse,
            crate::routes::models::CreatePermissionRequest,
            crate::routes::models::PermissionsResponse,
            crate::routes::models::PermissionResponse,
            crate::routes::models::MessageEditsResponse,
            crate::routes::models::AttachmentResponse,
            crate::routes::models::AttachmentsResponse,
            crate::routes::models::MessageResponse,
            crate::routes::models::MessagesResponse,
            crate::routes::chats::ChatsResponse,
            crate::routes::chats::ChatDetailResponse,
            crate::routes::notifications::UnreadCountResponse,
            crate::routes::notifications::BulkUpdateResponse
        )
    ),
    tags(
        (name = "Health", description = "Service health endpoints"),
        (name = "Auth", description = "Authentication and session management"),
        (name = "Users", description = "User profile management"),
        (name = "Models", description = "Model catalogue"),
        (name = "Chat", description = "LLM chat completions"),
        (name = "Folders", description = "Folder management"),
        (name = "Chats", description = "Chat workspace operations"),
        (name = "Chat Invites", description = "Inviting users to chats"),
        (name = "Chat Members", description = "Managing chat membership"),
        (name = "Messages", description = "Chat message CRUD and history"),
        (name = "Attachments", description = "Message attachment operations"),
        (name = "Notifications", description = "User notifications"),
        (name = "Permissions", description = "Resource permission management"),
        (name = "WebSocket", description = "Realtime updates stream")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        let schemes = &mut components.security_schemes;

        let mut scheme = SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer));
        if let SecurityScheme::Http(http) = &mut scheme {
            http.bearer_format = Some("Bearer".to_string());
        }

        schemes.insert("bearerAuth".to_string(), scheme);
    }
}
