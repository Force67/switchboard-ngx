use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use denkwerk::{
    providers::{
        openrouter::{
            OpenRouter as DenkwerkOpenRouter, OpenRouterConfig as DenkwerkOpenRouterConfig,
        },
        LLMProvider,
    },
    LLMError, ProviderCapabilities,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use tracing::{debug, error, info, warn};

use switchboard_config::{AppConfig, OpenRouterProviderConfig, OrchestratorConfig};

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("provider index not initialised")]
    ProviderIndexMissing,
    #[error("failed to load providers: {0}")]
    ProviderLoad(#[from] anyhow::Error),
    #[error("missing OpenRouter API key")]
    OpenRouterApiKeyMissing,
    #[error("failed to initialise provider {identifier}: {source}")]
    ProviderInit {
        identifier: &'static str,
        #[source]
        source: LLMError,
    },
    #[error("provider {0} is not registered")]
    ProviderNotFound(String),
    #[error("provider http request failed: {0}")]
    ProviderHttp(#[from] reqwest::Error),
    #[error("invalid provider response: {0}")]
    ProviderResponse(#[from] serde_json::Error),
    #[error("openrouter provider is not available")]
    OpenRouterUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub identifier: String,
    pub family: String,
    pub capabilities: Vec<String>,
}

#[derive(Default)]
struct ProviderIndex {
    metadata: Vec<ProviderMetadata>,
    handles: HashMap<String, Arc<dyn LLMProvider>>,
    openrouter: Option<ResolvedOpenRouterConfig>,
}

impl ProviderIndex {
    fn new(metadata: Vec<ProviderMetadata>) -> Self {
        Self {
            metadata,
            handles: HashMap::new(),
            openrouter: None,
        }
    }

    fn len(&self) -> usize {
        self.handles.len()
    }

    fn upsert_metadata(&mut self, metadata: ProviderMetadata) {
        if let Some(existing) = self
            .metadata
            .iter_mut()
            .find(|entry| entry.identifier == metadata.identifier)
        {
            *existing = metadata;
        } else {
            self.metadata.push(metadata);
        }
    }

    fn register(&mut self, metadata: ProviderMetadata, provider: Arc<dyn LLMProvider>) {
        let identifier = metadata.identifier.clone();
        self.upsert_metadata(metadata);
        self.handles.insert(identifier, provider);
    }

    fn get(&self, identifier: &str) -> Option<Arc<dyn LLMProvider>> {
        self.handles.get(identifier).cloned()
    }
}

#[derive(Debug, Clone)]
struct ResolvedOpenRouterConfig {
    api_key: String,
    base_url: String,
    request_timeout: Duration,
    referer: Option<String>,
    title: Option<String>,
}

pub struct Orchestrator {
    config: OrchestratorConfig,
    providers: Option<ProviderIndex>,
}

impl Orchestrator {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            config: config.orchestrator.clone(),
            providers: None,
        }
    }

    pub fn bootstrap(mut self) -> Result<Self, OrchestratorError> {
        let providers = load_providers(&self.config)?;
        info!(count = providers.len(), "provider catalogue initialised");
        self.providers = Some(providers);
        Ok(self)
    }

    pub fn active_model(&self) -> Option<String> {
        Some(self.config.default_model.clone())
    }

    pub fn provider(&self, identifier: &str) -> Result<Arc<dyn LLMProvider>, OrchestratorError> {
        let providers = self
            .providers
            .as_ref()
            .ok_or(OrchestratorError::ProviderIndexMissing)?;

        providers
            .get(identifier)
            .ok_or_else(|| OrchestratorError::ProviderNotFound(identifier.to_string()))
    }

    pub fn default_provider(&self) -> Result<Arc<dyn LLMProvider>, OrchestratorError> {
        self.provider_for_model(&self.config.default_model)
    }

    pub fn provider_for_model(
        &self,
        model: &str,
    ) -> Result<Arc<dyn LLMProvider>, OrchestratorError> {
        let providers = self
            .providers
            .as_ref()
            .ok_or(OrchestratorError::ProviderIndexMissing)?;

        let identifier = provider_identifier_from_model(model);
        if let Some(provider) = providers.get(&identifier) {
            return Ok(provider);
        }

        if let Some(provider) = providers.get("openrouter") {
            debug!(model = %model, fallback = "openrouter", "falling back to openrouter provider");
            return Ok(provider);
        }

        Err(OrchestratorError::ProviderNotFound(identifier))
    }

    pub async fn list_openrouter_models(
        &self,
    ) -> Result<Vec<OpenRouterModelSummary>, OrchestratorError> {
        let providers = self
            .providers
            .as_ref()
            .ok_or(OrchestratorError::ProviderIndexMissing)?;
        let openrouter = providers
            .openrouter
            .clone()
            .ok_or(OrchestratorError::OpenRouterUnavailable)?;

        let client = Client::builder()
            .timeout(openrouter.request_timeout)
            .build()?;

        let url = format!("{}/models", openrouter.base_url.trim_end_matches('/'));

        let mut request = client.get(url).bearer_auth(openrouter.api_key);
        if let Some(referer) = &openrouter.referer {
            request = request.header("HTTP-Referer", referer);
        }
        if let Some(title) = &openrouter.title {
            request = request.header("X-Title", title);
        }

        let response = request.send().await?.error_for_status()?;

        // Debug: log the raw response text first
        let response_text = response.text().await?;
        debug!(raw_response = %response_text, "Raw OpenRouter API response");

        // Try to parse as JSON
        let parsed: OpenRouterModelList = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!(error = %e, "Failed to parse OpenRouter response as JSON");
                error!(raw_response_preview = %&response_text[..response_text.len().min(500)], "Response preview");
                OrchestratorError::ProviderResponse(e)
            })?;

        let models = parsed
            .data
            .into_iter()
            .map(|model| {
                // Parse pricing directly from JSON - be more flexible with field names
                let pricing = model.pricing.as_ref().map(|p| {
                    let input = p
                        .prompt
                        .as_ref()
                        .or_else(|| p.input.as_ref())
                        .and_then(|s| s.parse::<f64>().ok());
                    let output = p
                        .completion
                        .as_ref()
                        .or_else(|| p.output.as_ref())
                        .and_then(|s| s.parse::<f64>().ok());
                    ModelPricing { input, output }
                });

                // Trust capabilities array from OpenRouter API
                let caps = model.capabilities.unwrap_or_default();

                // Get input modalities from the array (preferred)
                let input_modalities = model
                    .architecture
                    .as_ref()
                    .and_then(|a| a.input_modalities.clone())
                    .unwrap_or_default();

                // Fallback: parse string modality if input_modalities is empty
                let modalities = if input_modalities.is_empty() {
                    model
                        .architecture
                        .as_ref()
                        .and_then(|a| a.modality.as_ref())
                        .map(|m| {
                            // Parse "text+image->text" or "text->text" format
                            if m.contains("->") {
                                m.split("->")
                                    .next()
                                    .unwrap_or("")
                                    .split("+")
                                    .map(|s| s.trim().to_string())
                                    .collect()
                            } else {
                                m.split("+").map(|s| s.trim().to_string()).collect()
                            }
                        })
                        .unwrap_or_default()
                } else {
                    input_modalities
                };

                // Get supported parameters for capability detection
                let supported_params = model.supported_parameters.unwrap_or_default();

                OpenRouterModelSummary {
                    id: model.id.clone(),
                    label: model.name.unwrap_or_else(|| model.id.clone()),
                    description: model.description,
                    pricing,
                    supports_reasoning: supported_params.contains(&"reasoning".to_string()),
                    supports_images: modalities.contains(&"image".to_string()),
                    supports_tools: supported_params.contains(&"tools".to_string())
                        || supported_params.contains(&"tool_choice".to_string()),
                    supports_agents: caps.contains(&"agents".to_string()), // Only if explicitly marked as agent
                    supports_function_calling: supported_params
                        .contains(&"tool_choice".to_string())
                        || supported_params.contains(&"tools".to_string()),
                    supports_vision: modalities.contains(&"image".to_string()),
                    supports_tool_use: supported_params.contains(&"tools".to_string()),
                    supports_structured_outputs: supported_params
                        .contains(&"structured_outputs".to_string()),
                    supports_streaming: supported_params.contains(&"streaming".to_string()) || true, // Default true for modern models
                }
            })
            .collect();

        Ok(models)
    }
}

fn load_providers(config: &OrchestratorConfig) -> Result<ProviderIndex, OrchestratorError> {
    let mut metadata = Vec::new();

    for path in &config.provider_search_path {
        let path = PathBuf::from(path);
        if !path.exists() {
            warn!(path = %path.display(), "skipping missing provider directory");
            continue;
        }

        for entry in std::fs::read_dir(&path).context("unable to list provider directory")? {
            let entry = entry.context("failed to access provider entry")?;
            if entry
                .file_type()
                .context("failed to read file type")?
                .is_dir()
            {
                continue;
            }

            let file = std::fs::read_to_string(entry.path()).with_context(|| {
                format!("failed to read provider descriptor {:?}", entry.path())
            })?;
            let descriptor: ProviderMetadata = serde_json::from_str(&file)
                .with_context(|| format!("invalid provider descriptor {:?}", entry.path()))?;
            metadata.push(descriptor);
        }
    }

    let mut index = ProviderIndex::new(metadata);
    register_openrouter_provider(&config.openrouter, &mut index)?;

    Ok(index)
}

fn register_openrouter_provider(
    config: &OpenRouterProviderConfig,
    index: &mut ProviderIndex,
) -> Result<(), OrchestratorError> {
    let api_key = config
        .api_key
        .clone()
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .ok_or(OrchestratorError::OpenRouterApiKeyMissing)?;

    let api_key_source = if config.api_key.is_some() {
        "config"
    } else {
        "env"
    };

    let mut provider_config = DenkwerkOpenRouterConfig::new(api_key.clone());
    provider_config.base_url = config.base_url.clone();
    provider_config.request_timeout = Duration::from_secs(config.request_timeout_seconds);
    provider_config.referer = config.referer.clone();
    provider_config.title = config.title.clone();

    debug!(source = api_key_source, "initialising OpenRouter provider");

    let provider = DenkwerkOpenRouter::from_config(provider_config).map_err(|source| {
        OrchestratorError::ProviderInit {
            identifier: "openrouter",
            source,
        }
    })?;

    let provider_caps = provider.capabilities();
    let metadata = ProviderMetadata {
        identifier: "openrouter".to_string(),
        family: "openrouter".to_string(),
        capabilities: describe_capabilities(provider_caps),
    };

    let provider: Arc<dyn LLMProvider> = Arc::new(provider);
    index.register(metadata, provider);

    index.openrouter = Some(ResolvedOpenRouterConfig {
        api_key,
        base_url: config.base_url.clone(),
        request_timeout: Duration::from_secs(config.request_timeout_seconds),
        referer: config.referer.clone(),
        title: config.title.clone(),
    });
    Ok(())
}

fn describe_capabilities(capabilities: ProviderCapabilities) -> Vec<String> {
    let mut values = vec!["chat-completions".to_string()];

    if capabilities.supports_streaming {
        values.push("streaming".to_string());
    }

    if capabilities.supports_reasoning_stream {
        values.push("reasoning".to_string());
    }

    if capabilities.supports_image_uploads {
        values.push("image-uploads".to_string());
    }

    values
}

fn provider_identifier_from_model(model: &str) -> String {
    model
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("openrouter")
        .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input: Option<f64>,
    pub output: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModelSummary {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub supports_images: bool,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_agents: bool,
    #[serde(default)]
    pub supports_function_calling: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_tool_use: bool,
    #[serde(default)]
    pub supports_structured_outputs: bool,
    #[serde(default)]
    pub supports_streaming: bool,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelList {
    data: Vec<OpenRouterModelEntry>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelEntry {
    id: String,
    #[serde(default, alias = "model_name")]
    name: Option<String>,
    #[serde(default, alias = "model_description")]
    description: Option<String>,
    #[serde(default)]
    pricing: Option<OpenRouterPricing>,
    #[serde(default)]
    capabilities: Option<Vec<String>>,
    #[serde(default)]
    architecture: Option<OpenRouterArchitecture>,
    #[serde(default)]
    supported_parameters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterArchitecture {
    #[serde(default)]
    modality: Option<String>, // Keep as string for now
    #[serde(default)]
    input_modalities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    #[serde(default, alias = "prompt_price")]
    prompt: Option<String>,
    #[serde(default, alias = "completion_price")]
    completion: Option<String>,
    #[serde(default)]
    input: Option<String>,
    #[serde(default)]
    output: Option<String>,
}
