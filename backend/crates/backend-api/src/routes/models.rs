use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use switchboard_orchestrator::OpenRouterModelSummary;
use utoipa::ToSchema;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelsResponse {
    #[schema(value_type = Vec<ModelSummary>)]
    pub models: Vec<OpenRouterModelSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelSummary {
    pub id: String,
    pub label: String,
    #[schema(nullable)]
    pub description: Option<String>,
    #[schema(nullable)]
    pub pricing: Option<ModelPricing>,
    #[schema(default)]
    pub supports_reasoning: bool,
    #[schema(default)]
    pub supports_images: bool,
    #[schema(default)]
    pub supports_tools: bool,
    #[schema(default)]
    pub supports_agents: bool,
    #[schema(default)]
    pub supports_function_calling: bool,
    #[schema(default)]
    pub supports_vision: bool,
    #[schema(default)]
    pub supports_tool_use: bool,
    #[schema(default)]
    pub supports_structured_outputs: bool,
    #[schema(default)]
    pub supports_streaming: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModelPricing {
    #[schema(nullable)]
    pub input: Option<f64>,
    #[schema(nullable)]
    pub output: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Folder {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<i64>,
    pub collapsed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Chat {
    pub id: i64,
    pub public_id: String,
    pub user_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub title: String,
    pub chat_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct User {
    pub id: i64,
    pub public_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Message {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub user_id: i64,
    pub content: String,
    pub role: String,
    pub model: Option<String>,
    pub message_type: String,
    pub thread_id: Option<i64>,
    pub reply_to_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct MessageEdit {
    pub id: i64,
    pub message_id: i64,
    pub edited_by_user_id: i64,
    pub old_content: String,
    pub new_content: String,
    pub edited_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct MessageDeletion {
    pub id: i64,
    pub message_id: i64,
    pub deleted_by_user_id: i64,
    pub reason: Option<String>,
    pub deleted_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct MessageAttachment {
    pub id: i64,
    pub message_id: i64,
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub file_size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    pub r#type: String, // "type" is a reserved keyword in Rust
    pub title: String,
    pub body: String,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct Permission {
    pub id: i64,
    pub user_id: i64,
    pub resource_type: String,
    pub resource_id: i64,
    pub permission_level: String,
    pub granted_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>, // public_id
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub collapsed: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChatRequest {
    pub title: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    pub folder_id: Option<String>, // public_id
    #[serde(default = "default_chat_type")]
    pub chat_type: String,
}

fn default_chat_type() -> String {
    "direct".to_string()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
    pub messages: Option<Vec<ChatMessage>>,
    pub folder_id: Option<String>, // public_id
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub usage: Option<TokenUsage>,
    pub reasoning: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct ChatInvite {
    pub id: i64,
    pub chat_id: i64,
    pub inviter_id: i64,
    pub invitee_email: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitesResponse {
    pub invites: Vec<ChatInvite>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub invite: ChatInvite,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone)]
pub struct ChatMember {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MembersResponse {
    pub members: Vec<ChatMember>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MemberResponse {
    pub member: ChatMember,
}

// New DTOs for message features
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: String,
    pub role: String,
    pub model: Option<String>,
    pub message_type: Option<String>,
    pub thread_id: Option<String>,   // public_id
    pub reply_to_id: Option<String>, // public_id
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub file_size_bytes: i64,
}

// Notification DTOs
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNotificationRequest {
    pub user_id: String, // public_id
    pub r#type: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkNotificationReadRequest {
    pub read: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationsResponse {
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub notification: Notification,
}

// Permission DTOs
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePermissionRequest {
    pub user_id: String, // public_id
    pub resource_type: String,
    pub resource_id: String, // public_id
    pub permission_level: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionsResponse {
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub permission: Permission,
}

// Message edit history
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageEditsResponse {
    pub edits: Vec<MessageEdit>,
}

// Attachment response structs
#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub attachment: MessageAttachment,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentsResponse {
    pub attachments: Vec<MessageAttachment>,
}

// Message response structs
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: Message,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

#[utoipa::path(
    get,
    path = "/api/models",
    tag = "Models",
    responses(
        (status = 200, description = "List available language models", body = ModelsResponse),
        (status = 503, description = "Model provider unavailable", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to list models", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_models(State(state): State<AppState>) -> Result<Json<ModelsResponse>, ApiError> {
    let models = state.orchestrator().list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}
