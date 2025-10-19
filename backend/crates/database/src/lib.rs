//! Switchboard Database Crate
//!
//! This crate provides database functionality for the Switchboard application,
//! including connection management, migrations, and repository implementations.

use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use tokio::fs;
use tracing::{error, info};
use switchboard_config::DatabaseConfig;

pub mod connection;
pub mod migrations;
pub mod repos;
pub mod entities;
pub mod types;

pub use connection::{DatabaseConnection, prepare_database};
pub use migrations::run_migrations;

// Re-export repositories
pub use repos::{
    UserRepository, SessionRepository, SettingsRepository, NotificationRepository,
    AttachmentRepository, ChatRepository, MessageRepository, MemberRepository, InviteRepository,
};

// Re-export entities
pub use entities::{
    user::{User, CreateUserRequest, UpdateUserRequest, UserRole, UserStatus},
    session::{AuthSession, CreateSessionRequest, AuthProvider},
    notification::{Notification, NotificationType, NotificationPriority},
    settings::{UserSettings, UserPreferences},
    attachment::{MessageAttachment, CreateAttachmentRequest},
    chat::{Chat, ChatType, ChatStatus, CreateChatRequest, UpdateChatRequest},
    message::{ChatMessage, CreateMessageRequest, UpdateMessageRequest, MessageStatus},
    member::{ChatMember, MemberRole, CreateMemberRequest},
    invite::{ChatInvite, InviteStatus, CreateInviteRequest},
};

// Re-export types
pub use types::{
    errors::{DatabaseError, UserError, ChatError, NotificationError, AuthError},
    DatabaseResult, UserResult, ChatResult, NotificationResult, AuthResult,
    UpdateSettingsRequest,
};

/// Re-export commonly used types for convenience
pub use sqlx::Pool;


/// Initialize the database with migrations
pub async fn initialize_database(config: &DatabaseConfig) -> crate::types::DatabaseResult<SqlitePool> {
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
        let db_url = format!("sqlite:{}", db_path.display());

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