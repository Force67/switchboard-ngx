//! Shared application state for the gateway

use std::sync::Arc;
use sqlx::SqlitePool;
use switchboard_database::DatabaseConfig;
use switchboard_users::{
    UserService, AuthService, SessionService,
    services::repositories::UserRepository,
};
use switchboard_chats::{
    ChatService, MessageService, MemberService, InviteService, AttachmentService,
};
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
    pub user_service: Arc<UserService<UserRepository>>,
    /// Authentication service
    pub auth_service: Arc<AuthService>,
    /// Session service
    pub session_service: Arc<SessionService>,
    /// Chat service
    pub chat_service: Arc<ChatService<switchboard_database::ChatRepository>>,
    /// Message service
    pub message_service: Arc<MessageService<switchboard_database::MessageRepository>>,
    /// Member service
    pub member_service: Arc<MemberService<switchboard_database::MemberRepository>>,
    /// Invite service
    pub invite_service: Arc<InviteService<switchboard_database::InviteRepository>>,
    /// Attachment service
    pub attachment_service: Arc<AttachmentService<switchboard_database::AttachmentRepository>>,
}

impl GatewayState {
    /// Create a new gateway state with all services initialized
    pub async fn new(pool: SqlitePool, jwt_config: JwtConfig) -> GatewayResult<Self> {
        // Initialize user services
        let user_repository = UserRepository::new(pool.clone());
        let user_service = Arc::new(UserService::new(user_repository.clone()));

        let auth_service = Arc::new(AuthService::new(
            pool.clone(),
            &jwt_config.secret,
            jwt_config.issuer.clone(),
            jwt_config.audience.clone(),
        ));

        let session_service = Arc::new(SessionService::new(pool.clone()));

        // Initialize chat services
        let chat_service = Arc::new(ChatService::new(switchboard_database::ChatRepository::new(pool.clone())));
        let message_service = Arc::new(MessageService::new(switchboard_database::MessageRepository::new(pool.clone())));
        let member_service = Arc::new(MemberService::new(switchboard_database::MemberRepository::new(pool.clone())));
        let invite_service = Arc::new(InviteService::new(switchboard_database::InviteRepository::new(pool.clone())));
        let attachment_service = Arc::new(AttachmentService::new(switchboard_database::AttachmentRepository::new(pool.clone())));

        Ok(Self {
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
        })
    }

    /// Create gateway state from database configuration
    pub async fn from_config(config: &DatabaseConfig, jwt_config: JwtConfig) -> GatewayResult<Self> {
        let pool = switchboard_database::initialize_database(config)
            .await
            .map_err(|e| GatewayError::DatabaseError(format!("Failed to initialize database: {}", e)))?;

        Self::new(pool, jwt_config).await
    }

    /// Get a user service reference
    pub fn user_service(&self) -> &UserService<UserRepository> {
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
    pub fn chat_service(&self) -> &ChatService<switchboard_database::ChatRepository> {
        &self.chat_service
    }

    /// Get a message service reference
    pub fn message_service(&self) -> &MessageService<switchboard_database::MessageRepository> {
        &self.message_service
    }

    /// Get a member service reference
    pub fn member_service(&self) -> &MemberService<switchboard_database::MemberRepository> {
        &self.member_service
    }

    /// Get an invite service reference
    pub fn invite_service(&self) -> &InviteService<switchboard_database::InviteRepository> {
        &self.invite_service
    }

    /// Get an attachment service reference
    pub fn attachment_service(&self) -> &AttachmentService<switchboard_database::AttachmentRepository> {
        &self.attachment_service
    }
}

/// Create a gateway state with default configuration for development
pub async fn create_gateway_state() -> GatewayResult<GatewayState> {
    let config = DatabaseConfig {
        database_url: "sqlite::memory:".to_string(),
        max_connections: 10,
        min_connections: 1,
        acquire_timeout_seconds: 30,
        idle_timeout_seconds: 600,
        max_lifetime_seconds: 1800,
    };

    let jwt_config = JwtConfig::default();
    GatewayState::from_config(&config, jwt_config).await
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

        let state = state.unwrap();
        // Test that all services are accessible
        assert!(state.user_service().get_user_count().await.is_ok());
    }
}