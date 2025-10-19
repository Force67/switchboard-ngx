use std::sync::Arc;

use axum::{
    body::Body,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method, Request, StatusCode,
    },
    Router,
};
use chrono::Utc;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use switchboard_backend_api::{build_router, AppState};
use switchboard_backend_runtime::BackendServices;
use switchboard_config::AppConfig;
use tempfile::TempDir;
use tower::ServiceExt;

const TEST_TOKEN: &str = "test-token";

struct TestApp {
    router: Router,
    _db_dir: TempDir,
}

impl TestApp {
    async fn new() -> Self {
        let db_dir = TempDir::new().expect("create temp dir");
        let db_path = db_dir.path().join("switchboard-test.db");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());

        let mut config = AppConfig::default();
        config.database.url = db_url.clone();
        config.database.max_connections = 5;
        config.orchestrator.provider_search_path = Vec::new();
        config.orchestrator.openrouter.api_key = Some("test-api-key".to_string());

        let services = BackendServices::initialise(&config)
            .await
            .expect("initialise backend services");

        seed_user(&services.db_pool).await.expect("seed test user");

        let state = AppState::new(
            services.db_pool.clone(),
            Arc::clone(&services.orchestrator),
            services.authenticator.clone(),
            None,
        );

        let router = build_router(state);

        Self {
            router,
            _db_dir: db_dir,
        }
    }

    async fn request(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
        token: Option<&str>,
    ) -> TestResponse {
        let app = self.router.clone();
        let mut builder = Request::builder().method(method).uri(uri);

        if let Some(token) = token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let body = if let Some(json_body) = body {
            let bytes = serde_json::to_vec(&json_body).expect("serialize request body");
            builder = builder.header(CONTENT_TYPE, "application/json");
            Body::from(bytes)
        } else {
            Body::empty()
        };

        let response = app
            .oneshot(builder.body(body).expect("build request"))
            .await
            .expect("dispatch request");

        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("collect response body")
            .to_bytes();
        let text = String::from_utf8(bytes.to_vec()).unwrap_or_default();
        let json = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::Null)
        };

        TestResponse { status, text, json }
    }

    async fn authed_request(&self, method: Method, uri: &str, body: Option<Value>) -> TestResponse {
        self.request(method, uri, body, Some(TEST_TOKEN)).await
    }
}

struct TestResponse {
    status: StatusCode,
    text: String,
    json: Value,
}

async fn seed_user(pool: &SqlitePool) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, public_id, email, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(1_i64)
    .bind("test-user")
    .bind(Some("test@example.com".to_string()))
    .bind(Some("Test User".to_string()))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
async fn health_check_returns_ok() {
    let app = TestApp::new().await;

    let response = app.request(Method::GET, "/health", None, None).await;

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(
        response.json.get("status").and_then(Value::as_str),
        Some("ok")
    );
    assert!(
        response
            .json
            .get("timestamp")
            .and_then(Value::as_str)
            .is_some(),
        "health response should include timestamp"
    );
}

#[tokio::test]
async fn folders_require_authentication() {
    let app = TestApp::new().await;

    let response = app.request(Method::GET, "/api/folders", None, None).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
    assert!(
        response.text.contains("missing authorization header")
            || response.text.contains("invalid authorization"),
        "unexpected error message: {}",
        response.text
    );
}

#[tokio::test]
async fn folder_crud_flow() {
    let app = TestApp::new().await;

    let create_response = app
        .authed_request(
            Method::POST,
            "/api/folders",
            Some(json!({
                "name": "Projects",
                "color": "#FF00AA",
                "parent_id": Value::Null
            })),
        )
        .await;

    assert_eq!(create_response.status, StatusCode::OK);
    let folder = create_response
        .json
        .get("folder")
        .cloned()
        .expect("folder payload");
    let folder_id = folder
        .get("public_id")
        .and_then(Value::as_str)
        .expect("public_id")
        .to_string();

    assert_eq!(folder.get("name").and_then(Value::as_str), Some("Projects"));
    assert_eq!(folder.get("color").and_then(Value::as_str), Some("#FF00AA"));

    let list_response = app.authed_request(Method::GET, "/api/folders", None).await;

    assert_eq!(list_response.status, StatusCode::OK);
    let folders = list_response
        .json
        .get("folders")
        .and_then(Value::as_array)
        .cloned()
        .expect("folders array");
    assert_eq!(folders.len(), 1);
    assert_eq!(
        folders[0].get("public_id").and_then(Value::as_str),
        Some(folder_id.as_str())
    );

    let get_response = app
        .authed_request(Method::GET, &format!("/api/folders/{}", folder_id), None)
        .await;

    assert_eq!(get_response.status, StatusCode::OK);
    assert_eq!(
        get_response
            .json
            .get("folder")
            .and_then(|value| value.get("public_id"))
            .and_then(Value::as_str),
        Some(folder_id.as_str())
    );
}

#[tokio::test]
async fn chat_creation_persists_messages() {
    let app = TestApp::new().await;

    let folder_id = app
        .authed_request(
            Method::POST,
            "/api/folders",
            Some(json!({
                "name": "Workspace",
                "color": Value::Null,
                "parent_id": Value::Null
            })),
        )
        .await
        .json
        .get("folder")
        .and_then(|folder| folder.get("public_id"))
        .and_then(Value::as_str)
        .expect("folder id")
        .to_string();

    let chat_response = app
        .authed_request(
            Method::POST,
            "/api/chats",
            Some(json!({
                "title": "General",
                "messages": [
                    {
                        "role": "user",
                        "content": "Hello everyone!",
                        "model": Value::Null
                    }
                ],
                "folder_id": folder_id,
                "chat_type": "direct"
            })),
        )
        .await;

    assert_eq!(chat_response.status, StatusCode::OK);
    let chat_id = chat_response
        .json
        .get("chat")
        .and_then(|chat| chat.get("public_id"))
        .and_then(Value::as_str)
        .expect("chat id")
        .to_string();

    let chats_response = app.authed_request(Method::GET, "/api/chats", None).await;
    assert_eq!(chats_response.status, StatusCode::OK);
    let chats = chats_response
        .json
        .get("chats")
        .and_then(Value::as_array)
        .cloned()
        .expect("chats array");
    assert_eq!(chats.len(), 1);
    assert_eq!(
        chats[0].get("public_id").and_then(Value::as_str),
        Some(chat_id.as_str())
    );

    let messages_json = chats[0]
        .get("messages")
        .and_then(Value::as_str)
        .expect("messages payload");
    let messages: Vec<Value> = serde_json::from_str(messages_json).expect("parse messages array");
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].get("content").and_then(Value::as_str),
        Some("Hello everyone!")
    );

    let detail_response = app
        .authed_request(Method::GET, &format!("/api/chats/{}", chat_id), None)
        .await;
    assert_eq!(detail_response.status, StatusCode::OK);
    assert_eq!(
        detail_response
            .json
            .get("chat")
            .and_then(|chat| chat.get("title"))
            .and_then(Value::as_str),
        Some("General")
    );
}
