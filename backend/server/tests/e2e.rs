use std::sync::Arc;

use axum::{
    body::Body,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method, Request, StatusCode,
    },
    Router,
};
use chrono::{Duration, Utc};
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
    pool: SqlitePool,
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
            pool: services.db_pool.clone(),
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

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn create_user_with_session(
        &self,
        email: &str,
        display_name: &str,
        token: &str,
    ) -> i64 {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let public_id = format!("user-{}", token);

        sqlx::query(
            r#"
            INSERT INTO users (public_id, email, display_name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(email) DO UPDATE SET
                display_name = excluded.display_name,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&public_id)
        .bind(email)
        .bind(display_name)
        .bind(&now_str)
        .bind(&now_str)
        .execute(self.pool())
        .await
        .expect("insert or update user for session");

        let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(self.pool())
            .await
            .expect("fetch user id");

        let expires_at = (now + Duration::hours(1)).to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO sessions (user_id, token, created_at, expires_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(token) DO UPDATE SET
                user_id = excluded.user_id,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(&now_str)
        .bind(&expires_at)
        .execute(self.pool())
        .await
        .expect("insert or update session");

        user_id
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

#[cfg(debug_assertions)]
#[tokio::test]
async fn dev_token_endpoint_issues_session() {
    let app = TestApp::new().await;

    let response = app.request(Method::GET, "/api/auth/dev/token", None, None).await;

    assert_eq!(response.status, StatusCode::OK);
    let token = response
        .json
        .get("token")
        .and_then(Value::as_str)
        .expect("session token from dev endpoint")
        .to_string();

    let user = response
        .json
        .get("user")
        .and_then(Value::as_object)
        .expect("user payload from dev token");
    assert_eq!(user.get("id").and_then(Value::as_str), Some("dev-user-123"));

    let authed_response = app
        .request(Method::GET, "/api/chats", None, Some(token.as_str()))
        .await;
    assert_eq!(authed_response.status, StatusCode::OK);
    assert!(
        authed_response
            .json
            .get("chats")
            .map(Value::is_array)
            .unwrap_or(false),
        "expected dev token to authorise API access"
    );
}

#[tokio::test]
async fn message_lifecycle_records_edits_and_deletion() {
    let app = TestApp::new().await;

    let chat_response = app
        .authed_request(
            Method::POST,
            "/api/chats",
            Some(json!({
                "title": "Lifecycle Chat",
                "messages": [],
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
        .expect("chat public id for lifecycle test")
        .to_string();

    let message_response = app
        .authed_request(
            Method::POST,
            &format!("/api/chats/{}/messages", chat_id),
            Some(json!({
                "content": "Initial note",
                "role": "user",
                "model": Value::Null,
                "message_type": Value::Null,
                "thread_id": Value::Null,
                "reply_to_id": Value::Null
            })),
        )
        .await;
    assert_eq!(message_response.status, StatusCode::OK);
    let message = message_response
        .json
        .get("message")
        .cloned()
        .expect("message payload from create");
    let message_id = message
        .get("public_id")
        .and_then(Value::as_str)
        .expect("message public id")
        .to_string();
    assert_eq!(
        message.get("content").and_then(Value::as_str),
        Some("Initial note")
    );

    let history = app
        .authed_request(Method::GET, &format!("/api/chats/{}/messages", chat_id), None)
        .await;
    assert_eq!(history.status, StatusCode::OK);
    let messages = history
        .json
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .expect("messages array after creation");
    assert_eq!(messages.len(), 1);

    let update_response = app
        .authed_request(
            Method::PUT,
            &format!("/api/chats/{}/messages/{}", chat_id, message_id),
            Some(json!({ "content": "Edited note" })),
        )
        .await;
    assert_eq!(update_response.status, StatusCode::OK);
    assert_eq!(
        update_response
            .json
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str),
        Some("Edited note")
    );

    let edits_response = app
        .authed_request(
            Method::GET,
            &format!(
                "/api/chats/{}/messages/{}/edits",
                chat_id, message_id
            ),
            None,
        )
        .await;
    assert_eq!(edits_response.status, StatusCode::OK);
    let edits = edits_response
        .json
        .get("edits")
        .and_then(Value::as_array)
        .cloned()
        .expect("message edits array");
    assert_eq!(edits.len(), 1);
    let edit = &edits[0];
    assert_eq!(
        edit.get("old_content").and_then(Value::as_str),
        Some("Initial note")
    );
    assert_eq!(
        edit.get("new_content").and_then(Value::as_str),
        Some("Edited note")
    );

    let delete_response = app
        .authed_request(
            Method::DELETE,
            &format!("/api/chats/{}/messages/{}", chat_id, message_id),
            None,
        )
        .await;
    assert_eq!(delete_response.status, StatusCode::OK);

    let post_delete = app
        .authed_request(Method::GET, &format!("/api/chats/{}/messages", chat_id), None)
        .await;
    assert_eq!(post_delete.status, StatusCode::OK);
    let remaining = post_delete
        .json
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .expect("messages array after deletion");
    assert!(
        remaining.is_empty(),
        "expected message deletion to remove message"
    );
}

#[tokio::test]
async fn group_invite_flow_allows_acceptance() {
    let app = TestApp::new().await;

    let chat_response = app
        .authed_request(
            Method::POST,
            "/api/chats",
            Some(json!({
                "title": "Team Chat",
                "messages": [],
                "chat_type": "group"
            })),
        )
        .await;
    assert_eq!(chat_response.status, StatusCode::OK);
    let chat_id = chat_response
        .json
        .get("chat")
        .and_then(|chat| chat.get("public_id"))
        .and_then(Value::as_str)
        .expect("chat public id for group invite")
        .to_string();

    let invite_email = "invitee@example.com";
    let invite_response = app
        .authed_request(
            Method::POST,
            &format!("/api/chats/{}/invites", chat_id),
            Some(json!({ "email": invite_email })),
        )
        .await;
    assert_eq!(
        invite_response.status,
        StatusCode::OK,
        "create invite error payload: {}",
        invite_response.text
    );
    let invite = invite_response
        .json
        .get("invite")
        .cloned()
        .expect("invite payload");
    assert_eq!(
        invite.get("status").and_then(Value::as_str),
        Some("pending")
    );
    let invite_id = invite
        .get("id")
        .and_then(Value::as_i64)
        .expect("invite id for acceptance");

    let invited_user_token = "invite-token";
    let invited_user_id = app
        .create_user_with_session(invite_email, "Invited User", invited_user_token)
        .await;

    let accept_response = app
        .request(
            Method::POST,
            &format!("/api/invites/{}/accept", invite_id),
            None,
            Some(invited_user_token),
        )
        .await;
    assert_eq!(
        accept_response.status,
        StatusCode::OK,
        "invite acceptance error payload: {}",
        accept_response.text
    );

    let members_response = app
        .request(
            Method::GET,
            &format!("/api/chats/{}/members", chat_id),
            None,
            Some(invited_user_token),
        )
        .await;
    assert_eq!(members_response.status, StatusCode::OK);
    let members = members_response
        .json
        .get("members")
        .and_then(Value::as_array)
        .cloned()
        .expect("chat members after invite acceptance");
    assert_eq!(members.len(), 2);
    assert!(
        members
            .iter()
            .any(|member| member.get("user_id").and_then(Value::as_i64) == Some(invited_user_id)),
        "expected invited user to be part of chat after acceptance"
    );

    let invites_response = app
        .authed_request(
            Method::GET,
            &format!("/api/chats/{}/invites", chat_id),
            None,
        )
        .await;
    assert_eq!(invites_response.status, StatusCode::OK);
    let invites = invites_response
        .json
        .get("invites")
        .and_then(Value::as_array)
        .cloned()
        .expect("invites list after acceptance");
    assert_eq!(invites.len(), 1);
    assert_eq!(
        invites[0].get("status").and_then(Value::as_str),
        Some("accepted")
    );
}

#[tokio::test]
async fn notifications_endpoints_update_read_state() {
    let app = TestApp::new().await;

    let notify_token = "notify-token";
    let notify_user_id = app
        .create_user_with_session("notify@example.com", "Notify User", notify_token)
        .await;

    let base_time = Utc::now();
    for (index, read) in [false, false, true].into_iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, title, body, read, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(notify_user_id)
        .bind("chat")
        .bind(format!("Notification {}", index))
        .bind(format!("Body {}", index))
        .bind(read)
        .bind((base_time + Duration::minutes(index as i64)).to_rfc3339())
        .execute(app.pool())
        .await
        .expect("insert notification row");
    }

    let unread_count = app
        .request(
            Method::GET,
            "/api/notifications/unread-count",
            None,
            Some(notify_token),
        )
        .await;
    assert_eq!(unread_count.status, StatusCode::OK);
    assert_eq!(
        unread_count
            .json
            .get("unread_count")
            .and_then(Value::as_i64),
        Some(2)
    );

    let unread_list = app
        .request(
            Method::GET,
            "/api/notifications?unread_only=true",
            None,
            Some(notify_token),
        )
        .await;
    assert_eq!(unread_list.status, StatusCode::OK);
    let unread_notifications = unread_list
        .json
        .get("notifications")
        .and_then(Value::as_array)
        .cloned()
        .expect("unread notifications array");
    assert_eq!(unread_notifications.len(), 2);
    let target_notification_id = unread_notifications[0]
        .get("id")
        .and_then(Value::as_i64)
        .expect("target notification id");

    let mark_read = app
        .request(
            Method::PUT,
            &format!("/api/notifications/{}", target_notification_id),
            Some(json!({ "read": true })),
            Some(notify_token),
        )
        .await;
    assert_eq!(mark_read.status, StatusCode::OK);
    assert_eq!(
        mark_read
            .json
            .get("notification")
            .and_then(|notification| notification.get("read"))
            .and_then(Value::as_bool),
        Some(true)
    );

    let post_update_count = app
        .request(
            Method::GET,
            "/api/notifications/unread-count",
            None,
            Some(notify_token),
        )
        .await;
    assert_eq!(post_update_count.status, StatusCode::OK);
    assert_eq!(
        post_update_count
            .json
            .get("unread_count")
            .and_then(Value::as_i64),
        Some(1)
    );

    let all_notifications = app
        .request(Method::GET, "/api/notifications", None, Some(notify_token))
        .await;
    assert_eq!(all_notifications.status, StatusCode::OK);
    let notifications = all_notifications
        .json
        .get("notifications")
        .and_then(Value::as_array)
        .cloned()
        .expect("all notifications array");
    assert_eq!(notifications.len(), 3);
}
