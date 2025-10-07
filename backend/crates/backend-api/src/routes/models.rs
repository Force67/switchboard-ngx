use axum::{extract::State, Json};
use serde::Serialize;
use switchboard_orchestrator::OpenRouterModelSummary;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub models: Vec<OpenRouterModelSummary>,
}

pub async fn list_models(State(state): State<AppState>) -> Result<Json<ModelsResponse>, ApiError> {
    let models = state.orchestrator().list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}
