use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

use switchboard_config::{AppConfig, OrchestratorConfig};

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("provider index not initialised")]
    ProviderIndexMissing,
    #[error("failed to load providers: {0}")]
    ProviderLoad(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub identifier: String,
    pub family: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Default)]
struct ProviderIndex {
    providers: Vec<ProviderMetadata>,
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
        info!(count = providers.providers.len(), "provider catalogue initialised");
        self.providers = Some(providers);
        Ok(self)
    }

    pub fn active_model(&self) -> Option<String> {
        Some(self.config.default_model.clone())
    }
}

fn load_providers(config: &OrchestratorConfig) -> Result<ProviderIndex, OrchestratorError> {
    let mut providers = Vec::new();

    for path in &config.provider_search_path {
        let path = PathBuf::from(path);
        if !path.exists() {
            warn!(path = %path.display(), "skipping missing provider directory");
            continue;
        }

        for entry in std::fs::read_dir(&path).context("unable to list provider directory")? {
            let entry = entry.context("failed to access provider entry")?;
            if entry.file_type().context("failed to read file type")?.is_dir() {
                continue;
            }

            let file = std::fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read provider descriptor {:?}", entry.path()))?;
            let metadata: ProviderMetadata = serde_json::from_str(&file)
                .with_context(|| format!("invalid provider descriptor {:?}", entry.path()))?;
            providers.push(metadata);
        }
    }

    Ok(ProviderIndex { providers })
}
