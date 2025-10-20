use anyhow::anyhow;
use chrono::Utc;
use http_body_util::BodyExt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    extract::{Json, Query, State},
    http::{
        header::{
            ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS,
            ACCESS_CONTROL_REQUEST_METHOD, AUTHORIZATION, CONTENT_TYPE, ORIGIN,
        },
        Method, Request, StatusCode,
    },
    response::IntoResponse,
    Router,
};
use tower::ServiceExt;
use serde_json::{self, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use switchboard_auth::Authenticator;
use switchboard_backend_api::{
    build_router, routes, ApiError, AppState, ClientEvent, OAuthStateStore, ServerEvent,
};
use switchboard_config::AppConfig;
use switchboard_orchestrator::Orchestrator;
use tempfile::TempDir;
use tokio::sync::broadcast;
use tokio::time::sleep;
use url::Url;

type TestResult<T = ()> = anyhow::Result<T>;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

struct TestContext {
    _temp_dir: TempDir,
    pool: SqlitePool,
    state: AppState,
}

impl TestContext {
    async fn new() -> TestResult<Self> {
        Self::with_config(AppConfig::default()).await
    }

    async fn with_config(config: AppConfig) -> TestResult<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("backend_api.sqlite");
        let db_url = format!("sqlite://{}", db_path.display());

        let mut options = SqliteConnectOptions::from_str(&db_url)?;
        options = options.create_if_missing(true);
        options = options.foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        MIGRATOR.run(&pool).await?;

        let orchestrator = Arc::new(Orchestrator::new(&config));
        let authenticator = Authenticator::new(pool.clone(), config.auth.clone());
        let state = AppState::with_oauth_store(
            pool.clone(),
            orchestrator,
            authenticator,
            OAuthStateStore::default(),
            None,
        );

        Ok(Self {
            _temp_dir: temp_dir,
            pool,
            state,
        })
    }

    async fn with_github() -> TestResult<Self> {
        let mut config = AppConfig::default();
        config.auth.github.client_id = Some("test-client-id".into());
        config.auth.github.client_secret = Some("test-client-secret".into());
        Self::with_config(config).await
    }

    fn state(&self) -> AppState {
        self.state.clone()
    }

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn router(&self) -> Router {
        build_router(self.state())
    }

    async fn ensure_dev_session(&self, token: &str) -> TestResult<()> {
        self.state
            .authenticate(token)
            .await
            .map(|_| ())
            .map_err(|err| anyhow!("failed to ensure dev session: {}", err.message))?;
        Ok(())
    }

    async fn insert_user(&self, id: i64, public_id: &str) -> TestResult<()> {
        let now = Utc::now().to_rfc3339();
        let email = format!("{public_id}@example.com");
        let display = format!("User {id}");
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO users (id, public_id, email, display_name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(public_id)
        .bind(email)
        .bind(display)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    async fn create_chat(&self, public_id: &str, owner_id: i64) -> TestResult<i64> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            INSERT INTO chats (public_id, user_id, folder_id, title, is_group, chat_type, created_at, updated_at)
            VALUES (?, ?, NULL, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(public_id)
        .bind(owner_id)
        .bind(format!("Chat {public_id}"))
        .bind(false)
        .bind("direct")
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn add_chat_member(&self, chat_id: i64, user_id: i64, role: &str) -> TestResult<()> {
        let joined_at = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO chat_members (chat_id, user_id, role, joined_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(role)
        .bind(&joined_at)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    async fn insert_message(
        &self,
        chat_id: i64,
        user_id: i64,
        public_id: &str,
        content: &str,
    ) -> TestResult<i64> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"
            INSERT INTO messages (
                public_id, chat_id, user_id, content, message_type, role, model,
                thread_id, reply_to_id, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(public_id)
        .bind(chat_id)
        .bind(user_id)
        .bind(content)
        .bind("text")
        .bind("user")
        .bind(None::<String>)
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn insert_message_edit(
        &self,
        message_id: i64,
        editor_id: i64,
        old_content: &str,
        new_content: &str,
        edited_at: Option<&str>,
    ) -> TestResult<i64> {
        let timestamp = edited_at
            .map(ToString::to_string)
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let result = sqlx::query(
            r#"
            INSERT INTO message_edits (message_id, edited_by_user_id, old_content, new_content, edited_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(message_id)
        .bind(editor_id)
        .bind(old_content)
        .bind(new_content)
        .bind(&timestamp)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_rowid())
    }
}

mod router_tests {
    use super::*;

    #[tokio::test]
    async fn build_router_registers_expected_routes() -> TestResult {
        let ctx = TestContext::new().await?;
        let response = ctx
            .router()
            .oneshot(Request::builder().uri("/health").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await?.to_bytes();
        let payload: Value = serde_json::from_slice(&body)?;
        assert_eq!(payload["status"], "ok");

        Ok(())
    }

    #[tokio::test]
    async fn build_router_includes_swagger_ui_mount() -> TestResult {
        let ctx = TestContext::new().await?;
        let response = ctx
            .router()
            .oneshot(
                Request::builder()
                    .uri("/docs/openapi.json")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert!(
            content_type.contains("application/json"),
            "expected OpenAPI JSON content-type, got {}",
            content_type
        );

        let body = response.into_body().collect().await?.to_bytes();
        serde_json::from_slice::<Value>(&body)?;

        Ok(())
    }

    #[tokio::test]
    async fn cors_layer_allows_configured_methods_and_headers() -> TestResult {
        let ctx = TestContext::new().await?;
        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri("/health")
            .header(ORIGIN, "https://example.com")
            .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .header(
                ACCESS_CONTROL_REQUEST_HEADERS,
                "authorization, content-type",
            )
            .body(Body::empty())?;

        let response = ctx.router().oneshot(request).await?;
        let status = response.status();
        assert!(
            matches!(status, StatusCode::NO_CONTENT | StatusCode::OK),
            "expected CORS preflight to return 204 or 200, got {}",
            status
        );

        let allow_origin = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert_eq!(allow_origin, "*");

        let allow_methods = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_METHODS)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_uppercase();
        assert!(
            allow_methods.contains("GET") && allow_methods.contains("POST"),
            "expected allowed methods to include GET and POST, got {}",
            allow_methods
        );

        let allow_headers = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_HEADERS)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(
            allow_headers.contains("authorization") && allow_headers.contains("content-type"),
            "expected allowed headers to include authorization and content-type, got {}",
            allow_headers
        );

        Ok(())
    }
}

mod error_handling_tests {
    use super::*;
    use anyhow::anyhow;
    use switchboard_auth::AuthError;
    use switchboard_orchestrator::OrchestratorError;

    #[tokio::test]
    async fn api_error_into_response_sets_status_and_body() -> TestResult {
        let response = ApiError::bad_request("missing payload").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await?.to_bytes();
        let payload: Value = serde_json::from_slice(&body)?;
        assert_eq!(payload["error"], "missing payload");

        Ok(())
    }

    #[test]
    fn api_error_from_auth_error_maps_to_semantic_status_codes() {
        let cases = [
            (
                AuthError::GithubOauthDisabled,
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            (AuthError::SessionExpired, StatusCode::UNAUTHORIZED),
            (AuthError::InvalidCredentials, StatusCode::UNAUTHORIZED),
            (AuthError::UserExists, StatusCode::BAD_REQUEST),
            (
                AuthError::GithubOauth(anyhow!("oauth failed")),
                StatusCode::BAD_GATEWAY,
            ),
            (
                AuthError::Database(sqlx::Error::RowNotFound),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];

        for (error, expected) in cases {
            let api_error: ApiError = error.into();
            assert_eq!(
                api_error.status, expected,
                "unexpected HTTP status for {:?}",
                api_error.message
            );
        }
    }

    #[test]
    fn api_error_from_orchestrator_error_handles_openrouter_cases() {
        let unavailable: ApiError = OrchestratorError::OpenRouterUnavailable.into();
        assert_eq!(unavailable.status, StatusCode::SERVICE_UNAVAILABLE);

        let other: ApiError = OrchestratorError::ProviderIndexMissing.into();
        assert_eq!(other.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}

mod app_state_tests {
    use super::*;

    #[tokio::test]
    async fn authenticate_recovers_dev_session_in_debug_mode() -> TestResult {
        let ctx = TestContext::new().await?;
        let state = ctx.state();

        let (user, session) = state
            .authenticate("test-token")
            .await
            .expect("dev token should authenticate");
        assert_eq!(session.token, "test-token");
        assert_eq!(user.id, 1);

        let db_token: Option<String> =
            sqlx::query_scalar("SELECT token FROM sessions WHERE token = ?")
                .bind("test-token")
                .fetch_optional(ctx.pool())
                .await?;
        assert_eq!(db_token.as_deref(), Some("test-token"));

        Ok(())
    }

    #[tokio::test]
    async fn broadcast_to_chat_delivers_events_to_existing_subscribers() -> TestResult {
        let ctx = TestContext::new().await?;
        let state = ctx.state();
        let (sender, mut receiver) = broadcast::channel(4);
        {
            let mut guard = state.chat_broadcasters.lock().await;
            guard.insert("chat-1".into(), sender);
        }

        let event = ServerEvent::ChatDeleted {
            chat_id: "chat-1".into(),
        };
        state.broadcast_to_chat("chat-1", &event).await;

        let received = receiver.recv().await?;
        match received {
            ServerEvent::ChatDeleted { chat_id } => assert_eq!(chat_id, "chat-1"),
            other => panic!("unexpected event: {:?}", other),
        }

        Ok(())
    }

    #[tokio::test]
    async fn broadcast_to_users_sends_to_each_target_channel() -> TestResult {
        let ctx = TestContext::new().await?;
        let state = ctx.state();

        let (sender_a, mut rx_a) = broadcast::channel(4);
        let (sender_b, mut rx_b) = broadcast::channel(4);
        {
            let mut guard = state.user_broadcasters.lock().await;
            guard.insert(42, sender_a);
            guard.insert(43, sender_b);
        }

        let event = ServerEvent::FolderDeleted {
            folder_id: "folder-123".into(),
        };
        state.broadcast_to_users([42, 43], &event).await;

        let first = rx_a.recv().await?;
        let second = rx_b.recv().await?;
        assert!(matches!(first, ServerEvent::FolderDeleted { .. }));
        assert!(matches!(second, ServerEvent::FolderDeleted { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn oauth_state_store_issue_generates_unique_states() -> TestResult {
        let store = OAuthStateStore::new(Duration::from_secs(60));
        let first = store.issue().await;
        let second = store.issue().await;

        assert_eq!(first.len(), 32);
        assert_eq!(second.len(), 32);
        assert_ne!(first, second);
        assert!(store.consume(&first).await);
        assert!(!store.consume(&first).await);

        Ok(())
    }

    #[tokio::test]
    async fn oauth_state_store_prunes_expired_entries() -> TestResult {
        let store = OAuthStateStore::new(Duration::from_millis(10));
        store.store("stale-state".into()).await;
        sleep(Duration::from_millis(25)).await;
        assert!(!store.consume("stale-state").await);

        Ok(())
    }

    #[test]
    fn deserialize_models_accepts_scalar_and_array_payloads() {
        let single = serde_json::json!({
            "type": "message",
            "chat_id": "chat-1",
            "content": "hello",
            "models": "gpt-4"
        });
        let as_single: ClientEvent =
            serde_json::from_value(single).expect("single model payload should parse");
        match as_single {
            ClientEvent::Message { models, .. } => assert_eq!(models, vec!["gpt-4"]),
            other => panic!("unexpected event: {:?}", other),
        }

        let multiple = serde_json::json!({
            "type": "message",
            "chat_id": "chat-1",
            "content": "hello",
            "models": ["gpt-4", "gpt-4o"]
        });
        let as_multiple: ClientEvent =
            serde_json::from_value(multiple).expect("multiple models should parse");
        match as_multiple {
            ClientEvent::Message { models, .. } => {
                assert_eq!(models, vec!["gpt-4", "gpt-4o"]);
            }
            other => panic!("unexpected event: {:?}", other),
        }
    }
}

mod message_route_tests {
    use super::*;
    use axum::{
        extract::{Path, State},
        http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
        Json,
    };
    use switchboard_backend_api::routes::{
        messages::{
            create_message, delete_message, get_message_edits, get_messages, update_message,
        },
        models::{CreateMessageRequest, MessageResponse, UpdateMessageRequest},
    };
    use tokio::{
        sync::broadcast,
        time::{timeout, Duration},
    };

    fn bearer_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("valid bearer header"),
        );
        headers
    }

    fn expect_ok<T>(result: Result<T, ApiError>, context: &str) -> TestResult<T> {
        result.map_err(|err| anyhow!("{context}: {} ({})", err.message, err.status))
    }

    #[tokio::test]
    async fn get_messages_returns_history_for_member() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;

        let chat_public_id = "chat-history";
        let chat_id = ctx.create_chat(chat_public_id, 1).await?;
        ctx.add_chat_member(chat_id, 1, "owner").await?;
        ctx.insert_message(chat_id, 1, "msg-1", "hello world").await?;
        ctx.insert_message(chat_id, 1, "msg-2", "second message").await?;

        let Json(response) = expect_ok(
            get_messages(
                State(ctx.state()),
                Path(chat_public_id.to_string()),
                bearer_headers("test-token"),
            )
            .await,
            "get_messages for member",
        )?;

        assert_eq!(response.messages.len(), 2);
        assert!(
            response
                .messages
                .iter()
                .any(|message| message.content == "hello world"),
            "expected message history to include initial content"
        );
        assert!(
            response
                .messages
                .iter()
                .any(|message| message.content == "second message"),
            "expected message history to include subsequent content"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_messages_rejects_non_members() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;
        ctx.insert_user(2, "observer").await?;

        let chat_public_id = "chat-private";
        let chat_id = ctx.create_chat(chat_public_id, 2).await?;
        ctx.add_chat_member(chat_id, 2, "owner").await?;

        let error = get_messages(
            State(ctx.state()),
            Path(chat_public_id.to_string()),
            bearer_headers("test-token"),
        )
        .await
        .expect_err("non-members should not see chat messages");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        Ok(())
    }

    #[tokio::test]
    async fn create_message_persists_and_broadcasts_event() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;
        ctx.insert_user(2, "participant").await?;

        let chat_public_id = "chat-create";
        let chat_id = ctx.create_chat(chat_public_id, 1).await?;
        ctx.add_chat_member(chat_id, 1, "owner").await?;
        ctx.add_chat_member(chat_id, 2, "member").await?;

        let state = ctx.state();

        let (chat_sender, mut chat_rx) = broadcast::channel(4);
        {
            let mut guard = state.chat_broadcasters.lock().await;
            guard.insert(chat_public_id.to_string(), chat_sender.clone());
        }
        let (user_sender, mut user_rx) = broadcast::channel(4);
        let (other_sender, mut other_rx) = broadcast::channel(4);
        {
            let mut guard = state.user_broadcasters.lock().await;
            guard.insert(1, user_sender.clone());
            guard.insert(2, other_sender.clone());
        }

        let request = CreateMessageRequest {
            content: "Hello team".to_string(),
            role: "user".to_string(),
            model: Some("gpt-4".to_string()),
            message_type: None,
            thread_id: None,
            reply_to_id: None,
        };

        let Json(MessageResponse { message }) = expect_ok(
            create_message(
                State(state.clone()),
                Path(chat_public_id.to_string()),
                bearer_headers("test-token"),
                Json(request),
            )
            .await,
            "create_message",
        )?;

        assert_eq!(message.content, "Hello team");
        assert_eq!(message.role, "user");
        assert_eq!(message.model.as_deref(), Some("gpt-4"));

        let stored_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE public_id = ? AND chat_id = ?",
        )
        .bind(&message.public_id)
        .bind(chat_id)
        .fetch_one(ctx.pool())
        .await?;
        assert_eq!(stored_count, 1);

        let chat_event = timeout(Duration::from_millis(200), chat_rx.recv())
            .await
            .expect("chat broadcast timed out")?;
        assert!(
            matches!(chat_event, ServerEvent::Message { ref message_id, .. } if message_id == &message.public_id),
            "expected chat broadcast with message event"
        );

        let user_event = timeout(Duration::from_millis(200), user_rx.recv())
            .await
            .expect("user broadcast timed out")?;
        assert!(matches!(user_event, ServerEvent::Message { .. }));

        let other_event = timeout(Duration::from_millis(200), other_rx.recv())
            .await
            .expect("secondary user broadcast timed out")?;
        assert!(matches!(other_event, ServerEvent::Message { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn update_message_records_edit_and_notifies() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;

        let chat_public_id = "chat-update";
        let chat_id = ctx.create_chat(chat_public_id, 1).await?;
        ctx.add_chat_member(chat_id, 1, "owner").await?;

        let message_id = ctx
            .insert_message(chat_id, 1, "msg-edit", "original content")
            .await?;

        let state = ctx.state();
        let (chat_sender, mut chat_rx) = broadcast::channel(4);
        let (user_sender, mut user_rx) = broadcast::channel(4);
        {
            let mut guard = state.chat_broadcasters.lock().await;
            guard.insert(chat_public_id.to_string(), chat_sender.clone());
        }
        {
            let mut guard = state.user_broadcasters.lock().await;
            guard.insert(1, user_sender.clone());
        }

        let request = UpdateMessageRequest {
            content: "updated content".to_string(),
        };

        let Json(MessageResponse { message }) = expect_ok(
            update_message(
                State(state.clone()),
                Path((chat_public_id.to_string(), "msg-edit".to_string())),
                bearer_headers("test-token"),
                Json(request),
            )
            .await,
            "update_message",
        )?;

        assert_eq!(message.content, "updated content");

        let audit_row: (String, String) = sqlx::query_as(
            "SELECT old_content, new_content FROM message_edits WHERE message_id = ?",
        )
        .bind(message_id)
        .fetch_one(ctx.pool())
        .await?;
        assert_eq!(audit_row.0, "original content");
        assert_eq!(audit_row.1, "updated content");

        let chat_event = timeout(Duration::from_millis(200), chat_rx.recv())
            .await
            .expect("chat broadcast timed out")?;
        assert!(matches!(
            chat_event,
            ServerEvent::MessageUpdated { message: ref updated, .. }
            if updated.public_id == "msg-edit" && updated.content == "updated content"
        ));

        let user_event = timeout(Duration::from_millis(200), user_rx.recv())
            .await
            .expect("user broadcast timed out")?;
        assert!(matches!(user_event, ServerEvent::MessageUpdated { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn delete_message_notifies_and_removes_message() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;

        let chat_public_id = "chat-delete";
        let chat_id = ctx.create_chat(chat_public_id, 1).await?;
        ctx.add_chat_member(chat_id, 1, "owner").await?;

        ctx.insert_message(chat_id, 1, "msg-delete", "to be removed")
            .await?;

        let state = ctx.state();
        let (chat_sender, mut chat_rx) = broadcast::channel(4);
        let (user_sender, mut user_rx) = broadcast::channel(4);
        {
            let mut guard = state.chat_broadcasters.lock().await;
            guard.insert(chat_public_id.to_string(), chat_sender.clone());
        }
        {
            let mut guard = state.user_broadcasters.lock().await;
            guard.insert(1, user_sender.clone());
        }

        expect_ok(
            delete_message(
                State(state.clone()),
                Path((chat_public_id.to_string(), "msg-delete".to_string())),
                bearer_headers("test-token"),
            )
            .await,
            "delete_message",
        )?;

        let remaining: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM messages WHERE public_id = ?",
        )
        .bind("msg-delete")
        .fetch_optional(ctx.pool())
        .await?;
        assert!(remaining.is_none(), "message should be deleted");

        let chat_event = timeout(Duration::from_millis(200), chat_rx.recv())
            .await
            .expect("chat broadcast timed out")?;
        assert!(matches!(
            chat_event,
            ServerEvent::MessageDeleted { ref message_id, .. } if message_id == "msg-delete"
        ));

        let user_event = timeout(Duration::from_millis(200), user_rx.recv())
            .await
            .expect("user broadcast timed out")?;
        assert!(matches!(user_event, ServerEvent::MessageDeleted { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn get_message_edits_returns_recent_history() -> TestResult {
        let ctx = TestContext::new().await?;
        ctx.ensure_dev_session("test-token").await?;

        let chat_public_id = "chat-edits";
        let chat_id = ctx.create_chat(chat_public_id, 1).await?;
        ctx.add_chat_member(chat_id, 1, "owner").await?;

        let message_id = ctx
            .insert_message(chat_id, 1, "msg-history", "v1")
            .await?;

        ctx.insert_message_edit(
            message_id,
            1,
            "v1",
            "v2",
            Some("2024-05-01T00:00:00Z"),
        )
        .await?;
        ctx.insert_message_edit(
            message_id,
            1,
            "v2",
            "v3",
            Some("2024-06-01T00:00:00Z"),
        )
        .await?;

        let Json(response) = expect_ok(
            get_message_edits(
                State(ctx.state()),
                Path((chat_public_id.to_string(), "msg-history".to_string())),
                bearer_headers("test-token"),
            )
            .await,
            "get_message_edits",
        )?;

        assert_eq!(response.edits.len(), 2);
        assert_eq!(response.edits[0].new_content, "v3");
        assert_eq!(response.edits[1].new_content, "v2");

        Ok(())
    }
}

mod util_tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn require_bearer_rejects_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic abc123"));

        let err = switchboard_backend_api::require_bearer(&headers)
            .expect_err("expected non-bearer scheme to be rejected");
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert!(err.message.contains("invalid authorization scheme"));
    }

    #[test]
    fn require_bearer_rejects_missing_header() {
        let headers = HeaderMap::new();
        let err = switchboard_backend_api::require_bearer(&headers)
            .expect_err("expected missing header to be rejected");
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert!(err.message.contains("missing authorization header"));
    }
}

mod health_route_tests {
    use super::*;

    #[tokio::test]
    async fn health_check_returns_service_version() -> TestResult {
        let Json(response) = routes::health::health_check().await;
        assert_eq!(response.status, "ok");
        chrono::DateTime::parse_from_rfc3339(&response.timestamp).expect("valid timestamp");
        Ok(())
    }
}

mod auth_route_tests {
    use super::*;

    #[tokio::test]
    async fn github_login_requires_oauth_configuration() -> TestResult {
        let ctx = TestContext::new().await?;
        let result = routes::auth::github_login(
            State(ctx.state()),
            Query(routes::auth::GithubLoginQuery {
                redirect_uri: "https://example.com/callback".into(),
            }),
        )
        .await;

        let err = result.expect_err("expected login without configuration to fail");
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        Ok(())
    }

    #[tokio::test]
    async fn github_login_issues_state_and_authorization_url() -> TestResult {
        let ctx = TestContext::with_github().await?;
        let state = ctx.state();
        let redirect_uri = "https://example.com/callback";

        let response = routes::auth::github_login(
            State(state.clone()),
            Query(routes::auth::GithubLoginQuery {
                redirect_uri: redirect_uri.into(),
            }),
        )
        .await
        .expect("github login should succeed");

        let authorize_url = &response.0.authorize_url;
        let parsed = Url::parse(authorize_url)?;

        let state_param = parsed
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.to_string())
            .expect("state parameter to be present");
        assert_eq!(state_param.len(), 32);
        assert!(
            state.oauth_state().consume(&state_param).await,
            "state should be cached for consumption"
        );

        let redirect_param = parsed
            .query_pairs()
            .find(|(key, _)| key == "redirect_uri")
            .map(|(_, value)| value.to_string())
            .expect("redirect_uri parameter to be present");
        assert_eq!(redirect_param, redirect_uri);

        Ok(())
    }

    #[tokio::test]
    async fn github_callback_rejects_unknown_state() -> TestResult {
        let ctx = TestContext::with_github().await?;
        let result = routes::auth::github_callback(
            State(ctx.state()),
            Json(routes::auth::GithubCallbackRequest {
                code: "dummy".into(),
                state: "missing-state".into(),
                redirect_uri: "https://example.com/callback".into(),
            }),
        )
        .await;

        let err = result.expect_err("expected callback to reject unknown state");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(
            err.message.contains("invalid or expired"),
            "error message should reference invalid or expired state"
        );
        Ok(())
    }
}
