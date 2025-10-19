//! Repository for chat data access operations.

use crate::entities::{Chat, ChatType, ChatStatus, CreateChatRequest, UpdateChatRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::{SqlitePool, Row};
use tracing::{info, warn};

/// Repository for chat database operations
pub struct ChatRepository {
    pool: SqlitePool,
}

impl ChatRepository {
    /// Create a new chat repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find chat by public ID
    pub async fn find_by_public_id(&self, public_id: &str) -> ChatResult<Option<Chat>> {
        let row = sqlx::query(
            "SELECT c.id, c.public_id, c.title, c.description, c.avatar_url, c.folder_id, c.chat_type, c.status,
                    c.created_by, c.created_at, c.updated_at,
                    (SELECT COUNT(*) FROM chat_members WHERE chat_id = c.id) as member_count,
                    (SELECT COUNT(*) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as message_count,
                    (SELECT MAX(created_at) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as last_message_at
             FROM chats c WHERE c.public_id = ? AND c.status != 'deleted'"
        )
        .bind(public_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let chat_type_str: String = row.try_get("chat_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?;
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(Chat {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                title: row.try_get("title").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                avatar_url: row.try_get("avatar_url").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                folder_id: row.try_get("folder_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_type: ChatType::from(chat_type_str.as_str()),
                status: ChatStatus::from(status_str.as_str()),
                created_by: row.try_get("created_by").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                member_count: row.try_get("member_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_count: row.try_get("message_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                last_message_at: row.try_get("last_message_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find chats by user ID (as member or creator)
    pub async fn find_by_user_id(&self, user_id: i64) -> ChatResult<Vec<Chat>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT c.id, c.public_id, c.title, c.description, c.avatar_url, c.folder_id, c.chat_type, c.status,
                    c.created_by, c.created_at, c.updated_at,
                    (SELECT COUNT(*) FROM chat_members WHERE chat_id = c.id) as member_count,
                    (SELECT COUNT(*) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as message_count,
                    (SELECT MAX(created_at) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as last_message_at
            FROM chats c
            LEFT JOIN chat_members cm ON c.id = cm.chat_id
            WHERE (c.created_by = ? OR cm.user_id = ?) AND c.status != 'deleted'
            ORDER BY c.updated_at DESC
            "#
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let chats = rows.into_iter().map(|row| {
            let chat_type_str: String = row.try_get("chat_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?;
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Chat {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                title: row.try_get("title").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                avatar_url: row.try_get("avatar_url").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                folder_id: row.try_get("folder_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_type: ChatType::from(chat_type_str.as_str()),
                status: ChatStatus::from(status_str.as_str()),
                created_by: row.try_get("created_by").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                member_count: row.try_get("member_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_count: row.try_get("message_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                last_message_at: row.try_get("last_message_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(chats)
    }

    /// Find direct chat between two users
    pub async fn find_direct_chat(&self, user1_id: i64, user2_id: i64) -> ChatResult<Option<Chat>> {
        let row = sqlx::query(
            r#"
            SELECT c.id, c.public_id, c.title, c.description, c.avatar_url, c.folder_id, c.chat_type, c.status,
                    c.created_by, c.created_at, c.updated_at,
                    (SELECT COUNT(*) FROM chat_members WHERE chat_id = c.id) as member_count,
                    (SELECT COUNT(*) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as message_count,
                    (SELECT MAX(created_at) FROM messages WHERE chat_id = c.id AND deleted_at IS NULL) as last_message_at
            FROM chats c
            JOIN chat_members cm1 ON c.id = cm1.chat_id
            JOIN chat_members cm2 ON c.id = cm2.chat_id
            WHERE c.chat_type = 'direct'
            AND c.status = 'active'
            AND ((cm1.user_id = ? AND cm2.user_id = ?) OR (cm1.user_id = ? AND cm2.user_id = ?))
            AND cm1.role = 'member' AND cm2.role = 'member'
            "#
        )
        .bind(user1_id)
        .bind(user2_id)
        .bind(user2_id)
        .bind(user1_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let chat_type_str: String = row.try_get("chat_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?;
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(Chat {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                title: row.try_get("title").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                description: row.try_get("description").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                avatar_url: row.try_get("avatar_url").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                folder_id: row.try_get("folder_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_type: ChatType::from(chat_type_str.as_str()),
                status: ChatStatus::from(status_str.as_str()),
                created_by: row.try_get("created_by").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                member_count: row.try_get("member_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_count: row.try_get("message_count").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                last_message_at: row.try_get("last_message_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new chat
    pub async fn create(&self, user_id: i64, request: &CreateChatRequest) -> ChatResult<Chat> {
        let public_id = cuid2::cuid();
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO chats (public_id, title, description, avatar_url, folder_id, chat_type, status, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&public_id)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.avatar_url)
        .bind(&request.folder_id)
        .bind(request.chat_type.to_string())
        .bind(ChatStatus::Active.to_string())
        .bind(user_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let chat_id = result.last_insert_rowid();

        info!(
            chat_id = chat_id,
            public_id = %public_id,
            created_by = %request.created_by,
            chat_type = %request.chat_type.to_string(),
            "created new chat"
        );

        Ok(Chat {
            id: chat_id,
            public_id,
            title: request.title.clone(),
            description: request.description.clone(),
            avatar_url: request.avatar_url.clone(),
            folder_id: request.folder_id.clone(),
            chat_type: request.chat_type.clone(),
            status: ChatStatus::Active,
            created_by: request.created_by.clone(),
            created_at: now.clone(),
            updated_at: now,
            member_count: 0,
            message_count: 0,
            last_message_at: None,
        })
    }

    /// Update a chat
    pub async fn update(&self, public_id: &str, user_id: i64, request: &UpdateChatRequest) -> ChatResult<Chat> {
        // First check if chat exists and user has permission
        let chat = self.find_by_public_id(public_id).await?;
        if chat.is_none() {
            return Err(ChatError::ChatNotFound);
        }

        let chat = chat.unwrap();

        // Check if user is the creator or has admin permissions
        if chat.created_by != user_id.to_string() {
            // TODO: Check if user has admin role in chat_members
            return Err(ChatError::Unauthorized);
        }

        let mut update_fields = Vec::new();
        let mut values = Vec::new();

        if let Some(title) = &request.title {
            update_fields.push("title = ?");
            values.push(title.clone());
        }

        if let Some(description) = &request.description {
            update_fields.push("description = ?");
            values.push(description.clone());
        }

        if let Some(avatar_url) = &request.avatar_url {
            update_fields.push("avatar_url = ?");
            values.push(avatar_url.clone());
        }

        if let Some(folder_id) = &request.folder_id {
            update_fields.push("folder_id = ?");
            values.push(folder_id.clone());
        }

        if let Some(status) = &request.status {
            update_fields.push("status = ?");
            values.push(status.to_string());
        }

        if update_fields.is_empty() {
            return self.find_by_public_id(public_id).await.map(|c| c.unwrap());
        }

        let now = chrono::Utc::now().to_rfc3339();
        update_fields.push("updated_at = ?");
        values.push(now);

        let query = format!(
            "UPDATE chats SET {} WHERE public_id = ?",
            update_fields.join(", ")
        );

        values.push(public_id.to_string());

        let mut query_builder = sqlx::query(&query);
        for value in &values {
            query_builder = query_builder.bind(value);
        }

        query_builder
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        self.find_by_public_id(public_id).await.map(|c| c.unwrap())
    }

    /// Delete a chat (soft delete by setting status to deleted)
    pub async fn delete(&self, public_id: &str, user_id: i64) -> ChatResult<()> {
        let chat = self.find_by_public_id(public_id).await?;
        if chat.is_none() {
            return Err(ChatError::ChatNotFound);
        }

        let chat = chat.unwrap();

        // Check if user is the creator or has admin permissions
        if chat.created_by != user_id.to_string() {
            // TODO: Check if user has admin role in chat_members
            return Err(ChatError::Unauthorized);
        }

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE chats SET status = 'deleted', updated_at = ? WHERE public_id = ?")
            .bind(&now)
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            public_id = public_id,
            deleted_by = user_id,
            "soft deleted chat"
        );

        Ok(())
    }

    /// Archive a chat
    pub async fn archive(&self, public_id: &str, user_id: i64) -> ChatResult<Chat> {
        let request = UpdateChatRequest {
            title: None,
            description: None,
            avatar_url: None,
            folder_id: None,
            status: Some(ChatStatus::Archived),
        };

        self.update(public_id, user_id, &request).await
    }

    /// Unarchive a chat
    pub async fn unarchive(&self, public_id: &str, user_id: i64) -> ChatResult<Chat> {
        let request = UpdateChatRequest {
            title: None,
            description: None,
            avatar_url: None,
            folder_id: None,
            status: Some(ChatStatus::Active),
        };

        self.update(public_id, user_id, &request).await
    }

    /// Count chats for a user
    pub async fn count_chats_for_user(&self, user_id: i64) -> ChatResult<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT c.id) as count
            FROM chats c
            LEFT JOIN chat_members cm ON c.id = cm.chat_id
            WHERE (c.created_by = ? OR cm.user_id = ?) AND c.status != 'deleted'
            "#
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let count = row
            .map(|r| r.try_get::<i64, _>("count").unwrap_or(0))
            .unwrap_or(0);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_chats.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE chats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                title TEXT,
                description TEXT,
                avatar_url TEXT,
                folder_id TEXT,
                chat_type TEXT NOT NULL,
                status TEXT NOT NULL,
                created_by INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_chat() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let request = CreateChatRequest {
            title: "Test Chat".to_string(),
            description: Some("A test chat".to_string()),
            avatar_url: None,
            folder_id: None,
            chat_type: ChatType::Group,
            created_by: "1".to_string(),
        };

        let chat = repo.create(1, &request).await.unwrap();
        assert!(chat.id > 0);
        assert_eq!(chat.title, "Test Chat");
        assert_eq!(chat.chat_type, ChatType::Group);
        assert_eq!(chat.status, ChatStatus::Active);
        assert_eq!(chat.created_by, "1");
    }

    #[tokio::test]
    async fn test_find_by_public_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let request = CreateChatRequest {
            title: "Test Chat".to_string(),
            description: None,
            avatar_url: None,
            folder_id: None,
            chat_type: ChatType::Direct,
            created_by: "1".to_string(),
        };

        let created = repo.create(1, &request).await.unwrap();
        let found = repo.find_by_public_id(&created.public_id).await.unwrap();

        assert!(found.is_some());
        let found_chat = found.unwrap();
        assert_eq!(found_chat.id, created.id);
        assert_eq!(found_chat.public_id, created.public_id);
    }

    #[tokio::test]
    async fn test_update_chat() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let create_request = CreateChatRequest {
            title: "Original Title".to_string(),
            description: None,
            avatar_url: None,
            folder_id: None,
            chat_type: ChatType::Group,
            created_by: "1".to_string(),
        };

        let created = repo.create(1, &create_request).await.unwrap();

        let update_request = UpdateChatRequest {
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            avatar_url: None,
            folder_id: None,
            status: None,
        };

        let updated = repo.update(&created.public_id, 1, &update_request).await.unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description.as_ref().unwrap(), "Updated description");
    }

    #[tokio::test]
    async fn test_delete_chat() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let request = CreateChatRequest {
            title: "Test Chat".to_string(),
            description: None,
            avatar_url: None,
            folder_id: None,
            chat_type: ChatType::Group,
            created_by: "1".to_string(),
        };

        let created = repo.create(1, &request).await.unwrap();
        repo.delete(&created.public_id, 1).await.unwrap();

        let found = repo.find_by_public_id(&created.public_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_archive_unarchive() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let request = CreateChatRequest {
            title: "Test Chat".to_string(),
            description: None,
            avatar_url: None,
            folder_id: None,
            chat_type: ChatType::Group,
            created_by: "1".to_string(),
        };

        let created = repo.create(1, &request).await.unwrap();

        let archived = repo.archive(&created.public_id, 1).await.unwrap();
        assert_eq!(archived.status, ChatStatus::Archived);

        let unarchived = repo.unarchive(&created.public_id, 1).await.unwrap();
        assert_eq!(unarchived.status, ChatStatus::Active);
    }
}