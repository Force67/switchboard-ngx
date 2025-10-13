use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use switchboard_orchestrator::OpenRouterModelSummary;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<OpenRouterModelSummary>,
}

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub public_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize, FromRow)]
pub struct MessageEdit {
    pub id: i64,
    pub message_id: i64,
    pub edited_by_user_id: i64,
    pub old_content: String,
    pub new_content: String,
    pub edited_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageDeletion {
    pub id: i64,
    pub message_id: i64,
    pub deleted_by_user_id: i64,
    pub reason: Option<String>,
    pub deleted_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageAttachment {
    pub id: i64,
    pub message_id: i64,
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub file_size_bytes: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Notification {
    pub id: i64,
    pub user_id: i64,
    pub r#type: String, // "type" is a reserved keyword in Rust
    pub title: String,
    pub body: String,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Permission {
    pub id: i64,
    pub user_id: i64,
    pub resource_type: String,
    pub resource_id: i64,
    pub permission_level: String,
    pub granted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>, // public_id
}

#[derive(Debug, Deserialize)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub collapsed: Option<bool>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct UpdateChatRequest {
    pub title: Option<String>,
    pub messages: Option<Vec<ChatMessage>>,
    pub folder_id: Option<String>, // public_id
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub usage: Option<TokenUsage>,
    pub reasoning: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ChatInvite {
    pub id: i64,
    pub chat_id: i64,
    pub inviter_id: i64,
    pub invitee_email: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct InvitesResponse {
    pub invites: Vec<ChatInvite>,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub invite: ChatInvite,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ChatMember {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct MembersResponse {
    pub members: Vec<ChatMember>,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub member: ChatMember,
}

// New DTOs for message features
#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub role: String,
    pub model: Option<String>,
    pub message_type: Option<String>,
    pub thread_id: Option<String>, // public_id
    pub reply_to_id: Option<String>, // public_id
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub file_type: String,
    pub file_url: String,
    pub file_size_bytes: i64,
}

// Notification DTOs
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: String, // public_id
    pub r#type: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkNotificationReadRequest {
    pub read: bool,
}

#[derive(Debug, Serialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub notification: Notification,
}

// Permission DTOs
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub user_id: String, // public_id
    pub resource_type: String,
    pub resource_id: String, // public_id
    pub permission_level: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub permission: Permission,
}

// Message edit history
#[derive(Debug, Serialize)]
pub struct MessageEditsResponse {
    pub edits: Vec<MessageEdit>,
}

// Attachment response structs
#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub attachment: MessageAttachment,
}

#[derive(Debug, Serialize)]
pub struct AttachmentsResponse {
    pub attachments: Vec<MessageAttachment>,
}

// Message response structs
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: Message,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

pub async fn list_models(State(state): State<AppState>) -> Result<Json<ModelsResponse>, ApiError> {
    let models = state.orchestrator().list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}
