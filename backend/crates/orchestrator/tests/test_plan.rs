//! Integration tests for the orchestrator crate.

use std::{fs, path::Path, sync::Arc, time::Duration};

use async_trait::async_trait;
use denkwerk::{
    CompletionRequest, CompletionResponse, CompletionStream, ImageUploadRequest,
    ImageUploadResponse, LLMError, LLMProvider, ProviderCapabilities,
};
use httpmock::prelude::*;
use switchboard_config::{AppConfig, OpenRouterProviderConfig, OrchestratorConfig};
use switchboard_orchestrator::{
    test_support::{self, OrchestratorTestBuilder, TestOpenRouterSettings},
    Orchestrator, OrchestratorError, ProviderMetadata,
};
use tempfile::tempdir;

#[derive(Clone)]
struct DummyProvider {
    name: &'static str,
    capabilities: ProviderCapabilities,
}

impl DummyProvider {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            capabilities: ProviderCapabilities::default(),
        }
    }
}

#[async_trait]
impl LLMProvider for DummyProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        Err(LLMError::Unsupported("complete"))
    }

    async fn stream_completion(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionStream, LLMError> {
        Err(LLMError::Unsupported("stream"))
    }

    async fn upload_image(
        &self,
        _request: ImageUploadRequest,
    ) -> Result<ImageUploadResponse, LLMError> {
        Err(LLMError::Unsupported("upload"))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.capabilities
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

fn provider_descriptor(identifier: &str, family: &str) -> ProviderMetadata {
    ProviderMetadata {
        identifier: identifier.to_string(),
        family: family.to_string(),
        capabilities: vec!["chat-completions".to_string()],
    }
}

fn config_with_search_path(path: &Path) -> AppConfig {
    let mut config = AppConfig::default();
    config.orchestrator.provider_search_path = vec![path.display().to_string()];
    config
}

fn config_with_openrouter_key(api_key: &str) -> AppConfig {
    let mut config = AppConfig::default();
    config.orchestrator.openrouter.api_key = Some(api_key.to_string());
    config
}

fn openrouter_metadata() -> ProviderMetadata {
    ProviderMetadata {
        identifier: "openrouter".to_string(),
        family: "openrouter".to_string(),
        capabilities: vec!["chat-completions".to_string()],
    }
}

#[test]
fn bootstrap_registers_providers_from_search_path() {
    let temp = tempdir().expect("tempdir");
    let descriptor_a = provider_descriptor("acme", "llm");
    let descriptor_b = provider_descriptor("contoso", "llm");

    fs::write(
        temp.path().join("acme.json"),
        serde_json::to_string(&descriptor_a).unwrap(),
    )
    .unwrap();
    fs::write(
        temp.path().join("contoso.json"),
        serde_json::to_string(&descriptor_b).unwrap(),
    )
    .unwrap();

    let mut config = config_with_search_path(temp.path());
    config.orchestrator.openrouter = OpenRouterProviderConfig {
        api_key: Some("test-key".to_string()),
        base_url: "https://api.test".to_string(),
        request_timeout_seconds: 5,
        referer: None,
        title: Some("Test".to_string()),
    };

    let orchestrator = Orchestrator::new(&config)
        .bootstrap()
        .expect("bootstrap succeeds");

    let identifiers = test_support::provider_identifiers(&orchestrator).unwrap();
    assert!(identifiers.contains(&"acme".to_string()));
    assert!(identifiers.contains(&"contoso".to_string()));
    assert!(identifiers.contains(&"openrouter".to_string()));
}

#[test]
fn bootstrap_returns_error_when_descriptor_is_invalid() {
    let temp = tempdir().expect("tempdir");
    fs::write(temp.path().join("broken.json"), "{not:json").unwrap();

    let config = config_with_search_path(temp.path());

    let err = match Orchestrator::new(&config).bootstrap() {
        Ok(_) => panic!("bootstrap should fail"),
        Err(err) => err,
    };

    assert!(matches!(err, OrchestratorError::ProviderLoad(_)));
}

#[test]
fn provider_returns_registered_handle() {
    let mut config = config_with_openrouter_key("test-api");
    config.orchestrator.openrouter.base_url = "https://api.test".to_string();
    config.orchestrator.openrouter.request_timeout_seconds = 5;

    let orchestrator = Orchestrator::new(&config)
        .bootstrap()
        .expect("bootstrap succeeds");

    assert!(orchestrator.provider("openrouter").is_ok());
}

#[test]
fn provider_errors_when_identifier_unknown() {
    let mut config = config_with_openrouter_key("test-api");
    config.orchestrator.openrouter.base_url = "https://api.test".to_string();
    config.orchestrator.openrouter.request_timeout_seconds = 5;

    let orchestrator = Orchestrator::new(&config)
        .bootstrap()
        .expect("bootstrap succeeds");

    let err = match orchestrator.provider("unknown") {
        Ok(_) => panic!("unknown provider should error"),
        Err(err) => err,
    };
    assert!(matches!(err, OrchestratorError::ProviderNotFound(_)));
}

#[test]
fn provider_for_model_selects_exact_identifier() {
    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();
    config.default_model = "dummy/model".to_string();

    let dummy_provider: Arc<dyn LLMProvider> = Arc::new(DummyProvider::new("dummy"));
    let orchestrator = OrchestratorTestBuilder::new(config)
        .with_provider(provider_descriptor("dummy", "llm"), dummy_provider.clone())
        .build();

    let resolved = orchestrator
        .provider_for_model("dummy/small")
        .expect("provider should resolve");

    assert!(Arc::ptr_eq(&resolved, &dummy_provider));
}

#[test]
fn provider_for_model_falls_back_to_openrouter_when_missing() {
    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();

    let openrouter_provider: Arc<dyn LLMProvider> = Arc::new(DummyProvider::new("openrouter"));
    let orchestrator = OrchestratorTestBuilder::new(config)
        .with_provider(openrouter_metadata(), openrouter_provider.clone())
        .with_openrouter(
            TestOpenRouterSettings::new("fallback-key", "http://localhost")
                .with_timeout(Duration::from_secs(1)),
        )
        .build();

    let resolved = orchestrator
        .provider_for_model("anthropic/claude")
        .expect("fallback to openrouter");

    assert!(Arc::ptr_eq(&resolved, &openrouter_provider));
}

#[test]
fn provider_for_model_errors_without_openrouter_fallback() {
    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();

    let orchestrator = OrchestratorTestBuilder::new(config).build();
    let err = match orchestrator.provider_for_model("missing/model") {
        Ok(_) => panic!("error expected when provider missing"),
        Err(err) => err,
    };

    match err {
        OrchestratorError::ProviderNotFound(identifier) => assert_eq!(identifier, "missing"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn default_provider_returns_configured_model_provider() {
    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();
    config.default_model = "dummy/default".to_string();

    let dummy_provider: Arc<dyn LLMProvider> = Arc::new(DummyProvider::new("dummy"));
    let orchestrator = OrchestratorTestBuilder::new(config)
        .with_provider(provider_descriptor("dummy", "llm"), dummy_provider.clone())
        .build();

    let resolved = orchestrator
        .default_provider()
        .expect("default provider should resolve");

    assert!(Arc::ptr_eq(&resolved, &dummy_provider));
}

#[tokio::test]
async fn list_openrouter_models_fetches_and_parses_response() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/models");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    serde_json::json!({
                        "data": [{
                            "id": "anthropic/claude-3",
                            "model_name": "Claude 3",
                            "model_description": "A helpful model",
                            "pricing": {
                                "prompt_price": "0.0001",
                                "completion_price": "0.0002"
                            },
                            "capabilities": ["agents"],
                            "architecture": {
                                "input_modalities": ["text"]
                            },
                            "supported_parameters": ["tools", "streaming"]
                        }]
                    })
                    .to_string(),
                );
        })
        .await;

    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();

    let openrouter_provider: Arc<dyn LLMProvider> = Arc::new(DummyProvider::new("openrouter"));
    let settings = TestOpenRouterSettings::new("test-key", server.base_url())
        .with_timeout(Duration::from_secs(1))
        .with_title("Switchboard Tests");

    let orchestrator = OrchestratorTestBuilder::new(config)
        .with_provider(openrouter_metadata(), openrouter_provider)
        .with_openrouter(settings)
        .build();

    let models = orchestrator
        .list_openrouter_models()
        .await
        .expect("models should be returned");

    assert_eq!(models.len(), 1);
    let model = &models[0];
    assert_eq!(model.id, "anthropic/claude-3");
    assert_eq!(model.label, "Claude 3");
    assert_eq!(model.pricing.as_ref().unwrap().input, Some(0.0001));
    assert!(model.supports_streaming);
    assert!(model.supports_tool_use);
    assert!(model.supports_agents);
}

#[tokio::test]
async fn list_openrouter_models_handles_http_errors() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/models");
            then.status(503);
        })
        .await;

    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();

    let openrouter_provider: Arc<dyn LLMProvider> = Arc::new(DummyProvider::new("openrouter"));
    let orchestrator = OrchestratorTestBuilder::new(config)
        .with_provider(openrouter_metadata(), openrouter_provider)
        .with_openrouter(
            TestOpenRouterSettings::new("test-key", server.base_url())
                .with_timeout(Duration::from_secs(1)),
        )
        .build();

    let err = orchestrator
        .list_openrouter_models()
        .await
        .expect_err("http error expected");

    assert!(matches!(err, OrchestratorError::ProviderHttp(_)));
}

#[tokio::test]
async fn list_openrouter_models_requires_openrouter_registration() {
    let mut config = OrchestratorConfig::default();
    config.provider_search_path.clear();

    let orchestrator = OrchestratorTestBuilder::new(config).build();
    let err = orchestrator
        .list_openrouter_models()
        .await
        .expect_err("openrouter missing should error");

    assert!(matches!(err, OrchestratorError::OpenRouterUnavailable));
}

#[test]
fn bootstrap_requires_provider_index_initialised() {
    let config = AppConfig::default();
    let orchestrator = Orchestrator::new(&config);

    let err = match orchestrator.provider("openrouter") {
        Ok(_) => panic!("provider index missing"),
        Err(err) => err,
    };
    assert!(matches!(err, OrchestratorError::ProviderIndexMissing));
}

#[test]
fn register_openrouter_requires_api_key() {
    let config = AppConfig::default();
    let err = match Orchestrator::new(&config).bootstrap() {
        Ok(_) => panic!("missing api key should error"),
        Err(err) => err,
    };

    assert!(matches!(err, OrchestratorError::OpenRouterApiKeyMissing));
}

#[test]
fn describe_capabilities_maps_flags_to_strings() {
    let caps = ProviderCapabilities::new(true, true, true);
    let summary = test_support::describe_capabilities(caps);
    assert_eq!(
        summary,
        vec![
            "chat-completions",
            "streaming",
            "reasoning",
            "image-uploads"
        ]
    );

    let minimal_caps = ProviderCapabilities::default();
    let minimal_summary = test_support::describe_capabilities(minimal_caps);
    assert_eq!(minimal_summary, vec!["chat-completions"]);
}

#[test]
fn provider_identifier_from_model_splits_on_slash_prefix() {
    let identifier = test_support::provider_identifier_from_model("anthropic/claude-3");
    assert_eq!(identifier, "anthropic");
}

#[test]
fn provider_identifier_from_model_defaults_to_openrouter() {
    let identifier = test_support::provider_identifier_from_model("gpt-4.1");
    assert_eq!(identifier, "gpt-4.1");

    let empty_prefix = test_support::provider_identifier_from_model("/gpt-4.1");
    assert_eq!(empty_prefix, "openrouter");
}

#[test]
fn load_providers_skips_missing_directories_with_warning() {
    let temp = tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let mut config = config_with_search_path(&missing);
    config.orchestrator.openrouter = OpenRouterProviderConfig {
        api_key: Some("key".to_string()),
        base_url: "https://api.test".to_string(),
        request_timeout_seconds: 5,
        referer: None,
        title: Some("Test".to_string()),
    };

    let orchestrator = Orchestrator::new(&config)
        .bootstrap()
        .expect("bootstrap succeeds despite missing dirs");

    let identifiers = test_support::provider_identifiers(&orchestrator).unwrap();
    assert_eq!(identifiers, vec!["openrouter".to_string()]);
}

#[test]
fn load_providers_errors_on_invalid_json_descriptor() {
    let temp = tempdir().expect("tempdir");
    let descriptor_path = temp.path().join("broken.json");
    fs::write(&descriptor_path, "{\"identifier\": \"").unwrap();

    let config = config_with_search_path(temp.path());

    let err = match Orchestrator::new(&config).bootstrap() {
        Ok(_) => panic!("invalid descriptor should error"),
        Err(err) => err,
    };

    let message = err.to_string();
    assert!(message.contains("failed to load providers"));
    assert!(message.contains("invalid provider descriptor"));
}

#[test]
fn active_model_reports_configured_default_model() {
    let mut config = AppConfig::default();
    config.orchestrator.default_model = "custom-model".to_string();

    let orchestrator = Orchestrator::new(&config);
    assert_eq!(orchestrator.active_model().as_deref(), Some("custom-model"));
}
