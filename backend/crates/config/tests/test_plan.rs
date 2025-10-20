//! Comprehensive test plan for the `switchboard-config` crate.
//!
//! These tests exercise the configuration loader across default handling,
//! file discovery, environment overrides, and validation behaviour.

use std::fs;
use std::path::{Path, PathBuf};

use serial_test::serial;
use tempfile::TempDir;

use switchboard_config::{load, AppConfig, AuthConfig, HttpConfig, OrchestratorConfig};

const ENV_VARS_TO_RESET: &[&str] = &[
    "DATABASE_URL",
    "SWITCHBOARD_CONFIG",
    "SWITCHBOARD__AUTH__GITHUB__CLIENT_ID",
    "SWITCHBOARD__AUTH__GITHUB__CLIENT_SECRET",
    "SWITCHBOARD__AUTH__SESSION_TTL_SECONDS",
    "SWITCHBOARD__DATABASE__MAX_CONNECTIONS",
    "SWITCHBOARD__DATABASE__URL",
    "SWITCHBOARD__HTTP__ADDRESS",
    "SWITCHBOARD__HTTP__PORT",
    "SWITCHBOARD__ORCHESTRATOR__DEFAULT_MODEL",
    "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__API_KEY",
    "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__BASE_URL",
    "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__REFERER",
    "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__REQUEST_TIMEOUT_SECONDS",
    "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__TITLE",
    "SWITCHBOARD__ORCHESTRATOR__PROVIDER_SEARCH_PATH",
];

struct TestContext {
    vars: Vec<(String, Option<String>)>,
    original_dir: Option<PathBuf>,
}

impl TestContext {
    fn new() -> Self {
        Self {
            vars: Vec::new(),
            original_dir: None,
        }
    }

    fn reset_environment(&mut self) {
        for key in ENV_VARS_TO_RESET {
            self.remove_var(key);
        }
    }

    fn set_var(&mut self, key: &str, value: impl AsRef<str>) {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value.as_ref());
        self.vars.push((key.to_string(), previous));
    }

    fn remove_var(&mut self, key: &str) {
        let previous = std::env::var(key).ok();
        std::env::remove_var(key);
        self.vars.push((key.to_string(), previous));
    }

    fn set_current_dir(&mut self, dir: &Path) {
        if self.original_dir.is_none() {
            self.original_dir =
                Some(std::env::current_dir().expect("failed to capture current directory"));
        }
        std::env::set_current_dir(dir).expect("failed to set current directory");
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(original) = self.original_dir.take() {
            let _ = std::env::set_current_dir(original);
        }

        while let Some((key, value)) = self.vars.pop() {
            match value {
                Some(val) => std::env::set_var(&key, val),
                None => std::env::remove_var(&key),
            }
        }
    }
}

fn write_config_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create config directories");
    }
    fs::write(path, contents).expect("failed to write config file");
}

#[test]
#[serial]
fn load_uses_default_values_when_no_files_found() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    let config = load().expect("configuration load should succeed without files");
    let defaults = AppConfig::default();

    assert_eq!(config.http.address, defaults.http.address);
    assert_eq!(config.http.port, defaults.http.port);
    assert_eq!(
        config.orchestrator.default_model,
        defaults.orchestrator.default_model
    );
    assert_eq!(
        config.orchestrator.provider_search_path,
        defaults.orchestrator.provider_search_path
    );
    assert_eq!(config.database.url, defaults.database.url);
    assert_eq!(
        config.database.max_connections,
        defaults.database.max_connections
    );
    assert_eq!(config.auth.session_ttl_seconds, defaults.auth.session_ttl_seconds);
    assert_eq!(config.auth.github.client_id, defaults.auth.github.client_id);
    assert_eq!(
        config.auth.github.client_secret,
        defaults.auth.github.client_secret
    );
}

#[test]
#[serial]
fn load_picks_first_available_file_in_search_order() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    write_config_file(
        temp_dir.path(),
        "switchboard.toml",
        r#"
        [http]
        port = 4242
        "#,
    );
    write_config_file(
        temp_dir.path(),
        "config/switchboard.toml",
        r#"
        [http]
        port = 5151
        "#,
    );

    let config = load().expect("configuration load should pick the first file");
    assert_eq!(config.http.port, 4242);
}

#[test]
#[serial]
fn load_merges_partial_file_with_defaults() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    write_config_file(
        temp_dir.path(),
        "switchboard.toml",
        r#"
        [http]
        port = 8181

        [database]
        max_connections = 50
        "#,
    );

    let config = load().expect("configuration load should succeed");
    let defaults = AppConfig::default();

    assert_eq!(config.http.port, 8181);
    assert_eq!(config.http.address, defaults.http.address);
    assert_eq!(config.database.max_connections, 50);
    assert_eq!(config.database.url, defaults.database.url);
    assert_eq!(
        config.orchestrator.default_model,
        defaults.orchestrator.default_model
    );
}

#[test]
#[serial]
fn load_applies_environment_overrides() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    write_config_file(
        temp_dir.path(),
        "switchboard.toml",
        r#"
        [http]
        port = 3030
        "#,
    );

    ctx.set_var("SWITCHBOARD__HTTP__PORT", "8080");

    let config = load().expect("configuration load should honour env overrides");
    assert_eq!(config.http.port, 8080);
}

#[test]
#[serial]
fn load_supports_database_url_environment_variable() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    let url = "postgres://postgres:password@localhost:5432/switchboard";
    ctx.set_var("SWITCHBOARD__DATABASE__URL", url);

    let config = load().expect("configuration load should read database env override");
    assert_eq!(config.database.url, url);
}

#[test]
#[serial]
fn load_clamps_session_ttl_to_i64_maximum() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    let oversized = (i64::MAX as u128 + 42).to_string();
    ctx.set_var("SWITCHBOARD__AUTH__SESSION_TTL_SECONDS", &oversized);

    let config = load().expect("configuration load should succeed with oversized TTL");
    assert_eq!(
        config.auth.session_ttl_seconds,
        i64::MAX as u64,
        "session TTL should be clamped to i64::MAX"
    );
}

#[test]
#[serial]
fn load_populates_openrouter_defaults_when_missing() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    write_config_file(
        temp_dir.path(),
        "switchboard.toml",
        r#"
        [orchestrator]
        default_model = "custom-model"
        "#,
    );

    let config = load().expect("configuration load should succeed with missing openrouter");
    let defaults = OrchestratorConfig::default();

    assert!(config.orchestrator.openrouter.api_key.is_none());
    assert_eq!(
        config.orchestrator.openrouter.base_url,
        defaults.openrouter.base_url
    );
    assert_eq!(
        config.orchestrator.openrouter.request_timeout_seconds,
        defaults.openrouter.request_timeout_seconds
    );
    assert_eq!(
        config.orchestrator.openrouter.title,
        defaults.openrouter.title
    );
}

#[test]
#[serial]
fn load_accepts_openrouter_api_key_from_env() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    ctx.set_var(
        "SWITCHBOARD__ORCHESTRATOR__OPENROUTER__API_KEY",
        "sk-test-key",
    );

    let config = load().expect("configuration load should read OpenRouter API key");
    assert_eq!(
        config.orchestrator.openrouter.api_key.as_deref(),
        Some("sk-test-key")
    );
}

#[test]
#[serial]
fn load_errors_on_invalid_toml_contents() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let mut ctx = TestContext::new();
    ctx.reset_environment();
    ctx.set_current_dir(temp_dir.path());

    write_config_file(
        temp_dir.path(),
        "switchboard.toml",
        r#"
        [http]
        port = "not-a-number
        "#,
    );

    let error = load().expect_err("invalid TOML should cause load to fail");
    let message = error.to_string();
    assert!(
        message.contains("invalid configuration") || message.contains("unable to build configuration"),
        "unexpected error message: {message}"
    );
}

#[test]
fn github_auth_config_defaults_to_optional_fields_none() {
    let defaults = AuthConfig::default();
    assert!(defaults.github.client_id.is_none());
    assert!(defaults.github.client_secret.is_none());
}

#[test]
fn orchestrator_config_defaults_include_provider_search_path() {
    let defaults = OrchestratorConfig::default();
    assert_eq!(defaults.provider_search_path, vec!["providers".to_string()]);
    assert_eq!(defaults.default_model, "gpt-4.1");
}

#[test]
fn http_config_defaults_match_expected_host_and_port() {
    let defaults = HttpConfig::default();
    assert_eq!(defaults.address, "127.0.0.1");
    assert_eq!(defaults.port, 7070);
}
