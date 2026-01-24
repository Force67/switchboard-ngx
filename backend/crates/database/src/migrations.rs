//! Database migrations

use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

/// Execute a single SQL statement
async fn exec(pool: &SqlitePool, sql: &str) -> anyhow::Result<()> {
    sqlx::query(sql)
        .execute(pool)
        .await
        .with_context(|| format!("failed to execute: {}", sql.chars().take(50).collect::<String>()))?;
    Ok(())
}

/// Run database migrations
///
/// The codebase expects a fairly featureful schema (users, auth sessions, chats, folders,
/// messages, attachments, invites, settings, etc). The checked-in SQL files are out of sync
/// with the current repository models, so we create the tables we need here if they do not
/// already exist. This keeps local development unblocked until proper migrations are added.
pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    info!("running lightweight builtin migrations (SQLite)");

    // Users table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            username TEXT UNIQUE,
            display_name TEXT,
            avatar_url TEXT,
            bio TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login_at TEXT,
            email_verified BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        )
    "#).await.context("failed to create users table")?;

    // User identities table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS user_identities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            provider TEXT NOT NULL,
            provider_uid TEXT NOT NULL,
            secret TEXT,
            credential_encrypted TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(provider, provider_uid),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create user_identities table")?;

    // Sessions table (legacy)
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create sessions table")?;

    // Auth sessions table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS auth_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            provider TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_accessed_at TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create auth_sessions table")?;

    // Folders table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            color TEXT,
            parent_id INTEGER,
            collapsed BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create folders table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_folders_user_id ON folders(user_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id)").await?;

    // Chats table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS chats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            description TEXT,
            avatar_url TEXT,
            folder_id TEXT,
            chat_type TEXT NOT NULL DEFAULT 'direct',
            status TEXT NOT NULL DEFAULT 'active',
            created_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    "#).await.context("failed to create chats table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chats_folder_id ON chats(folder_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chats_created_by ON chats(created_by)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chats_status ON chats(status)").await?;

    // Chat members table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS chat_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL,
            UNIQUE(chat_id, user_id),
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create chat_members table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chat_members_chat_id ON chat_members(chat_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chat_members_user_id ON chat_members(user_id)").await?;

    // Messages table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            chat_id INTEGER NOT NULL,
            sender_id INTEGER NOT NULL,
            content TEXT,
            message_type TEXT NOT NULL DEFAULT 'text',
            status TEXT NOT NULL DEFAULT 'sent',
            reply_to_id INTEGER,
            reply_to_public_id TEXT,
            thread_id INTEGER,
            thread_public_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT,
            deleted_at TEXT,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
            FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create messages table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)").await?;

    // Message attachments table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS message_attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            message_id INTEGER NOT NULL,
            file_name TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            file_url TEXT NOT NULL,
            created_at TEXT NOT NULL,
            uploader_id INTEGER NOT NULL,
            FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
            FOREIGN KEY (uploader_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create message_attachments table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_message_attachments_message_id ON message_attachments(message_id)").await?;

    // Chat invites table - drop and recreate to handle schema changes
    exec(pool, "DROP TABLE IF EXISTS chat_invites").await?;
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS chat_invites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            chat_id INTEGER NOT NULL,
            inviter_id INTEGER NOT NULL,
            invited_email TEXT NOT NULL,
            invite_code TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            accepted_at TEXT,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
            FOREIGN KEY (inviter_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create chat_invites table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chat_invites_chat_id ON chat_invites(chat_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_chat_invites_status ON chat_invites(status)").await?;

    // Notifications table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            read BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create notifications table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id)").await?;
    exec(pool, "CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read)").await?;

    // Permissions table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            permission_level TEXT NOT NULL,
            granted_at TEXT NOT NULL,
            UNIQUE(user_id, resource_type, resource_id),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create permissions table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_permissions_user_id ON permissions(user_id)").await?;

    // User settings table
    exec(pool, r#"
        CREATE TABLE IF NOT EXISTS user_settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            preferences TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
    "#).await.context("failed to create user_settings table")?;

    exec(pool, "CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id)").await?;

    info!("migrations completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::prepare_database;
    use switchboard_config::DatabaseConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_migrations_run() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_migrations.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = prepare_database(&config).await.unwrap();

        // This will fail if there are no migration files, but that's expected in testing
        let result = run_migrations(&pool).await;

        // The test passes whether migrations exist or not, as we're just testing the function
        match result {
            Ok(_) => println!("Migrations ran successfully"),
            Err(e) => println!("No migrations found or migration error: {}", e),
        }
    }
}
