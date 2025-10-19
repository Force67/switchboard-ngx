//! Repository for attachment data access operations.

use crate::entities::{MessageAttachment, CreateAttachmentRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::{SqlitePool, Row};
use tracing::{info, warn};

/// Repository for attachment database operations
pub struct AttachmentRepository {
    pool: SqlitePool,
}

impl AttachmentRepository {
    /// Create a new attachment repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all attachments for a message
    pub async fn find_by_message_id(&self, message_id: i64) -> ChatResult<Vec<MessageAttachment>> {
        let rows = sqlx::query(
            "SELECT id, public_id, message_id, filename, content_type, file_size, file_path, created_at
             FROM message_attachments WHERE message_id = ? ORDER BY created_at ASC"
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let attachments = rows.into_iter().map(|row| {
            Ok(MessageAttachment {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_id: row.try_get("message_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                filename: row.try_get("filename").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content_type: row.try_get("content_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                file_size: row.try_get("file_size").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                file_path: row.try_get("file_path").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(attachments)
    }

    /// Find an attachment by its public ID
    pub async fn find_by_public_id(&self, public_id: &str) -> ChatResult<Option<MessageAttachment>> {
        let row = sqlx::query(
            "SELECT id, public_id, message_id, filename, content_type, file_size, file_path, created_at
             FROM message_attachments WHERE public_id = ?"
        )
        .bind(public_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(MessageAttachment {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                message_id: row.try_get("message_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                filename: row.try_get("filename").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                content_type: row.try_get("content_type").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                file_size: row.try_get("file_size").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                file_path: row.try_get("file_path").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new attachment
    pub async fn create(&self, request: &CreateAttachmentRequest) -> ChatResult<MessageAttachment> {
        let public_id = cuid2::cuid();
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO message_attachments (public_id, message_id, filename, content_type, file_size, file_path, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&public_id)
        .bind(request.message_id)
        .bind(&request.filename)
        .bind(&request.content_type)
        .bind(request.file_size)
        .bind(&request.file_path)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let attachment_id = result.last_insert_rowid();

        info!(
            attachment_id = attachment_id,
            message_id = request.message_id,
            filename = %request.filename,
            "created new attachment"
        );

        Ok(MessageAttachment {
            id: attachment_id,
            public_id,
            message_id: request.message_id,
            filename: request.filename.clone(),
            content_type: request.content_type.clone(),
            file_size: request.file_size,
            file_path: request.file_path.clone(),
            created_at: now,
        })
    }

    /// Delete an attachment by public ID
    pub async fn delete(&self, public_id: &str) -> ChatResult<()> {
        // First check if attachment exists
        let attachment = self.find_by_public_id(public_id).await?;
        if attachment.is_none() {
            return Err(ChatError::AttachmentNotFound);
        }

        let result = sqlx::query("DELETE FROM message_attachments WHERE public_id = ?")
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ChatError::AttachmentNotFound);
        }

        info!(
            public_id = public_id,
            "deleted attachment"
        );

        Ok(())
    }

    /// Delete all attachments for a message (typically when message is deleted)
    pub async fn delete_by_message_id(&self, message_id: i64) -> ChatResult<usize> {
        let result = sqlx::query("DELETE FROM message_attachments WHERE message_id = ?")
            .bind(message_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let deleted_count = result.rows_affected();

        if deleted_count > 0 {
            info!(
                message_id = message_id,
                deleted_count = deleted_count,
                "deleted attachments for message"
            );
        }

        Ok(deleted_count as usize)
    }

    /// Get total file size for all attachments in a message
    pub async fn get_total_size_for_message(&self, message_id: i64) -> ChatResult<i64> {
        let row = sqlx::query("SELECT COALESCE(SUM(file_size), 0) as total_size FROM message_attachments WHERE message_id = ?")
            .bind(message_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let total_size = row
            .map(|r| r.try_get::<i64, _>("total_size").unwrap_or(0))
            .unwrap_or(0);

        Ok(total_size)
    }

    /// Count attachments for a message
    pub async fn count_attachments_for_message(&self, message_id: i64) -> ChatResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM message_attachments WHERE message_id = ?")
            .bind(message_id)
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
        let db_path = temp_dir.path().join("test_attachments.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE message_attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                message_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                content_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                created_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_attachment() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = AttachmentRepository::new(pool);

        let request = CreateAttachmentRequest {
            message_id: 1,
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 1024,
            file_path: "/uploads/test.txt".to_string(),
        };

        let attachment = repo.create(&request).await.unwrap();
        assert!(attachment.id > 0);
        assert_eq!(attachment.message_id, 1);
        assert_eq!(attachment.filename, "test.txt");
        assert_eq!(attachment.content_type, "text/plain");
        assert_eq!(attachment.file_size, 1024);
    }

    #[tokio::test]
    async fn test_find_by_message_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = AttachmentRepository::new(pool);

        let request = CreateAttachmentRequest {
            message_id: 1,
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 1024,
            file_path: "/uploads/test.txt".to_string(),
        };

        repo.create(&request).await.unwrap();

        let attachments = repo.find_by_message_id(1).await.unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "test.txt");
    }

    #[tokio::test]
    async fn test_find_by_public_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = AttachmentRepository::new(pool);

        let request = CreateAttachmentRequest {
            message_id: 1,
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 1024,
            file_path: "/uploads/test.txt".to_string(),
        };

        let created = repo.create(&request).await.unwrap();
        let found = repo.find_by_public_id(&created.public_id).await.unwrap();

        assert!(found.is_some());
        let found_attachment = found.unwrap();
        assert_eq!(found_attachment.id, created.id);
        assert_eq!(found_attachment.public_id, created.public_id);
    }

    #[tokio::test]
    async fn test_delete_attachment() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = AttachmentRepository::new(pool);

        let request = CreateAttachmentRequest {
            message_id: 1,
            filename: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 1024,
            file_path: "/uploads/test.txt".to_string(),
        };

        let created = repo.create(&request).await.unwrap();
        repo.delete(&created.public_id).await.unwrap();

        let found = repo.find_by_public_id(&created.public_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_count_and_size() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = AttachmentRepository::new(pool);

        let request1 = CreateAttachmentRequest {
            message_id: 1,
            filename: "test1.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 1024,
            file_path: "/uploads/test1.txt".to_string(),
        };

        let request2 = CreateAttachmentRequest {
            message_id: 1,
            filename: "test2.txt".to_string(),
            content_type: "text/plain".to_string(),
            file_size: 2048,
            file_path: "/uploads/test2.txt".to_string(),
        };

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let count = repo.count_attachments_for_message(1).await.unwrap();
        assert_eq!(count, 2);

        let total_size = repo.get_total_size_for_message(1).await.unwrap();
        assert_eq!(total_size, 3072); // 1024 + 2048
    }
}