//! Test utilities for service layer testing

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, sqlite::SqliteJournalMode};
use tempfile::TempDir;
use std::str::FromStr;
use uuid::Uuid;

/// Creates a test database with the necessary schema
pub async fn create_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Memory)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(connect_options)
        .await
        .expect("Failed to create test database");

    // Run migrations - create the schema
    create_schema(&pool).await.expect("Failed to create schema");

    (pool, temp_dir)
}

/// Creates the database schema for testing
async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            email TEXT,
            display_name TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    // Sessions table
    sqlx::query(
        r#"
        CREATE TABLE sessions (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Chats table
    sqlx::query(
        r#"
        CREATE TABLE chats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            user_id INTEGER,
            folder_id INTEGER,
            title TEXT NOT NULL,
            chat_type TEXT NOT NULL DEFAULT 'direct',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (folder_id) REFERENCES folders (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Folders table
    sqlx::query(
        r#"
        CREATE TABLE folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Chat members table
    sqlx::query(
        r#"
        CREATE TABLE chat_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL,
            FOREIGN KEY (chat_id) REFERENCES chats (id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
            UNIQUE(chat_id, user_id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Messages table
    sqlx::query(
        r#"
        CREATE TABLE messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'text',
            role TEXT NOT NULL,
            model TEXT,
            thread_id INTEGER,
            reply_to_id INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (chat_id) REFERENCES chats (id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (thread_id) REFERENCES messages (id),
            FOREIGN KEY (reply_to_id) REFERENCES messages (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Message edits table
    sqlx::query(
        r#"
        CREATE TABLE message_edits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id INTEGER NOT NULL,
            edited_by_user_id INTEGER NOT NULL,
            old_content TEXT NOT NULL,
            new_content TEXT NOT NULL,
            edited_at TEXT NOT NULL,
            FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE,
            FOREIGN KEY (edited_by_user_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Message attachments table
    sqlx::query(
        r#"
        CREATE TABLE message_attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id INTEGER NOT NULL,
            file_name TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_url TEXT NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
        )
        "#
    )
    .execute(pool)
    .await?;

    // Message deletions table
    sqlx::query(
        r#"
        CREATE TABLE message_deletions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id INTEGER NOT NULL,
            deleted_by_user_id INTEGER NOT NULL,
            reason TEXT,
            deleted_at TEXT NOT NULL,
            FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE,
            FOREIGN KEY (deleted_by_user_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Folders table
    sqlx::query(
        r#"
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
        "#
    )
    .execute(pool)
    .await?;

    // Notifications table
    sqlx::query(
        r#"
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
        "#
    )
    .execute(pool)
    .await?;

    // Permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id INTEGER NOT NULL,
            permission_level TEXT NOT NULL CHECK (permission_level IN ('read', 'write', 'admin')),
            granted_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(user_id, resource_type, resource_id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Chat invites table
    sqlx::query(
        r#"
        CREATE TABLE chat_invites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id INTEGER NOT NULL,
            inviter_id INTEGER NOT NULL,
            invitee_email TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (chat_id) REFERENCES chats (id) ON DELETE CASCADE,
            FOREIGN KEY (inviter_id) REFERENCES users (id)
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a test user in the database
pub async fn create_test_user(
    pool: &SqlitePool,
    id: i64,
    public_id: &str,
    email: Option<&str>,
    display_name: Option<&str>,
) -> Result<switchboard_auth::User, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, public_id, email, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(id)
    .bind(public_id)
    .bind(email)
    .bind(display_name)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(switchboard_auth::User {
        id,
        public_id: public_id.to_string(),
        email: email.map(|e| e.to_string()),
        display_name: display_name.map(|d| d.to_string()),
    })
}

/// Creates a test chat in the database
pub async fn create_test_chat(
    pool: &SqlitePool,
    user_id: i64,
    title: &str,
    chat_type: &str,
) -> Result<i64, sqlx::Error> {
    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let chat_id = sqlx::query(
        r#"
        INSERT INTO chats (public_id, user_id, title, chat_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(user_id)
    .bind(title)
    .bind(chat_type)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    // Add user as chat member
    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, 'owner', ?)
        "#
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(chat_id)
}

/// Adds a user as a member of a chat
pub async fn add_chat_member(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
    role: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, ?, ?)
        "#
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(role)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a test message in the database
pub async fn create_test_message(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
    content: &str,
    role: &str,
) -> Result<i64, sqlx::Error> {
    let public_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let message_id = sqlx::query(
        r#"
        INSERT INTO messages (public_id, chat_id, user_id, content, role, message_type, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'text', ?, ?)
        "#
    )
    .bind(&public_id)
    .bind(chat_id)
    .bind(user_id)
    .bind(content)
    .bind(role)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(message_id)
}

/// Test fixtures and common data
pub mod fixtures {
    use super::*;

    pub const TEST_USER_ID: i64 = 1;
    pub const TEST_USER_ID_2: i64 = 2;
    pub const TEST_USER_PUBLIC_ID: &str = "test-user-123";
    pub const TEST_USER_EMAIL: &str = "test@example.com";
    pub const TEST_USER_DISPLAY_NAME: &str = "Test User";

    pub fn test_user() -> switchboard_auth::User {
        switchboard_auth::User {
            id: TEST_USER_ID,
            public_id: TEST_USER_PUBLIC_ID.to_string(),
            email: Some(TEST_USER_EMAIL.to_string()),
            display_name: Some(TEST_USER_DISPLAY_NAME.to_string()),
        }
    }

    pub fn test_user_2() -> switchboard_auth::User {
        switchboard_auth::User {
            id: TEST_USER_ID_2,
            public_id: "test-user-456".to_string(),
            email: Some("test2@example.com".to_string()),
            display_name: Some("Test User 2".to_string()),
        }
    }

    pub const TEST_CHAT_TITLE: &str = "Test Chat";
    pub const TEST_CHAT_TYPE: &str = "direct";
    pub const TEST_MESSAGE_CONTENT: &str = "Hello, world!";
    pub const TEST_MESSAGE_ROLE: &str = "user";
    pub const TEST_FOLDER_NAME: &str = "Test Folder";
}