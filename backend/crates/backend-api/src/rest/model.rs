//! LLM model routes

use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;
use tracing::warn;

use crate::error::GatewayResult;
use crate::rest::models::{fallback_models, ModelsResponse};
use crate::state::GatewayState;

/// Routes for model metadata
pub fn create_models_routes() -> Router<Arc<GatewayState>> {
    Router::new().route("/models", get(list_models))
}

#[utoipa::path(
    get,
    path = "/api/v1/models",
    tag = "Models",
    responses(
        (status = 200, description = "List available language models", body = ModelsResponse),
        (status = 503, description = "Model provider unavailable", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to list models", body = crate::error::ErrorResponse)
    )
)]
pub async fn list_models(
    State(state): State<Arc<GatewayState>>,
) -> GatewayResult<Json<ModelsResponse>> {
    let models = if let Some(orchestrator) = state.orchestrator() {
        match orchestrator.list_openrouter_models().await {
            Ok(models) if !models.is_empty() => models,
            Ok(_) => {
                warn!("orchestrator returned no models, using fallback catalogue");
                fallback_models()
            }
            Err(error) => {
                warn!(
                    ?error,
                    "failed to load models from orchestrator, using fallback catalogue"
                );
                fallback_models()
            }
        }
    } else {
        warn!("orchestrator unavailable, using fallback catalogue");
        fallback_models()
    };

    Ok(Json(ModelsResponse { models }))
}
