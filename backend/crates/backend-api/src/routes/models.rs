use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use switchboard_orchestrator::OpenRouterModelSummary;
use sqlx::FromRow;

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
    pub user_id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub is_group: bool,
    pub messages: String,
    pub created_at: String,
    pub updated_at: String,
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
    pub messages: Vec<ChatMessage>,
    pub folder_id: Option<String>, // public_id
    #[serde(default)]
    pub is_group: bool,
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

pub async fn list_models(State(state): State<AppState>) -> Result<Json<ModelsResponse>, ApiError> {
    let models = state.orchestrator().list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}
