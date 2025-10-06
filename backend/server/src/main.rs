use std::{
    collections::HashMap, path::Path, sync::Arc, time::Duration as StdDuration, time::Instant,
};

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::{header::AUTHORIZATION, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use denkwerk::{ChatMessage, CompletionRequest, LLMError, TokenUsage};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::any::{install_default_drivers, AnyPoolOptions};
use switchboard_auth::{AuthError, AuthSession, Authenticator, User};
use switchboard_config::load as load_config;
use switchboard_orchestrator::{OpenRouterModelSummary, Orchestrator, OrchestratorError};
use tokio::{fs, net::TcpListener, signal, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod migrations {
    pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
}

const OAUTH_STATE_TTL: StdDuration = StdDuration::from_secs(600);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("failed to set tracing subscriber")?;

    info!("starting Switchboard backend");

    let config = load_config().context("failed to load configuration")?;

    // Ensure SQLx Any can talk to SQLite/Postgres before we connect.
    install_default_drivers();

    if let Some(sqlite_path) = config.database.url.strip_prefix("sqlite://") {
        if sqlite_path != ":memory:" {
            let path = Path::new(sqlite_path);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).await.with_context(|| {
                        format!("failed to create sqlite directory {}", parent.display())
                    })?;
                }
            }

            if !path.exists() {
                fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .await
                    .with_context(|| {
                        format!("failed to create sqlite database file {}", path.display())
                    })?;
            }
        }
    }

    let db_pool = AnyPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .with_context(|| format!("failed to connect to database {}", config.database.url))?;

    if config.database.url.starts_with("sqlite://") {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db_pool)
            .await
            .context("failed to enable foreign keys for sqlite")?;
    }

    migrations::MIGRATOR
        .run(&db_pool)
        .await
        .context("database migrations failed")?;

    let authenticator = Authenticator::new(db_pool.clone(), config.auth.clone());
    info!(
        github_oauth = authenticator.github_enabled(),
        "authentication subsystem ready"
    );

    let orchestrator = Arc::new(
        Orchestrator::new(&config)
            .bootstrap()
            .context("failed to bootstrap orchestrator")?,
    );

    info!(model = ?orchestrator.active_model(), "orchestrator ready");

    let state = AppState {
        orchestrator: orchestrator.clone(),
        authenticator: authenticator.clone(),
        oauth_state: OAuthStateStore::new(OAUTH_STATE_TTL),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/auth/github/login", get(github_login))
        .route("/api/auth/github/callback", post(github_callback))
        .route("/api/models", get(list_models))
        .route("/api/chat", post(chat_completion))
        .with_state(state)
        .layer(cors);

    let address = format!("{}:{}", config.http.address, config.http.port);
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind http listener on {address}"))?;

    info!(%address, "http server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("http server error")?;

    info!("backend shut down");
    Ok(())
}

#[derive(Clone)]
struct AppState {
    orchestrator: Arc<Orchestrator>,
    authenticator: Authenticator,
    oauth_state: OAuthStateStore,
}

impl AppState {
    async fn authenticate(&self, token: &str) -> Result<(User, AuthSession), ApiError> {
        self.authenticator
            .authenticate_token(token)
            .await
            .map_err(ApiError::from)
    }
}

#[derive(Clone)]
struct OAuthStateStore {
    inner: Arc<Mutex<HashMap<String, Instant>>>,
    ttl: StdDuration,
}

impl OAuthStateStore {
    fn new(ttl: StdDuration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    async fn issue(&self) -> String {
        let state = Self::random_state();
        self.store(state.clone()).await;
        state
    }

    async fn store(&self, state: String) {
        let mut guard = self.inner.lock().await;
        Self::prune(&mut guard, self.ttl);
        guard.insert(state, Instant::now());
    }

    async fn consume(&self, state: &str) -> bool {
        let mut guard = self.inner.lock().await;
        Self::prune(&mut guard, self.ttl);
        guard.remove(state).is_some()
    }

    fn random_state() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    fn prune(map: &mut HashMap<String, Instant>, ttl: StdDuration) {
        let now = Instant::now();
        map.retain(|_, created| now.duration_since(*created) <= ttl);
    }
}

impl Default for OAuthStateStore {
    fn default() -> Self {
        Self::new(OAUTH_STATE_TTL)
    }
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    prompt: String,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    model: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct GithubLoginResponse {
    authorize_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubLoginQuery {
    redirect_uri: String,
}

#[derive(Debug, Deserialize)]
struct GithubCallbackRequest {
    code: String,
    state: String,
    redirect_uri: String,
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    token: String,
    user: UserResponse,
    expires_at: String,
}

impl SessionResponse {
    fn new(session: AuthSession, user: User) -> Self {
        Self {
            token: session.token,
            user: user.into(),
            expires_at: session.expires_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id.to_string(),
            email: value.email,
            display_name: value.display_name,
        }
    }
}

#[derive(Debug, Serialize)]
struct ModelsResponse {
    models: Vec<OpenRouterModelSummary>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

impl From<LLMError> for ApiError {
    fn from(value: LLMError) -> Self {
        error!(error = ?value, "llm error");
        Self::new(StatusCode::BAD_GATEWAY, value.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        error!(error = ?value, "internal error");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

impl From<OrchestratorError> for ApiError {
    fn from(value: OrchestratorError) -> Self {
        error!(error = ?value, "orchestrator error");
        let status = match value {
            OrchestratorError::ProviderNotFound(_) => StatusCode::BAD_REQUEST,
            OrchestratorError::OpenRouterApiKeyMissing
            | OrchestratorError::OpenRouterUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self::new(status, value.to_string())
    }
}

impl From<AuthError> for ApiError {
    fn from(value: AuthError) -> Self {
        error!(error = ?value, "auth error");
        let status = match value {
            AuthError::GithubOauthDisabled => StatusCode::SERVICE_UNAVAILABLE,
            AuthError::GithubOauth(_) => StatusCode::BAD_GATEWAY,
            AuthError::InvalidCredentials
            | AuthError::SessionNotFound
            | AuthError::SessionExpired
            | AuthError::InvalidSession => StatusCode::UNAUTHORIZED,
            AuthError::UserExists => StatusCode::BAD_REQUEST,
            AuthError::Database(_) | AuthError::PasswordHash(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Self::new(status, value.to_string())
    }
}

async fn chat_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let _ = state.authenticate(&token).await?;

    let prompt = payload.prompt.trim();
    if prompt.is_empty() {
        return Err(ApiError::bad_request("prompt must not be empty"));
    }

    let model = payload
        .model
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| state.orchestrator.active_model())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "no active model configured",
            )
        })?;

    let provider = if payload
        .model
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        state.orchestrator.provider_for_model(&model)?
    } else {
        state.orchestrator.default_provider()?
    };

    let request = CompletionRequest::new(model.clone(), vec![ChatMessage::user(prompt)]);
    let completion = provider.complete(request).await?;

    let content = completion.message.text().unwrap_or_default().to_string();
    let reasoning = completion
        .reasoning
        .map(|steps| steps.into_iter().map(|step| step.content).collect());

    Ok(Json(ChatResponse {
        model,
        content,
        usage: completion.usage,
        reasoning,
    }))
}

async fn list_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ModelsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let _ = state.authenticate(&token).await?;
    let models = state.orchestrator.list_openrouter_models().await?;
    Ok(Json(ModelsResponse { models }))
}

async fn github_login(
    State(state): State<AppState>,
    Query(params): Query<GithubLoginQuery>,
) -> Result<Json<GithubLoginResponse>, ApiError> {
    if !state.authenticator.github_enabled() {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "GitHub OAuth is not configured",
        ));
    }

    let oauth_state = state.oauth_state.issue().await;
    let authorize_url = match state
        .authenticator
        .github_authorization_url(&oauth_state, &params.redirect_uri)
    {
        Ok(url) => url,
        Err(err) => {
            state.oauth_state.consume(&oauth_state).await;
            return Err(ApiError::from(err));
        }
    };

    Ok(Json(GithubLoginResponse { authorize_url }))
}

async fn github_callback(
    State(state): State<AppState>,
    Json(payload): Json<GithubCallbackRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    if !state.oauth_state.consume(&payload.state).await {
        return Err(ApiError::bad_request("invalid or expired OAuth state"));
    }

    let session = state
        .authenticator
        .login_with_github_code(&payload.code, &payload.redirect_uri)
        .await
        .map_err(ApiError::from)?;
    let user = state
        .authenticator
        .user_profile(session.user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(SessionResponse::new(session, user)))
}

fn require_bearer(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing authorization header"))?;

    let mut parts = value.split_whitespace();
    let scheme = parts.next().unwrap_or("");
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(ApiError::unauthorized("invalid authorization scheme"));
    }

    let token = parts.next().unwrap_or("");
    if token.is_empty() {
        return Err(ApiError::unauthorized("missing bearer token"));
    }

    Ok(token.to_string())
}

fn shutdown_signal() -> impl std::future::Future<Output = ()> {
    async {
        if let Err(error) = signal::ctrl_c().await {
            error!(?error, "failed to listen for shutdown signal");
        }
        info!("shutdown signal received");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn oauth_state_issue_and_consume_once() {
        let store = OAuthStateStore::new(Duration::from_secs(60));
        let state = store.issue().await;

        assert_eq!(state.len(), 32);
        assert!(store.consume(&state).await);
        assert!(!store.consume(&state).await);
    }

    #[tokio::test]
    async fn oauth_state_entry_expires_after_ttl() {
        let store = OAuthStateStore::new(Duration::from_millis(10));
        let state = "expired-state".to_string();
        store.store(state.clone()).await;

        sleep(Duration::from_millis(25)).await;

        assert!(!store.consume(&state).await);
    }

    #[test]
    fn require_bearer_extracts_token_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("bearer TOKEN123"));

        let token = require_bearer(&headers).expect("token should be extracted");
        assert_eq!(token, "TOKEN123");
    }

    #[test]
    fn require_bearer_rejects_missing_token() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer"));

        let error = require_bearer(&headers).expect_err("should reject missing token");
        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert!(error.message.contains("missing bearer token"));
    }
}
