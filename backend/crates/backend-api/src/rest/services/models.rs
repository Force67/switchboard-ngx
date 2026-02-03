//! Models service implementation (LLM models)

use crate::error::GatewayResult;
use crate::generated::rest::traits::ModelsServiceTrait;
use crate::rest::models::{fallback_models, ModelsResponse};
use crate::state::GatewayState;
use tracing::warn;

pub struct ModelsServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> ModelsServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl ModelsServiceTrait for ModelsServiceImpl<'_> {
    async fn list_models(&self) -> GatewayResult<ModelsResponse> {
        let models = if let Some(orchestrator) = self.state.orchestrator() {
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

        Ok(ModelsResponse { models })
    }
}
