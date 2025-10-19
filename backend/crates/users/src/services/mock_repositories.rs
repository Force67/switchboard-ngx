//! Mock repository implementations for testing core service functionality

use switchboard_database::{User, CreateUserRequest, UpdateUserRequest, UserRole, UserStatus};
use switchboard_database::{UserError, UserResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock user repository for testing
pub struct MockUserRepository {
    users: Arc<RwLock<HashMap<i64, User>>>,
    next_id: Arc<RwLock<i64>>,
    email_index: Arc<RwLock<HashMap<String, i64>>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            email_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_by_id(&self, user_id: i64) -> UserResult<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(&user_id).cloned())
    }

    pub async fn find_by_public_id(&self, public_id: &str) -> UserResult<Option<User>> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.public_id == public_id).cloned())
    }

    pub async fn find_by_email(&self, email: &str) -> UserResult<Option<User>> {
        let email_index = self.email_index.read().await;
        if let Some(user_id) = email_index.get(email) {
            let users = self.users.read().await;
            Ok(users.get(user_id).cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn create(&self, request: &CreateUserRequest) -> UserResult<User> {
        let mut next_id = self.next_id.write().await;
        let user_id = *next_id;
        *next_id += 1;

        let user = User {
            id: user_id,
            public_id: format!("user_{}", user_id),
            email: Some(request.email.clone()),
            username: request.username.clone(),
            display_name: Some(request.display_name.clone()),
            avatar_url: request.avatar_url.clone(),
            bio: request.bio.clone(),
            role: UserRole::User, // Default role for created users
            status: UserStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            last_login_at: None,
            email_verified: false,
            is_active: true,
        };

        // Store user
        let mut users = self.users.write().await;
        users.insert(user_id, user.clone());

        // Update email index
        let mut email_index = self.email_index.write().await;
        email_index.insert(request.email.clone(), user_id);

        Ok(user)
    }

    pub async fn update(&self, user_id: i64, request: &UpdateUserRequest) -> UserResult<User> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            if let Some(ref display_name) = request.display_name {
                user.display_name = Some(display_name.clone());
            }
            if let Some(ref avatar_url) = request.avatar_url {
                user.avatar_url = Some(avatar_url.clone());
            }
            if let Some(ref bio) = request.bio {
                user.bio = Some(bio.clone());
            }
            if let Some(ref role) = request.role {
                user.role = role.clone();
            }

            user.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(user.clone())
        } else {
            Err(UserError::UserNotFound)
        }
    }

    pub async fn delete(&self, user_id: i64) -> UserResult<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.remove(&user_id) {
            // Remove from email index
            if let Some(email) = user.email {
                let mut email_index = self.email_index.write().await;
                email_index.remove(&email);
            }
            Ok(())
        } else {
            Err(UserError::UserNotFound)
        }
    }

    pub async fn email_exists(&self, email: &str) -> UserResult<bool> {
        let email_index = self.email_index.read().await;
        Ok(email_index.contains_key(email))
    }

    pub async fn update_last_login(&self, user_id: i64) -> UserResult<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            user.last_login_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(())
        } else {
            Err(UserError::UserNotFound)
        }
    }

    pub async fn search_by_display_name(&self, query: &str, limit: u32) -> UserResult<Vec<User>> {
        let users = self.users.read().await;
        let mut results: Vec<User> = users
            .values()
            .filter(|user| {
                if let Some(ref display_name) = user.display_name {
                    display_name.to_lowercase().contains(&query.to_lowercase())
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        // Sort by display name and limit results
        results.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        results.truncate(limit as usize);
        Ok(results)
    }

    pub async fn get_user_stats(&self) -> UserResult<MockUserStats> {
        let users = self.users.read().await;
        let total_users = users.len() as i64;
        let active_users = users.values().filter(|u| u.is_active).count() as i64;

        Ok(MockUserStats {
            total_users,
            active_users,
            inactive_users: total_users - active_users,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MockUserStats {
    pub total_users: i64,
    pub active_users: i64,
    pub inactive_users: i64,
}

// Mock session repository for testing
use switchboard_database::{AuthSession, CreateSessionRequest, AuthProvider};
use switchboard_database::{AuthError, AuthResult};

pub struct MockSessionRepository {
    sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    next_id: Arc<RwLock<i64>>,
}

impl MockSessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn create(&self, request: &CreateSessionRequest) -> AuthResult<AuthSession> {
        let mut next_id = self.next_id.write().await;
        let session_id = *next_id;
        *next_id += 1;

        let session = AuthSession {
            id: session_id,
            public_id: format!("session_{}", session_id),
            user_id: request.user_id,
            token: request.token.clone(),
            provider: request.provider.clone(),
            expires_at: request.expires_at.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_accessed_at: Some(chrono::Utc::now().to_rfc3339()),
            is_active: true,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(request.token.clone(), session.clone());
        Ok(session)
    }

    pub async fn find_by_token(&self, token: &str) -> AuthResult<Option<AuthSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(token).cloned())
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> AuthResult<Vec<AuthSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.user_id == user_id && s.is_active)
            .cloned()
            .collect())
    }

    pub async fn delete_by_token(&self, token: &str) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(token).is_some() {
            Ok(())
        } else {
            Err(AuthError::AuthenticationFailed)
        }
    }

    pub async fn delete_by_user_id(&self, user_id: i64) -> AuthResult<u32> {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        sessions.retain(|_, session| session.user_id != user_id);
        let deleted_count = initial_count - sessions.len();
        Ok(deleted_count as u32)
    }

    pub async fn delete_expired(&self) -> AuthResult<u32> {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|_, session| {
            if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(&session.expires_at) {
                let expires_at_utc = expires_at.with_timezone(&chrono::Utc);
                now <= expires_at_utc
            } else {
                false // Remove sessions with invalid expiration dates
            }
        });

        let deleted_count = initial_count - sessions.len();
        Ok(deleted_count as u32)
    }

    pub async fn update_last_used(&self, token: &str) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            session.last_accessed_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(())
        } else {
            Err(AuthError::AuthenticationFailed)
        }
    }

    pub async fn count_active_sessions(&self, user_id: i64) -> AuthResult<i64> {
        let sessions = self.sessions.read().await;
        let count = sessions
            .values()
            .filter(|s| s.user_id == user_id && s.is_active)
            .count() as i64;
        Ok(count)
    }
}