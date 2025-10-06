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
use thiserror::Error;
use tracing::{debug, info, warn};

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
        let parsed: OpenRouterModelList = response.json().await?;

        let models = parsed
            .data
            .into_iter()
            .map(|model| {
                let pricing = model.pricing.as_ref().map(|p| {
                    let input = p.get("prompt").and_then(|s| s.parse::<f64>().ok());
                    let output = p.get("completion").and_then(|s| s.parse::<f64>().ok());
                    ModelPricing { input, output }
                });
                let supports_images = model.modality.as_ref().map(|m| m.contains("image")).unwrap_or(false);
                let supports_reasoning = model.id.contains("deepseek") || model.id.contains("o1") || model.id.contains("reasoning");
                OpenRouterModelSummary {
                    id: model.id.clone(),
                    label: model.name.unwrap_or_else(|| model.id.clone()),
                    description: model.description,
                    pricing,
                    supports_reasoning,
                    supports_images,
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
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelList {
    data: Vec<OpenRouterModelEntry>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    pricing: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    modality: Option<String>,
}
