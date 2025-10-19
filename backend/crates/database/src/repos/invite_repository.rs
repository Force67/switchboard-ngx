//! Repository for invite data access operations.

use crate::entities::{ChatInvite, InviteStatus, CreateInviteRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::{SqlitePool, Row};
use tracing::{info, warn};
use std::collections::HashMap;

/// Repository for invite database operations
pub struct InviteRepository {
    pool: SqlitePool,
}

impl InviteRepository {
    /// Create a new invite repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find invite by public ID
    pub async fn find_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatInvite>> {
        let row = sqlx::query(
            "SELECT i.id, i.public_id, i.chat_id, i.inviter_id, i.invited_email, i.invite_code, i.status, i.expires_at, i.created_at, i.accepted_at,
                    c.public_id as chat_public_id, c.title as chat_title,
                    u.public_id as invited_by_public_id, u.display_name as inviter_display_name, u.avatar_url as inviter_avatar_url
             FROM chat_invites i
             LEFT JOIN chats c ON i.chat_id = c.id
             LEFT JOIN users u ON i.inviter_id = u.id
             WHERE i.public_id = ?"
        )
        .bind(public_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(ChatInvite {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_public_id: row.try_get("chat_public_id").unwrap_or("unknown".to_string()),
                chat_title: row.try_get("chat_title").unwrap_or("Unknown Chat".to_string()),
                inviter_id: row.try_get("inviter_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invited_by_public_id: row.try_get("invited_by_public_id").unwrap_or("unknown".to_string()),
                inviter_display_name: row.try_get("inviter_display_name").ok(),
                inviter_avatar_url: row.try_get("inviter_avatar_url").ok(),
                invited_email: row.try_get("invited_email").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invite_code: row.try_get("invite_code").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: InviteStatus::from(status_str.as_str()),
                expires_at: row.try_get("expires_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                accepted_at: row.try_get("accepted_at").ok(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Find invite by invite code
    pub async fn find_by_invite_code(&self, invite_code: &str) -> ChatResult<Option<ChatInvite>> {
        let row = sqlx::query(
            "SELECT i.id, i.public_id, i.chat_id, i.inviter_id, i.invited_email, i.invite_code,
                    i.status, i.expires_at, i.created_at, i.accepted_at,
                    c.public_id as chat_public_id, c.title as chat_title,
                    u.public_id as invited_by_public_id, u.display_name as inviter_display_name, u.avatar_url as inviter_avatar_url
             FROM chat_invites i
             LEFT JOIN chats c ON i.chat_id = c.id
             LEFT JOIN users u ON i.inviter_id = u.id
             WHERE i.invite_code = ?"
        )
        .bind(invite_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(ChatInvite {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_public_id: row.try_get("chat_public_id").unwrap_or("unknown".to_string()),
                chat_title: row.try_get("chat_title").unwrap_or("Unknown Chat".to_string()),
                inviter_id: row.try_get("inviter_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invited_by_public_id: row.try_get("invited_by_public_id").unwrap_or("unknown".to_string()),
                inviter_display_name: row.try_get("inviter_display_name").ok(),
                inviter_avatar_url: row.try_get("inviter_avatar_url").ok(),
                invited_email: row.try_get("invited_email").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invite_code: row.try_get("invite_code").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: InviteStatus::from(status_str.as_str()),
                expires_at: row.try_get("expires_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                accepted_at: row.try_get("accepted_at").ok(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Find all invites for a chat
    pub async fn find_by_chat_id(&self, chat_id: i64) -> ChatResult<Vec<ChatInvite>> {
        let rows = sqlx::query(
            "SELECT i.id, i.public_id, i.chat_id, i.inviter_id, i.invited_email, i.invite_code,
                    i.status, i.expires_at, i.created_at, i.accepted_at,
                    c.public_id as chat_public_id, c.title as chat_title,
                    u.public_id as invited_by_public_id, u.display_name as inviter_display_name, u.avatar_url as inviter_avatar_url
             FROM chat_invites i
             LEFT JOIN chats c ON i.chat_id = c.id
             LEFT JOIN users u ON i.inviter_id = u.id
             WHERE i.chat_id = ? ORDER BY i.created_at DESC"
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let invites = rows.into_iter().map(|row| {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatInvite {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_public_id: row.try_get("chat_public_id").unwrap_or("unknown".to_string()),
                chat_title: row.try_get("chat_title").unwrap_or("Unknown Chat".to_string()),
                inviter_id: row.try_get("inviter_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invited_by_public_id: row.try_get("invited_by_public_id").unwrap_or("unknown".to_string()),
                inviter_display_name: row.try_get("inviter_display_name").ok(),
                inviter_avatar_url: row.try_get("inviter_avatar_url").ok(),
                invited_email: row.try_get("invited_email").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invite_code: row.try_get("invite_code").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: InviteStatus::from(status_str.as_str()),
                expires_at: row.try_get("expires_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                accepted_at: row.try_get("accepted_at").ok(),
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(invites)
    }

    /// Find invites sent by a user
    pub async fn find_by_inviter_id(&self, inviter_id: i64) -> ChatResult<Vec<ChatInvite>> {
        let rows = sqlx::query(
            "SELECT i.id, i.public_id, i.chat_id, i.inviter_id, i.invited_email, i.invite_code,
                    i.status, i.expires_at, i.created_at, i.accepted_at,
                    c.public_id as chat_public_id, c.title as chat_title,
                    u.public_id as invited_by_public_id, u.display_name as inviter_display_name, u.avatar_url as inviter_avatar_url
             FROM chat_invites i
             LEFT JOIN chats c ON i.chat_id = c.id
             LEFT JOIN users u ON i.inviter_id = u.id
             WHERE i.inviter_id = ? ORDER BY i.created_at DESC"
        )
        .bind(inviter_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let invites = rows.into_iter().map(|row| {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatInvite {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_public_id: row.try_get("chat_public_id").unwrap_or("unknown".to_string()),
                chat_title: row.try_get("chat_title").unwrap_or("Unknown Chat".to_string()),
                inviter_id: row.try_get("inviter_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invited_by_public_id: row.try_get("invited_by_public_id").unwrap_or("unknown".to_string()),
                inviter_display_name: row.try_get("inviter_display_name").ok(),
                inviter_avatar_url: row.try_get("inviter_avatar_url").ok(),
                invited_email: row.try_get("invited_email").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invite_code: row.try_get("invite_code").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: InviteStatus::from(status_str.as_str()),
                expires_at: row.try_get("expires_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                accepted_at: row.try_get("accepted_at").ok(),
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(invites)
    }

    /// Find invites for a specific email
    pub async fn find_by_email(&self, email: &str) -> ChatResult<Vec<ChatInvite>> {
        let rows = sqlx::query(
            "SELECT i.id, i.public_id, i.chat_id, i.inviter_id, i.invited_email, i.invite_code,
                    i.status, i.expires_at, i.created_at, i.accepted_at,
                    c.public_id as chat_public_id, c.title as chat_title,
                    u.public_id as invited_by_public_id, u.display_name as inviter_display_name, u.avatar_url as inviter_avatar_url
             FROM chat_invites i
             LEFT JOIN chats c ON i.chat_id = c.id
             LEFT JOIN users u ON i.inviter_id = u.id
             WHERE i.invited_email = ? ORDER BY i.created_at DESC"
        )
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let invites = rows.into_iter().map(|row| {
            let status_str: String = row.try_get("status").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatInvite {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                public_id: row.try_get("public_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_public_id: row.try_get("chat_public_id").unwrap_or("unknown".to_string()),
                chat_title: row.try_get("chat_title").unwrap_or("Unknown Chat".to_string()),
                inviter_id: row.try_get("inviter_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invited_by_public_id: row.try_get("invited_by_public_id").unwrap_or("unknown".to_string()),
                inviter_display_name: row.try_get("inviter_display_name").ok(),
                inviter_avatar_url: row.try_get("inviter_avatar_url").ok(),
                invited_email: row.try_get("invited_email").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                invite_code: row.try_get("invite_code").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                status: InviteStatus::from(status_str.as_str()),
                expires_at: row.try_get("expires_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                accepted_at: row.try_get("accepted_at").ok(),
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(invites)
    }

    /// Create a new invite
    pub async fn create(&self, inviter_id: i64, request: &CreateInviteRequest) -> ChatResult<ChatInvite> {
        let public_id = cuid2::cuid();
        let invite_code = self.generate_invite_code().await?;
        let now = chrono::Utc::now().to_rfc3339();

        // Calculate expires_at from expires_in_hours
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(request.expires_in_hours);
        let expires_at_str = expires_at.to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO chat_invites (public_id, chat_id, inviter_id, invited_email, invite_code, status, expires_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&public_id)
        .bind(request.chat_id)
        .bind(inviter_id)
        .bind(&request.invited_email)
        .bind(&invite_code)
        .bind(InviteStatus::Pending.to_string())
        .bind(&expires_at_str)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let invite_id = result.last_insert_rowid();

        info!(
            invite_id = invite_id,
            public_id = %public_id,
            chat_id = request.chat_id,
            inviter_id = inviter_id,
            invite_code = %invite_code,
            "created new chat invite"
        );

        // Get chat details for the response
        let chat_row = sqlx::query("SELECT public_id, title FROM chats WHERE id = ?")
            .bind(request.chat_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        // Get inviter details for the response
        let inviter_row = sqlx::query("SELECT public_id, display_name, avatar_url FROM users WHERE id = ?")
            .bind(inviter_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        Ok(ChatInvite {
            id: invite_id,
            public_id,
            chat_id: request.chat_id,
            chat_public_id: chat_row
                .as_ref()
                .and_then(|r| r.try_get::<String, _>("public_id").ok())
                .unwrap_or("unknown".to_string()),
            chat_title: chat_row
                .as_ref()
                .and_then(|r| r.try_get::<String, _>("title").ok())
                .unwrap_or("Unknown Chat".to_string()),
            inviter_id,
            invited_by_public_id: inviter_row
                .as_ref()
                .and_then(|r| r.try_get::<String, _>("public_id").ok())
                .unwrap_or("unknown".to_string()),
            inviter_display_name: inviter_row.as_ref().and_then(|r| r.try_get("display_name").ok()),
            inviter_avatar_url: inviter_row.as_ref().and_then(|r| r.try_get("avatar_url").ok()),
            invited_email: request.invited_email.clone(),
            invite_code,
            status: InviteStatus::Pending,
            expires_at: expires_at_str,
            created_at: now,
            accepted_at: None,
        })
    }

    /// Accept an invite
    pub async fn accept(&self, public_id: &str, user_id: i64) -> ChatResult<ChatInvite> {
        let invite = self.find_by_public_id(public_id).await?;
        if invite.is_none() {
            return Err(ChatError::InviteNotFound);
        }

        let invite = invite.unwrap();

        // Check if invite is still pending
        if invite.status != InviteStatus::Pending {
            return Err(ChatError::InviteAlreadyUsed);
        }

        // Check if invite has expired
        let expiry_time = chrono::DateTime::parse_from_rfc3339(&invite.expires_at)
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;
        if expiry_time < chrono::Utc::now() {
            return Err(ChatError::InviteExpired);
        }

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE chat_invites SET status = 'accepted' WHERE public_id = ?")
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            public_id = public_id,
            accepted_by = user_id,
            "accepted chat invite"
        );

        // Return updated invite
        let mut updated_invite = invite;
        updated_invite.status = InviteStatus::Accepted;
        Ok(updated_invite)
    }

    /// Decline an invite
    pub async fn decline(&self, public_id: &str, user_id: i64) -> ChatResult<ChatInvite> {
        let invite = self.find_by_public_id(public_id).await?;
        if invite.is_none() {
            return Err(ChatError::InviteNotFound);
        }

        let invite = invite.unwrap();

        // Check if invite is still pending
        if invite.status != InviteStatus::Pending {
            return Err(ChatError::InviteAlreadyUsed);
        }

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE chat_invites SET status = 'rejected' WHERE public_id = ?")
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            public_id = public_id,
            declined_by = user_id,
            "declined chat invite"
        );

        // Return updated invite
        let mut updated_invite = invite;
        updated_invite.status = InviteStatus::Rejected;
        Ok(updated_invite)
    }

    /// Cancel an invite (only by inviter)
    pub async fn cancel(&self, public_id: &str, requester_id: i64) -> ChatResult<()> {
        let invite = self.find_by_public_id(public_id).await?;
        if invite.is_none() {
            return Err(ChatError::InviteNotFound);
        }

        let invite = invite.unwrap();

        // Check if requester is the inviter
        if invite.inviter_id != requester_id {
            return Err(ChatError::Unauthorized);
        }

        // Check if invite is still pending
        if invite.status != InviteStatus::Pending {
            return Err(ChatError::InviteAlreadyUsed);
        }

        sqlx::query("DELETE FROM chat_invites WHERE public_id = ?")
            .bind(public_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            public_id = public_id,
            cancelled_by = requester_id,
            "cancelled chat invite"
        );

        Ok(())
    }

    /// Mark expired invites
    pub async fn mark_expired_invites(&self) -> ChatResult<usize> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE chat_invites SET status = 'expired' WHERE status = 'pending' AND expires_at IS NOT NULL AND expires_at < ?"
        )
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let expired_count = result.rows_affected();

        if expired_count > 0 {
            info!(
                expired_count = expired_count,
                "marked expired invites"
            );
        }

        Ok(expired_count as usize)
    }

    /// Generate a unique invite code
    async fn generate_invite_code(&self) -> ChatResult<String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 10;

        while attempts < MAX_ATTEMPTS {
            let code = format!("{:08}", rand::random::<u32>());

            // Check if code already exists
            let existing = self.find_by_invite_code(&code).await?;
            if existing.is_none() {
                return Ok(code);
            }

            attempts += 1;
        }

        Err(ChatError::DatabaseError("Failed to generate unique invite code".to_string()))
    }

    /// Count active invites for a chat
    pub async fn count_active_invites_for_chat(&self, chat_id: i64) -> ChatResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM chat_invites WHERE chat_id = ? AND status = 'pending'")
            .bind(chat_id)
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
        let db_path = temp_dir.path().join("test_invites.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE chat_invites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_id TEXT NOT NULL UNIQUE,
                chat_id INTEGER NOT NULL,
                inviter_id INTEGER NOT NULL,
                invitee_email TEXT,
                invite_code TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL,
                expires_at TEXT,
                created_at TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_invite() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let invite = repo.create(1, &request).await.unwrap();
        assert!(invite.id > 0);
        assert_eq!(invite.chat_id, 1);
        assert_eq!(invite.inviter_id, 1);
        assert_eq!(invite.invited_email, "test@example.com");
        assert_eq!(invite.status, InviteStatus::Pending);
        assert!(!invite.invite_code.is_empty());
    }

    #[tokio::test]
    async fn test_find_by_public_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let created = repo.create(1, &request).await.unwrap();
        let found = repo.find_by_public_id(&created.public_id).await.unwrap();

        assert!(found.is_some());
        let found_invite = found.unwrap();
        assert_eq!(found_invite.id, created.id);
        assert_eq!(found_invite.public_id, created.public_id);
    }

    #[tokio::test]
    async fn test_find_by_invite_code() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let created = repo.create(1, &request).await.unwrap();
        let found = repo.find_by_invite_code(&created.invite_code).await.unwrap();

        assert!(found.is_some());
        let found_invite = found.unwrap();
        assert_eq!(found_invite.invite_code, created.invite_code);
    }

    #[tokio::test]
    async fn test_accept_invite() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let created = repo.create(1, &request).await.unwrap();
        let accepted = repo.accept(&created.public_id, 2).await.unwrap();

        assert_eq!(accepted.status, InviteStatus::Accepted);
    }

    #[tokio::test]
    async fn test_decline_invite() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let created = repo.create(1, &request).await.unwrap();
        let declined = repo.decline(&created.public_id, 2).await.unwrap();

        assert_eq!(declined.status, InviteStatus::Rejected);
    }

    #[tokio::test]
    async fn test_cancel_invite() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test@example.com".to_string(),
            expires_in_hours: 24,
        };

        let created = repo.create(1, &request).await.unwrap();
        repo.cancel(&created.public_id, 1).await.unwrap();

        let found = repo.find_by_public_id(&created.public_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_chat_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = InviteRepository::new(pool);

        let request1 = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test1@example.com".to_string(),
            expires_in_hours: 24,
        };

        let request2 = CreateInviteRequest {
            chat_id: 1,
            chat_public_id: "test_chat".to_string(),
            invited_by_public_id: "test_user".to_string(),
            invited_email: "test2@example.com".to_string(),
            expires_in_hours: 24,
        };

        repo.create(1, &request1).await.unwrap();
        repo.create(1, &request2).await.unwrap();

        let invites = repo.find_by_chat_id(1).await.unwrap();
        assert_eq!(invites.len(), 2);
    }
}