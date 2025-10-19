//! Shared application state for the gateway

use std::sync::Arc;
use sqlx::SqlitePool;
use switchboard_users::{UserService, AuthService, SessionService};
use switchboard_chats::{ChatService, MessageService, MemberService, InviteService, AttachmentService};
use switchboard_database::{ChatRepository, MessageRepository, MemberRepository, InviteRepository, AttachmentRepository};
use crate::error::{GatewayError, GatewayResult};

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "default_secret_change_in_production".to_string(),
            issuer: "switchboard".to_string(),
            audience: "switchboard-users".to_string(),
        }
    }
}

/// Shared application state containing all services
#[derive(Clone)]
pub struct GatewayState {
    /// Database connection pool
    pub pool: SqlitePool,
    /// JWT configuration
    pub jwt_config: JwtConfig,
    /// User service
    pub user_service: Arc<UserService<switchboard_database::UserRepository>>,
    /// Authentication service
    pub auth_service: Arc<AuthService>,
    /// Session service
    pub session_service: Arc<SessionService>,
    /// Chat service
    pub chat_service: Arc<ChatService>,
    /// Message service
    pub message_service: Arc<MessageService>,
    /// Member service
    pub member_service: Arc<MemberService>,
    /// Invite service
    pub invite_service: Arc<InviteService>,
    /// Attachment service
    pub attachment_service: Arc<AttachmentService>,
}

impl GatewayState {
    /// Create a new gateway state with all services initialized
    pub fn new(pool: SqlitePool, jwt_config: JwtConfig) -> Self {
        // Initialize user services
        let user_service = Arc::new(UserService::new(pool.clone()));
        let auth_service = Arc::new(AuthService::new(pool.clone()));
        let session_service = Arc::new(SessionService::new(pool.clone()));

        // Initialize chat services
        let chat_service = Arc::new(ChatService::new(pool.clone()));
        let message_service = Arc::new(MessageService::new(pool.clone()));
        let member_service = Arc::new(MemberService::new(pool.clone()));
        let invite_service = Arc::new(InviteService::new(pool.clone()));
        let attachment_service = Arc::new(AttachmentService::new(pool.clone()));

        Self {
            pool,
            jwt_config,
            user_service,
            auth_service,
            session_service,
            chat_service,
            message_service,
            member_service,
            invite_service,
            attachment_service,
        }
    }

    /// Create gateway state from database URL
    pub async fn from_database_url(database_url: &str, jwt_config: JwtConfig) -> GatewayResult<Self> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| GatewayError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

        Ok(Self::new(pool, jwt_config))
    }

    /// Get a user service reference
    pub fn user_service(&self) -> &UserService<switchboard_database::UserRepository> {
        &self.user_service
    }

    /// Get an auth service reference
    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    /// Get a session service reference
    pub fn session_service(&self) -> &SessionService {
        &self.session_service
    }

    /// Get a chat service reference
    pub fn chat_service(&self) -> &ChatService {
        &self.chat_service
    }

    /// Get a message service reference
    pub fn message_service(&self) -> &MessageService {
        &self.message_service
    }

    /// Get a member service reference
    pub fn member_service(&self) -> &MemberService {
        &self.member_service
    }

    /// Get an invite service reference
    pub fn invite_service(&self) -> &InviteService {
        &self.invite_service
    }

    /// Get an attachment service reference
    pub fn attachment_service(&self) -> &AttachmentService {
        &self.attachment_service
    }
}

/// Create a gateway state with default configuration for development
pub async fn create_gateway_state() -> GatewayResult<GatewayState> {
    let jwt_config = JwtConfig::default();
    GatewayState::from_database_url("sqlite::memory:", jwt_config).await
}

/// Create a gateway state with in-memory database for testing
pub async fn create_test_gateway_state() -> GatewayResult<GatewayState> {
    create_gateway_state().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_gateway_state() {
        let state = create_test_gateway_state().await;
        assert!(state.is_ok());
    }
}