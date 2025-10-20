use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration, time::Instant};

use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use switchboard_auth::{AuthError, AuthSession, Authenticator, User};
use switchboard_orchestrator::Orchestrator;
use tokio::sync::{broadcast, Mutex};

use crate::{
    routes::models::{Chat, ChatInvite, ChatMember, Folder, Message},
    ApiError,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    Subscribe {
        chat_id: String,
    },
    Unsubscribe {
        chat_id: String,
    },
    Message {
        chat_id: String,
        content: String,
        #[serde(default, deserialize_with = "deserialize_models")]
        models: Vec<String>,
    },
    Typing {
        chat_id: String,
        is_typing: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Hello {
        version: String,
        user_id: i64,
    },
    Subscribed {
        chat_id: String,
    },
    Unsubscribed {
        chat_id: String,
    },
    Message {
        chat_id: String,
        message_id: String,
        user_id: i64,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        timestamp: String,
        message_type: String,
    },
    Typing {
        chat_id: String,
        user_id: i64,
        is_typing: bool,
    },
    Error {
        message: String,
    },
    ChatCreated {
        chat: Chat,
    },
    ChatUpdated {
        chat: Chat,
    },
    ChatDeleted {
        chat_id: String,
    },
    FolderCreated {
        folder: Folder,
    },
    FolderUpdated {
        folder: Folder,
    },
    FolderDeleted {
        folder_id: String,
    },
    MessageUpdated {
        chat_id: String,
        message: Message,
    },
    MessageDeleted {
        chat_id: String,
        message_id: String,
    },
    InviteCreated {
        chat_id: String,
        invite: ChatInvite,
    },
    MemberUpdated {
        chat_id: String,
        member: ChatMember,
    },
    MemberRemoved {
        chat_id: String,
        user_id: i64,
    },
}

const DEFAULT_OAUTH_STATE_TTL: StdDuration = StdDuration::from_secs(600);

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    orchestrator: Arc<Orchestrator>,
    authenticator: Authenticator,
    oauth_state: OAuthStateStore,
    redis_conn: Option<ConnectionManager>,
    pub chat_broadcasters: Arc<Mutex<HashMap<String, broadcast::Sender<ServerEvent>>>>,
    pub user_broadcasters: Arc<Mutex<HashMap<i64, broadcast::Sender<ServerEvent>>>>,
}

impl AppState {
    pub fn new(
        db_pool: SqlitePool,
        orchestrator: Arc<Orchestrator>,
        authenticator: Authenticator,
        redis_conn: Option<ConnectionManager>,
    ) -> Self {
        Self {
            db_pool,
            orchestrator,
            authenticator,
            oauth_state: OAuthStateStore::default(),
            redis_conn,
            chat_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            user_broadcasters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_oauth_store(
        db_pool: SqlitePool,
        orchestrator: Arc<Orchestrator>,
        authenticator: Authenticator,
        oauth_state: OAuthStateStore,
        redis_conn: Option<ConnectionManager>,
    ) -> Self {
        Self {
            db_pool,
            orchestrator,
            authenticator,
            oauth_state,
            redis_conn,
            chat_broadcasters: Arc::new(Mutex::new(HashMap::new())),
            user_broadcasters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn orchestrator(&self) -> &Orchestrator {
        &self.orchestrator
    }

    pub fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }

    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    pub fn oauth_state(&self) -> &OAuthStateStore {
        &self.oauth_state
    }

    pub fn redis_conn(&self) -> Option<&ConnectionManager> {
        self.redis_conn.as_ref()
    }

    pub async fn get_user_broadcaster(&self, user_id: i64) -> broadcast::Sender<ServerEvent> {
        let mut broadcasters = self.user_broadcasters.lock().await;
        broadcasters
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    }

    pub async fn broadcast_to_user(&self, user_id: i64, event: &ServerEvent) {
        let sender = self.get_user_broadcaster(user_id).await;
        if let Err(err) = sender.send(event.clone()) {
            tracing::debug!(
                "failed to deliver event {:?} to user {}: {}",
                event,
                user_id,
                err
            );
        }
    }

    pub async fn broadcast_to_users(
        &self,
        user_ids: impl IntoIterator<Item = i64>,
        event: &ServerEvent,
    ) {
        for user_id in user_ids {
            self.broadcast_to_user(user_id, event).await;
        }
    }

    pub async fn broadcast_to_chat(&self, chat_public_id: &str, event: &ServerEvent) {
        let broadcaster = {
            let broadcasters = self.chat_broadcasters.lock().await;
            broadcasters.get(chat_public_id).cloned()
        };

        if let Some(sender) = broadcaster {
            if let Err(err) = sender.send(event.clone()) {
                tracing::debug!(
                    "failed to deliver chat event {:?} for chat {}: {}",
                    event,
                    chat_public_id,
                    err
                );
            }
        }
    }

    async fn ensure_dev_session(&self, token: &str) -> Result<(User, AuthSession), ApiError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);
        let now_str = now.to_rfc3339();
        let expires_at_str = expires_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO users (id, public_id, email, display_name, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1i64)
        .bind("dev-user-123")
        .bind("dev@example.com")
        .bind("Dev User")
        .bind(&now_str)
        .bind(&now_str)
        .execute(self.db_pool())
        .await
        .map_err(|error| {
            tracing::error!("failed to ensure development user: {}", error);
            ApiError::internal_server_error("failed to ensure development user")
        })?;

        sqlx::query(
            r#"
            INSERT INTO sessions (token, user_id, expires_at, created_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(token) DO UPDATE SET
                user_id = excluded.user_id,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(token)
        .bind(1i64)
        .bind(&expires_at_str)
        .bind(&now_str)
        .execute(self.db_pool())
        .await
        .map_err(|error| {
            tracing::error!("failed to upsert development session: {}", error);
            ApiError::internal_server_error("failed to ensure development session")
        })?;

        let user = self
            .authenticator()
            .user_profile(1)
            .await
            .map_err(ApiError::from)?;

        let session = AuthSession {
            token: token.to_string(),
            user_id: 1,
            expires_at,
        };

        Ok((user, session))
    }

    pub async fn authenticate(&self, token: &str) -> Result<(User, AuthSession), ApiError> {
        if cfg!(debug_assertions) && token == "test-token" {
            return self.ensure_dev_session(token).await;
        }

        match self.authenticator.authenticate_token(token).await {
            Ok(result) => Ok(result),
            Err(
                auth_error @ (AuthError::SessionNotFound
                | AuthError::SessionExpired
                | AuthError::InvalidSession),
            ) => {
                if cfg!(debug_assertions) {
                    tracing::warn!(
                        "development session for token {} missing ({}), recreating",
                        token,
                        auth_error
                    );
                    self.ensure_dev_session(token).await
                } else {
                    Err(ApiError::from(auth_error))
                }
            }
            Err(other) => Err(ApiError::from(other)),
        }
    }
}

fn deserialize_models<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ModelsField {
        List(Vec<String>),
        Single(String),
    }

    let value: Option<ModelsField> = Option::deserialize(deserializer)?;
    let models = match value {
        Some(ModelsField::List(list)) => list,
        Some(ModelsField::Single(single)) => vec![single],
        None => Vec::new(),
    };

    Ok(models)
}

#[derive(Clone)]
pub struct OAuthStateStore {
    inner: Arc<Mutex<HashMap<String, Instant>>>,
    ttl: StdDuration,
}

impl OAuthStateStore {
    pub fn new(ttl: StdDuration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn issue(&self) -> String {
        let state = Self::random_state();
        self.store(state.clone()).await;
        state
    }

    pub async fn store(&self, state: String) {
        let mut guard = self.inner.lock().await;
        Self::prune(&mut guard, self.ttl);
        guard.insert(state, Instant::now());
    }

    pub async fn consume(&self, state: &str) -> bool {
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
        Self::new(DEFAULT_OAUTH_STATE_TTL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

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
}
