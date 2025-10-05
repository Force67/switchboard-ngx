use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::debug;

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
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            default_model: "gpt-4.1".to_string(),
            provider_search_path: vec!["providers".to_string()],
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

pub fn load() -> anyhow::Result<AppConfig> {
    let defaults = AppConfig::default();

    let db_max = defaults.database.max_connections as i64;
    let session_ttl = defaults.auth.session_ttl_seconds;
    let session_ttl_i64 = if session_ttl > i64::MAX as u64 {
        i64::MAX
    } else {
        session_ttl as i64
    };

    let builder = config::Config::builder()
        .set_default("http.address", defaults.http.address.clone())
        .unwrap()
        .set_default("http.port", i64::from(defaults.http.port))
        .unwrap()
        .set_default("orchestrator.default_model", defaults.orchestrator.default_model.clone())
        .unwrap()
        .set_default(
            "orchestrator.provider_search_path",
            defaults.orchestrator.provider_search_path.clone(),
        )
        .unwrap()
        .set_default("database.url", defaults.database.url.clone())
        .unwrap()
        .set_default("database.max_connections", db_max)
        .unwrap()
        .set_default(
            "auth.session_ttl_seconds",
            session_ttl_i64,
        )
        .unwrap()
        .add_source(config::Environment::with_prefix("SWITCHBOARD").separator("__"));

    let builder = if let Ok(path) = std::env::var("SWITCHBOARD_CONFIG") {
        builder.add_source(config::File::with_name(&path).required(false))
    } else {
        builder
    };

    let cfg = builder
        .build()
        .context("unable to build configuration")?;

    let config = cfg
        .try_deserialize::<AppConfig>()
        .context("invalid configuration")?;

    debug!(?config, "loaded backend configuration");
    Ok(config)
}
