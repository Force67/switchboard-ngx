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

pub async fn list_models(State(state): State<AppState>) -> Result<Json<ModelsResponse>, ApiError> {
    let models = state.orchestrator().list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}
