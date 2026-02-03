//! Shared application state for the gateway

use crate::error::{GatewayError, GatewayResult};
use sqlx::SqlitePool;
use std::sync::Arc;
use switchboard_auth::Authenticator;
use switchboard_config::{AuthConfig, WebSearchConfig};
use switchboard_database::{
    AttachmentRepository, ChatRepository, InviteRepository, MemberRepository, MessageRepository,
    Permissions,
};
use switchboard_orchestrator::Orchestrator;
use switchboard_users::{AuthService, NotificationService, SessionService, UserService};

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

/// Shared application state containing all services and repositories
#[derive(Clone)]
pub struct GatewayState {
    /// Database connection pool
    pub pool: SqlitePool,
    /// JWT configuration
    pub jwt_config: JwtConfig,
    /// Authenticator for session/token handling
    pub authenticator: Arc<Authenticator>,
    /// User service
    pub user_service: Arc<UserService<switchboard_database::UserRepository>>,
    /// Authentication service
    pub auth_service: Arc<AuthService>,
    /// Session service
    pub session_service: Arc<SessionService>,
    /// Notification service
    pub notification_service: Arc<NotificationService>,
    /// Chat repository
    pub chat_repo: ChatRepository,
    /// Message repository
    pub message_repo: MessageRepository,
    /// Member repository
    pub member_repo: MemberRepository,
    /// Invite repository
    pub invite_repo: InviteRepository,
    /// Attachment repository
    pub attachment_repo: AttachmentRepository,
    /// Optional orchestrator for model listings and provider lookups
    pub orchestrator: Option<Arc<Orchestrator>>,
    /// Web search configuration
    web_search_config: Option<WebSearchConfig>,
}

impl GatewayState {
    /// Create a new gateway state with all services initialized
    pub fn new(
        pool: SqlitePool,
        authenticator: Arc<Authenticator>,
        jwt_config: JwtConfig,
        orchestrator: Option<Arc<Orchestrator>>,
    ) -> Self {
        Self::with_web_search(pool, authenticator, jwt_config, orchestrator, None)
    }

    /// Create a new gateway state with web search configuration
    pub fn with_web_search(
        pool: SqlitePool,
        authenticator: Arc<Authenticator>,
        jwt_config: JwtConfig,
        orchestrator: Option<Arc<Orchestrator>>,
        web_search_config: Option<WebSearchConfig>,
    ) -> Self {
        // Initialize user services
        let user_service = Arc::new(UserService::new(pool.clone()));
        let auth_service = Arc::new(AuthService::new(pool.clone()));
        let session_service = Arc::new(SessionService::new(pool.clone()));
        let notification_service = Arc::new(NotificationService::new(pool.clone()));

        // Initialize repositories
        let chat_repo = ChatRepository::new(pool.clone());
        let message_repo = MessageRepository::new(pool.clone());
        let member_repo = MemberRepository::new(pool.clone());
        let invite_repo = InviteRepository::new(pool.clone());
        let attachment_repo = AttachmentRepository::new(pool.clone());

        Self {
            pool,
            jwt_config,
            authenticator,
            user_service,
            auth_service,
            session_service,
            notification_service,
            chat_repo,
            message_repo,
            member_repo,
            invite_repo,
            attachment_repo,
            orchestrator,
            web_search_config,
        }
    }

    /// Create gateway state from database URL
    pub async fn from_database_url(
        database_url: &str,
        jwt_config: JwtConfig,
    ) -> GatewayResult<Self> {
        let pool = SqlitePool::connect(database_url).await.map_err(|e| {
            GatewayError::DatabaseError(format!("Failed to connect to database: {}", e))
        })?;

        let authenticator = Arc::new(Authenticator::new(pool.clone(), AuthConfig::default()));

        Ok(Self::with_web_search(
            pool,
            authenticator,
            jwt_config,
            None,
            None,
        ))
    }

    /// Get permission checking utilities
    pub fn permissions(&self) -> Permissions<'_> {
        Permissions::new(&self.member_repo)
    }

    /// Get a user service reference
    pub fn user_service(&self) -> &UserService<switchboard_database::UserRepository> {
        &self.user_service
    }

    /// Get authenticator reference
    pub fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }

    /// Get a session service reference
    pub fn session_service(&self) -> &SessionService {
        &self.session_service
    }

    /// Get orchestrator reference if available
    pub fn orchestrator(&self) -> Option<&Orchestrator> {
        self.orchestrator.as_deref()
    }

    /// Get web search configuration if available
    pub fn web_search_config(&self) -> Option<&WebSearchConfig> {
        self.web_search_config.as_ref()
    }

    // =========================================================================
    // Service trait accessors (used by generated REST handlers)
    // =========================================================================

    /// Get health service
    pub fn health_service(&self) -> crate::rest::services::HealthServiceImpl<'_> {
        crate::rest::services::HealthServiceImpl::new(self)
    }

    /// Get auth service (trait impl for generated REST handlers)
    pub fn auth_service(&self) -> crate::rest::services::AuthServiceImpl<'_> {
        crate::rest::services::AuthServiceImpl::new(self)
    }

    /// Get folders service
    pub fn folders_service(&self) -> crate::rest::services::FoldersServiceImpl<'_> {
        crate::rest::services::FoldersServiceImpl::new(self)
    }

    /// Get chats service
    pub fn chats_service(&self) -> crate::rest::services::ChatsServiceImpl<'_> {
        crate::rest::services::ChatsServiceImpl::new(self)
    }

    /// Get messages service
    pub fn messages_service(&self) -> crate::rest::services::MessagesServiceImpl<'_> {
        crate::rest::services::MessagesServiceImpl::new(self)
    }

    /// Get members service
    pub fn members_service(&self) -> crate::rest::services::MembersServiceImpl<'_> {
        crate::rest::services::MembersServiceImpl::new(self)
    }

    /// Get attachments service
    pub fn attachments_service(&self) -> crate::rest::services::AttachmentsServiceImpl<'_> {
        crate::rest::services::AttachmentsServiceImpl::new(self)
    }

    /// Get invites service
    pub fn invites_service(&self) -> crate::rest::services::InvitesServiceImpl<'_> {
        crate::rest::services::InvitesServiceImpl::new(self)
    }

    /// Get notifications service (trait impl for generated REST handlers)
    pub fn notifications_service(&self) -> crate::rest::services::NotificationsServiceImpl<'_> {
        crate::rest::services::NotificationsServiceImpl::new(self)
    }

    /// Get permissions service (trait impl)
    pub fn permissions_service(&self) -> crate::rest::services::PermissionsServiceImpl<'_> {
        crate::rest::services::PermissionsServiceImpl::new(self)
    }

    /// Get models service
    pub fn models_service(&self) -> crate::rest::services::ModelsServiceImpl<'_> {
        crate::rest::services::ModelsServiceImpl::new(self)
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
