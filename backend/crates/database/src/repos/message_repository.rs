//! Repository for message data access operations.

use crate::entities::{ChatMessage, MessageStatus, CreateMessageRequest, UpdateMessageRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::{SqlitePool, Row};
use tracing::{info, warn};

/// Repository for message database operations
pub struct MessageRepository {
    pool: SqlitePool,
}

impl MessageRepository {
    /// Create a new message repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find messages by chat ID with pagination
    pub async fn find_by_chat_id(
        &self,
        chat_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ChatResult<Vec<ChatMessage>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, public_id, chat_id, sender_id, content, message_type, status, created_at, updated_at
             FROM messages WHERE chat_id = ? AND status != 'deleted' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(chat_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let messages = rows.into_iter().map(|row| {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatMessage {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                sender_id: row.try_get("sender_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content: row.try_get("content").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_type: row.try_get("message_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: MessageStatus::from(status_str.as_str()),
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    /// Find a message by its public ID
    pub async fn find_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatMessage>> {
        let row = sqlx::query(
            "SELECT id, public_id, chat_id, sender_id, content, message_type, status, created_at, updated_at
             FROM messages WHERE public_id = ?"
        )
        .bind(public_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(ChatMessage {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                sender_id: row.try_get("sender_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content: row.try_get("content").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_type: row.try_get("message_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: MessageStatus::from(status_str.as_str()),
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new message
    pub async fn create(&self, sender_id: i64, request: &CreateMessageRequest) -> ChatResult<ChatMessage> {
        let public_id = cuid2::cuid();
        let now = chrono::Utc::now().to_rfc3339();
        let message_type = request.message_type.as_deref().unwrap_or("text");

        let result = sqlx::query(
            "INSERT INTO messages (public_id, chat_id, sender_id, content, message_type, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&public_id)
        .bind(request.chat_id)
        .bind(sender_id)
        .bind(&request.content)
        .bind(message_type)
        .bind(MessageStatus::Sent.to_string())
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let message_id = result.last_insert_rowid();

        info!(
            message_id = message_id,
            public_id = %public_id,
            chat_id = request.chat_id,
            sender_id = sender_id,
            "created new message"
        );

        Ok(ChatMessage {
            id: message_id,
            public_id,
            chat_id: request.chat_id,
            sender_id,
            content: request.content.clone(),
            message_type: message_type.to_string(),
            status: MessageStatus::Sent,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Update a message
    pub async fn update(&self, public_id: &str, user_id: i64, request: &UpdateMessageRequest) -> ChatResult<ChatMessage> {
        // First check if message exists and user has permission
        let message = self.find_by_public_id(public_id).await?;
        if message.is_none() {
            return Err(ChatError::MessageNotFound);
        }

        let message = message.unwrap();

        // Check if user is the sender
        if message.sender_id != user_id {
            return Err(ChatError::Unauthorized);
        }

        let mut update_fields = Vec::new();
        let mut values = Vec::new();

        if let Some(content) = &request.content {
            update_fields.push("content = ?");
            values.push(content.clone());
        }

        if let Some(status) = &request.status {
            update_fields.push("status = ?");
            values.push(status.to_string());
        }

        if update_fields.is_empty() {
            return self.find_by_public_id(public_id).await.map(|m| m.unwrap());
        }

        let now = chrono::Utc::now().to_rfc3339();
        update_fields.push("updated_at = ?");
        values.push(now);

        let query = format!(
            "UPDATE messages SET {} WHERE public_id = ?",
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

        self.find_by_public_id(public_id).await.map(|m| m.unwrap())
    }

    /// Delete a message (soft delete by setting status to deleted)
    pub async fn delete(&self, public_id: &str, user_id: i64) -> ChatResult<()> {
        let message = self.find_by_public_id(public_id).await?;
        if message.is_none() {
            return Err(ChatError::MessageNotFound);
        }

        let message = message.unwrap();

        // Check if user is the sender
        if message.sender_id != user_id {
            return Err(ChatError::Unauthorized);
        }

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE messages SET status = 'deleted', updated_at = ? WHERE public_id = ?")
            .bind(&now)
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            public_id = public_id,
            deleted_by = user_id,
            "soft deleted message"
        );

        Ok(())
    }

    /// Mark message as delivered
    pub async fn mark_delivered(&self, public_id: &str) -> ChatResult<ChatMessage> {
        let request = UpdateMessageRequest {
            content: None,
            status: Some(MessageStatus::Delivered),
        };

        self.update(public_id, 0, &request).await // Use 0 as user_id since this is a system update
    }

    /// Mark message as read
    pub async fn mark_read(&self, public_id: &str) -> ChatResult<ChatMessage> {
        let request = UpdateMessageRequest {
            content: None,
            status: Some(MessageStatus::Read),
        };

        self.update(public_id, 0, &request).await // Use 0 as user_id since this is a system update
    }

    /// Count messages for a chat
    pub async fn count_messages_for_chat(&self, chat_id: i64) -> ChatResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM messages WHERE chat_id = ? AND status != 'deleted'")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let count = row
            .map(|r| r.try_get::<i64, _>("count").unwrap_or(0))
            .unwrap_or(0);

        Ok(count)
    }

    /// Get last message for a chat
    pub async fn get_last_message(&self, chat_id: i64) -> ChatResult<Option<ChatMessage>> {
        let row = sqlx::query(
            "SELECT id, public_id, chat_id, sender_id, content, message_type, status, created_at, updated_at
             FROM messages WHERE chat_id = ? AND status != 'deleted' ORDER BY created_at DESC LIMIT 1"
        )
        .bind(chat_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(ChatMessage {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                sender_id: row.try_get("sender_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content: row.try_get("content").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_type: row.try_get("message_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: MessageStatus::from(status_str.as_str()),
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Search messages in a chat
    pub async fn search_messages(&self, chat_id: i64, query: &str, limit: Option<i64>) -> ChatResult<Vec<ChatMessage>> {
        let limit = limit.unwrap_or(20);
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            "SELECT id, public_id, chat_id, sender_id, content, message_type, status, created_at, updated_at
             FROM messages WHERE chat_id = ? AND status != 'deleted' AND content LIKE ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(chat_id)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let messages = rows.into_iter().map(|row| {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatMessage {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                sender_id: row.try_get("sender_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content: row.try_get("content").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_type: row.try_get("message_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: MessageStatus::from(status_str.as_str()),
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_messages.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                chat_id INTEGER NOT NULL,
                sender_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                message_type TEXT NOT NULL,
                status TEXT NOT NULL,
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
    async fn test_create_message() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let request = CreateMessageRequest {
            chat_id: 1,
            content: "Hello, world!".to_string(),
            message_type: None,
        };

        let message = repo.create(1, &request).await.unwrap();
        assert!(message.id > 0);
        assert_eq!(message.chat_id, 1);
        assert_eq!(message.sender_id, 1);
        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.status, MessageStatus::Sent);
    }

    #[tokio::test]
    async fn test_find_by_chat_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let request = CreateMessageRequest {
            chat_id: 1,
            content: "Test message".to_string(),
            message_type: None,
        };

        repo.create(1, &request).await.unwrap();

        let messages = repo.find_by_chat_id(1, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Test message");
    }

    #[tokio::test]
    async fn test_find_by_public_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let request = CreateMessageRequest {
            chat_id: 1,
            content: "Test message".to_string(),
            message_type: None,
        };

        let created = repo.create(1, &request).await.unwrap();
        let found = repo.find_by_public_id(&created.public_id).await.unwrap();

        assert!(found.is_some());
        let found_message = found.unwrap();
        assert_eq!(found_message.id, created.id);
        assert_eq!(found_message.public_id, created.public_id);
    }

    #[tokio::test]
    async fn test_update_message() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let create_request = CreateMessageRequest {
            chat_id: 1,
            content: "Original content".to_string(),
            message_type: None,
        };

        let created = repo.create(1, &create_request).await.unwrap();

        let update_request = UpdateMessageRequest {
            content: Some("Updated content".to_string()),
            status: Some(MessageStatus::Delivered),
        };

        let updated = repo.update(&created.public_id, 1, &update_request).await.unwrap();
        assert_eq!(updated.content, "Updated content");
        assert_eq!(updated.status, MessageStatus::Delivered);
    }

    #[tokio::test]
    async fn test_delete_message() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let request = CreateMessageRequest {
            chat_id: 1,
            content: "Test message".to_string(),
            message_type: None,
        };

        let created = repo.create(1, &request).await.unwrap();
        repo.delete(&created.public_id, 1).await.unwrap();

        let found = repo.find_by_public_id(&created.public_id).await.unwrap();
        assert!(found.is_some()); // Should still exist but with deleted status
        assert_eq!(found.unwrap().status, MessageStatus::Deleted);
    }

    #[tokio::test]
    async fn test_search_messages() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MessageRepository::new(pool);

        let request1 = CreateMessageRequest {
            chat_id: 1,
            content: "Hello world".to_string(),
            message_type: None,
        };

        let request2 = CreateMessageRequest {
            chat_id: 1,
            content: "Another message".to_string(),
            message_type: None,
        };

        repo.create(1, &request1).await.unwrap();
        repo.create(1, &request2).await.unwrap();

        let results = repo.search_messages(1, "hello", None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello world");
    }
}