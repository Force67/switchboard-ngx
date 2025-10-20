use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

const DEFAULT_CONFIG_FILES: &[&str] = &[
    "switchboard.toml",
    "config/switchboard.toml",
    "crates/config/switchboard.toml",
    "../switchboard.toml",
    "../config/switchboard.toml",
    "../crates/config/switchboard.toml",
    "backend/switchboard.toml",
    "backend/config/switchboard.toml",
    "backend/crates/config/switchboard.toml",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub orchestrator: OrchestratorConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: HttpConfig::default(),
            orchestrator: OrchestratorConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub address: String,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 7070,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub default_model: String,
    pub provider_search_path: Vec<String>,
    #[serde(default)]
    pub openrouter: OpenRouterProviderConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            default_model: "gpt-4.1".to_string(),
            provider_search_path: vec!["providers".to_string()],
            openrouter: OpenRouterProviderConfig::default(),
        }
    }
}

/// Configuration options for the built-in OpenRouter provider integration.
///
/// ```
/// use switchboard_config::OpenRouterProviderConfig;
///
/// let provider = OpenRouterProviderConfig::default();
/// assert_eq!(provider.base_url, "https://openrouter.ai/api/v1");
/// assert_eq!(provider.request_timeout_seconds, 30);
/// assert!(provider.api_key.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterProviderConfig {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "OpenRouterProviderConfig::default_base_url")]
    pub base_url: String,
    #[serde(default = "OpenRouterProviderConfig::default_request_timeout")]
    pub request_timeout_seconds: u64,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default = "OpenRouterProviderConfig::default_title")]
    pub title: Option<String>,
}

impl OpenRouterProviderConfig {
    fn default_base_url() -> String {
        "https://openrouter.ai/api/v1".to_string()
    }

    const fn default_request_timeout() -> u64 {
        30
    }

    fn default_title() -> Option<String> {
        Some("Switchboard NGX".to_string())
    }
}

impl Default for OpenRouterProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: Self::default_base_url(),
            request_timeout_seconds: Self::default_request_timeout(),
            referer: None,
            title: Self::default_title(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://switchboard.db".to_string(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "AuthConfig::default_session_ttl")]
    pub session_ttl_seconds: u64,
    #[serde(default)]
    pub github: GithubAuthConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_ttl_seconds: 86_400,
            github: GithubAuthConfig::default(),
        }
    }
}

impl AuthConfig {
    fn default_session_ttl() -> u64 {
        86_400
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GithubAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// Load the application configuration by combining defaults, files, and environment overrides.
///
/// ```
/// use switchboard_config::load;
///
/// std::env::remove_var("SWITCHBOARD_CONFIG");
///
/// let config = load().expect("configuration should load with defaults");
/// assert!(!config.http.address.is_empty());
/// ```
pub fn load() -> anyhow::Result<AppConfig> {
    let defaults = AppConfig::default();

    let db_max = defaults.database.max_connections as i64;
    let session_ttl = defaults.auth.session_ttl_seconds;
    let session_ttl_i64 = if session_ttl > i64::MAX as u64 {
        i64::MAX
    } else {
        session_ttl as i64
    };

    let mut builder = config::Config::builder();
    builder = builder
        .set_default("http.address", defaults.http.address.clone())
        .unwrap()
        .set_default("http.port", i64::from(defaults.http.port))
        .unwrap()
        .set_default(
            "orchestrator.default_model",
            defaults.orchestrator.default_model.clone(),
        )
        .unwrap()
        .set_default(
            "orchestrator.provider_search_path",
            defaults.orchestrator.provider_search_path.clone(),
        )
        .unwrap()
        .set_default(
            "orchestrator.openrouter.base_url",
            defaults.orchestrator.openrouter.base_url.clone(),
        )
        .unwrap()
        .set_default(
            "orchestrator.openrouter.request_timeout_seconds",
            i64::try_from(defaults.orchestrator.openrouter.request_timeout_seconds)
                .unwrap_or(i64::MAX),
        )
        .unwrap();

    if let Some(title) = defaults.orchestrator.openrouter.title.clone() {
        builder = builder
            .set_default("orchestrator.openrouter.title", title)
            .unwrap();
    }

    builder = builder
        .set_default("database.url", defaults.database.url.clone())
        .unwrap()
        .set_default("database.max_connections", db_max)
        .unwrap()
        .set_default("auth.session_ttl_seconds", session_ttl_i64)
        .unwrap();

    let environment_overrides =
        config::Environment::with_prefix("SWITCHBOARD").separator("__");

    let mut config_file_attached = false;

    if let Ok(path) = std::env::var("SWITCHBOARD_CONFIG") {
        builder = builder.add_source(config::File::from(PathBuf::from(&path)));
        config_file_attached = true;
        debug!(path, "loading configuration via SWITCHBOARD_CONFIG");
    } else if let Ok(cwd) = std::env::current_dir() {
        let fallback = DEFAULT_CONFIG_FILES
            .iter()
            .map(|candidate| cwd.join(candidate))
            .find(|path| path.exists());

        if let Some(path) = fallback {
            debug!(path = %path.display(), "loading configuration file");
            builder = builder.add_source(config::File::from(path));
            config_file_attached = true;
        }
    }

    if !config_file_attached {
        debug!("no configuration file found, relying on defaults and environment overrides");
    }

    builder = builder.add_source(environment_overrides);

    let cfg = builder.build().context("unable to build configuration")?;

    let mut config = cfg
        .try_deserialize::<AppConfig>()
        .context("invalid configuration")?;

    if config.auth.session_ttl_seconds > i64::MAX as u64 {
        config.auth.session_ttl_seconds = i64::MAX as u64;
    }

    debug!(?config, "loaded backend configuration");
    Ok(config)
}
