//! LLM model-related response types

use serde::Serialize;
use switchboard_orchestrator::{ModelPricing as OrchestratorModelPricing, OpenRouterModelSummary};
use utoipa::ToSchema;

/// Response containing available LLM models
#[derive(Debug, Serialize, ToSchema)]
pub struct ModelsResponse {
    #[schema(value_type = Vec<ModelSummary>)]
    pub models: Vec<OpenRouterModelSummary>,
}

/// Summary of an LLM model
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

/// Pricing information for an LLM model
#[derive(Debug, Serialize, ToSchema)]
pub struct ModelPricing {
    #[schema(nullable)]
    pub input: Option<f64>,
    #[schema(nullable)]
    pub output: Option<f64>,
}

/// Returns fallback models when the orchestrator is unavailable
pub fn fallback_models() -> Vec<OpenRouterModelSummary> {
    vec![
        OpenRouterModelSummary {
            id: "debug/echo".to_string(),
            label: "Debug Echo (offline)".to_string(),
            description: Some(
                "Returns a deterministic fallback response without calling a provider.".to_string(),
            ),
            pricing: None,
            supports_reasoning: false,
            supports_images: false,
            supports_tools: false,
            supports_agents: false,
            supports_function_calling: false,
            supports_vision: false,
            supports_tool_use: false,
            supports_structured_outputs: false,
            supports_streaming: false,
        },
        OpenRouterModelSummary {
            id: "gpt-4o-mini".to_string(),
            label: "GPT-4o Mini".to_string(),
            description: Some(
                "Fast, low-cost model suitable for development and smoke tests".to_string(),
            ),
            pricing: Some(OrchestratorModelPricing {
                input: Some(0.15),
                output: Some(0.6),
            }),
            supports_reasoning: false,
            supports_images: true,
            supports_tools: true,
            supports_agents: false,
            supports_function_calling: true,
            supports_vision: true,
            supports_tool_use: true,
            supports_structured_outputs: false,
            supports_streaming: true,
        },
        OpenRouterModelSummary {
            id: "gpt-3.5-turbo".to_string(),
            label: "GPT-3.5 Turbo".to_string(),
            description: Some("Legacy baseline model".to_string()),
            pricing: Some(OrchestratorModelPricing {
                input: Some(0.5),
                output: Some(1.5),
            }),
            supports_reasoning: false,
            supports_images: false,
            supports_tools: true,
            supports_agents: false,
            supports_function_calling: true,
            supports_vision: false,
            supports_tool_use: true,
            supports_structured_outputs: false,
            supports_streaming: true,
        },
    ]
}
