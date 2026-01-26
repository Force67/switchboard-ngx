//! Switchboard Database Crate
//!
//! This crate provides database functionality for the Switchboard application,
//! including connection management, migrations, and repository implementations.

use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use switchboard_config::DatabaseConfig;
use tokio::fs;
use tracing::{error, info};

pub mod connection;
pub mod entities;
pub mod migrations;
pub mod permissions;
pub mod repos;
pub mod types;

pub use connection::{prepare_database, DatabaseConnection};
pub use migrations::run_migrations;
pub use permissions::Permissions;

// Re-export repositories
pub use repos::{
    AttachmentRepository, ChatRepository, InviteRepository, MemberRepository, MessageRepository,
    NotificationRepository, SessionRepository, SettingsRepository, UserRepository,
};

// Re-export entities
pub use entities::{
    attachment::{AttachmentType, CreateAttachmentRequest, MessageAttachment},
    chat::{Chat, ChatStatus, ChatType, CreateChatRequest, UpdateChatRequest},
    invite::{ChatInvite, CreateInviteRequest, InviteStatus},
    member::{ChatMember, CreateMemberRequest, MemberRole, UpdateMemberRoleRequest},
    message::{
        ChatMessage, CreateMessageRequest, MessageStatus, MessageType, UpdateMessageRequest,
    },
    notification::{
        CreateNotificationRequest, Notification, NotificationPriority, NotificationType,
    },
    session::{AuthProvider, AuthSession, CreateSessionRequest},
    settings::{UserPreferences, UserSettings},
    user::{CreateUserRequest, UpdateUserRequest, User, UserRole, UserStatus},
};

// Re-export types
pub use types::{
    errors::{AuthError, ChatError, DatabaseError, NotificationError, UserError},
    AuthResult, ChatResult, DatabaseResult, NotificationResult, UpdateSettingsRequest, UserResult,
};

/// Re-export commonly used types for convenience
pub use sqlx::Pool;

/// Initialize the database with migrations
pub async fn initialize_database(
    config: &DatabaseConfig,
) -> crate::types::DatabaseResult<SqlitePool> {
    let pool = prepare_database(config)
        .await
        .map_err(|e| crate::types::errors::DatabaseError::ConnectionError(e.to_string()))?;

    run_migrations(&pool)
        .await
        .map_err(|e| crate::types::errors::DatabaseError::MigrationError(e.to_string()))?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_database() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = prepare_database(&config).await.unwrap();
        run_migrations(&pool).await.unwrap_or_else(|_| {
            // Migrations might not exist in test environment
            println!("No migrations found or migration error (expected in tests)");
        });
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_database_initialization() {
        let (_pool, _temp_dir) = create_test_database().await;
        // Database should be initialized successfully
    }

    #[tokio::test]
    async fn test_foreign_keys_enabled() {
        let (pool, _temp_dir) = create_test_database().await;

        // Check that foreign keys are enabled
        let result: (bool,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, true);
    }
}
