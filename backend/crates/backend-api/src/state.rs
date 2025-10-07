use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration, time::Instant};

use rand::{distributions::Alphanumeric, Rng};
use switchboard_auth::{AuthSession, Authenticator, User};
use switchboard_orchestrator::Orchestrator;
use tokio::sync::Mutex;

use crate::ApiError;

const DEFAULT_OAUTH_STATE_TTL: StdDuration = StdDuration::from_secs(600);

#[derive(Clone)]
pub struct AppState {
    orchestrator: Arc<Orchestrator>,
    authenticator: Authenticator,
    oauth_state: OAuthStateStore,
}

impl AppState {
    pub fn new(orchestrator: Arc<Orchestrator>, authenticator: Authenticator) -> Self {
        Self {
            orchestrator,
            authenticator,
            oauth_state: OAuthStateStore::default(),
        }
    }

    pub fn with_oauth_store(
        orchestrator: Arc<Orchestrator>,
        authenticator: Authenticator,
        oauth_state: OAuthStateStore,
    ) -> Self {
        Self {
            orchestrator,
            authenticator,
            oauth_state,
        }
    }

    pub fn orchestrator(&self) -> &Orchestrator {
        &self.orchestrator
    }

    pub fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }

    pub fn oauth_state(&self) -> &OAuthStateStore {
        &self.oauth_state
    }

    pub async fn authenticate(&self, token: &str) -> Result<(User, AuthSession), ApiError> {
        self.authenticator
            .authenticate_token(token)
            .await
            .map_err(ApiError::from)
    }
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
