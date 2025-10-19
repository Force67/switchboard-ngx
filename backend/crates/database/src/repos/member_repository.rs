//! Repository for member data access operations.

use crate::entities::{ChatMember, MemberRole, CreateMemberRequest};
use crate::types::{ChatResult, ChatError};
use sqlx::{SqlitePool, Row};
use tracing::{info, warn};

/// Repository for member database operations
pub struct MemberRepository {
    pool: SqlitePool,
}

impl MemberRepository {
    /// Create a new member repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find all members for a chat
    pub async fn find_by_chat_id(&self, chat_id: i64) -> ChatResult<Vec<ChatMember>> {
        let rows = sqlx::query(
            "SELECT id, chat_id, user_id, role, joined_at
             FROM chat_members WHERE chat_id = ? ORDER BY joined_at ASC"
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let members = rows.into_iter().map(|row| {
            let role_str: String = row.try_get("role").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatMember {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                role: MemberRole::from(role_str.as_str()),
                joined_at: row.try_get("joined_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(members)
    }

    /// Find member by chat ID and user ID
    pub async fn find_by_chat_and_user(&self, chat_id: i64, user_id: i64) -> ChatResult<Option<ChatMember>> {
        let row = sqlx::query(
            "SELECT id, chat_id, user_id, role, joined_at
             FROM chat_members WHERE chat_id = ? AND user_id = ?"
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let role_str: String = row.try_get("role").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(Some(ChatMember {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                role: MemberRole::from(role_str.as_str()),
                joined_at: row.try_get("joined_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find all chats for a user
    pub async fn find_chats_by_user_id(&self, user_id: i64) -> ChatResult<Vec<ChatMember>> {
        let rows = sqlx::query(
            "SELECT id, chat_id, user_id, role, joined_at
             FROM chat_members WHERE user_id = ? ORDER BY joined_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let members = rows.into_iter().map(|row| {
            let role_str: String = row.try_get("role").map_err(|e| ChatError::DatabaseError(e.to_string()))?;

            Ok(ChatMember {
                id: row.try_get("id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                chat_id: row.try_get("chat_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                user_id: row.try_get("user_id").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
                role: MemberRole::from(role_str.as_str()),
                joined_at: row.try_get("joined_at").map_err(|e| ChatError::DatabaseError(e.to_string()))?,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(members)
    }

    /// Create a new member
    pub async fn create(&self, request: &CreateMemberRequest) -> ChatResult<ChatMember> {
        // Check if member already exists
        if let Some(_existing) = self.find_by_chat_and_user(request.chat_id, request.user_id).await? {
            return Err(ChatError::MemberAlreadyExists);
        }

        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO chat_members (chat_id, user_id, role, joined_at) VALUES (?, ?, ?, ?)"
        )
        .bind(request.chat_id)
        .bind(request.user_id)
        .bind(request.role.to_string())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let member_id = result.last_insert_rowid();

        info!(
            member_id = member_id,
            chat_id = request.chat_id,
            user_id = request.user_id,
            role = %request.role.to_string(),
            "added new member to chat"
        );

        Ok(ChatMember {
            id: member_id,
            chat_id: request.chat_id,
            user_id: request.user_id,
            role: request.role.clone(),
            joined_at: now,
        })
    }

    /// Update a member's role
    pub async fn update_role(
        &self,
        chat_id: i64,
        user_id: i64,
        new_role: MemberRole,
        requester_id: i64,
    ) -> ChatResult<ChatMember> {
        // Check if member exists
        let member = self.find_by_chat_and_user(chat_id, user_id).await?;
        if member.is_none() {
            return Err(ChatError::MemberNotFound);
        }

        // Check if requester has permission (owner or admin)
        let requester_member = self.find_by_chat_and_user(chat_id, requester_id).await?;
        if requester_member.is_none() {
            return Err(ChatError::Unauthorized);
        }

        let requester_role = requester_member.unwrap().role;
        if requester_role != MemberRole::Owner && requester_role != MemberRole::Admin {
            return Err(ChatError::Unauthorized);
        }

        // Don't allow changing role of owners unless requester is also an owner
        let current_member = member.unwrap();
        if current_member.role == MemberRole::Owner && requester_role != MemberRole::Owner {
            return Err(ChatError::Unauthorized);
        }

        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE chat_members SET role = ?, joined_at = ? WHERE chat_id = ? AND user_id = ?")
            .bind(new_role.to_string())
            .bind(&now)
            .bind(chat_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            chat_id = chat_id,
            user_id = user_id,
            new_role = %new_role.to_string(),
            updated_by = requester_id,
            "updated member role"
        );

        self.find_by_chat_and_user(chat_id, user_id).await.map(|m| m.unwrap())
    }

    /// Remove a member from a chat
    pub async fn delete(&self, chat_id: i64, user_id: i64, requester_id: i64) -> ChatResult<()> {
        // Check if member exists
        let member = self.find_by_chat_and_user(chat_id, user_id).await?;
        if member.is_none() {
            return Err(ChatError::MemberNotFound);
        }

        let current_member = member.unwrap();

        // Check if requester has permission (owner, admin, or the member themselves)
        let requester_member = self.find_by_chat_and_user(chat_id, requester_id).await?;
        if requester_member.is_none() {
            return Err(ChatError::Unauthorized);
        }

        let requester_role = requester_member.unwrap().role;

        // Allow self-removal, or removal by owner/admin
        let can_remove = requester_id == user_id
            || requester_role == MemberRole::Owner
            || requester_role == MemberRole::Admin;

        if !can_remove {
            return Err(ChatError::Unauthorized);
        }

        // Don't allow removing owners unless they're removing themselves
        if current_member.role == MemberRole::Owner && requester_id != user_id {
            return Err(ChatError::Unauthorized);
        }

        sqlx::query("DELETE FROM chat_members WHERE chat_id = ? AND user_id = ?")
            .bind(chat_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        info!(
            chat_id = chat_id,
            user_id = user_id,
            removed_by = requester_id,
            "removed member from chat"
        );

        Ok(())
    }

    /// Count members for a chat
    pub async fn count_members_for_chat(&self, chat_id: i64) -> ChatResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM chat_members WHERE chat_id = ?")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ChatError::DatabaseError(e.to_string()))?;

        let count = row
            .map(|r| r.try_get::<i64, _>("count").unwrap_or(0))
            .unwrap_or(0);

        Ok(count)
    }

    /// Check if a user is a member of a chat
    pub async fn is_member(&self, chat_id: i64, user_id: i64) -> ChatResult<bool> {
        let member = self.find_by_chat_and_user(chat_id, user_id).await?;
        Ok(member.is_some())
    }

    /// Get member role for a user in a chat
    pub async fn get_member_role(&self, chat_id: i64, user_id: i64) -> ChatResult<Option<MemberRole>> {
        let member = self.find_by_chat_and_user(chat_id, user_id).await?;
        Ok(member.map(|m| m.role))
    }

    /// Check if a user has admin or owner permissions in a chat
    pub async fn has_admin_permissions(&self, chat_id: i64, user_id: i64) -> ChatResult<bool> {
        let role = self.get_member_role(chat_id, user_id).await?;
        Ok(matches!(role, Some(MemberRole::Owner) | Some(MemberRole::Admin)))
    }

    /// Check if a user is the owner of a chat
    pub async fn is_owner(&self, chat_id: i64, user_id: i64) -> ChatResult<bool> {
        let role = self.get_member_role(chat_id, user_id).await?;
        Ok(matches!(role, Some(MemberRole::Owner)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_members.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = SqlitePool::connect(&db_url).await.unwrap();

        // Create test schema
        sqlx::query(
            "CREATE TABLE chat_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                UNIQUE(chat_id, user_id)
            )"
        )
        .execute(&pool)
        .await
        .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_member() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        let request = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Owner,
        };

        let member = repo.create(&request).await.unwrap();
        assert!(member.id > 0);
        assert_eq!(member.chat_id, 1);
        assert_eq!(member.user_id, 1);
        assert_eq!(member.role, MemberRole::Owner);
    }

    #[tokio::test]
    async fn test_find_by_chat_id() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        let request1 = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Owner,
        };

        let request2 = CreateMemberRequest {
            chat_id: 1,
            user_id: 2,
            role: MemberRole::Member,
        };

        repo.create(&request1).await.unwrap();
        repo.create(&request2).await.unwrap();

        let members = repo.find_by_chat_id(1).await.unwrap();
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_chat_and_user() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        let request = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Admin,
        };

        repo.create(&request).await.unwrap();

        let member = repo.find_by_chat_and_user(1, 1).await.unwrap();
        assert!(member.is_some());
        assert_eq!(member.unwrap().role, MemberRole::Admin);
    }

    #[tokio::test]
    async fn test_update_role() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        // Create owner
        let owner_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Owner,
        };
        repo.create(&owner_request).await.unwrap();

        // Create member
        let member_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 2,
            role: MemberRole::Member,
        };
        repo.create(&member_request).await.unwrap();

        // Update member to admin
        let updated = repo.update_role(1, 2, MemberRole::Admin, 1).await.unwrap();
        assert_eq!(updated.role, MemberRole::Admin);
    }

    #[tokio::test]
    async fn test_delete_member() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        // Create owner
        let owner_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Owner,
        };
        repo.create(&owner_request).await.unwrap();

        // Create member
        let member_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 2,
            role: MemberRole::Member,
        };
        repo.create(&member_request).await.unwrap();

        // Remove member
        repo.delete(1, 2, 1).await.unwrap();

        let member = repo.find_by_chat_and_user(1, 2).await.unwrap();
        assert!(member.is_none());
    }

    #[tokio::test]
    async fn test_permission_checks() {
        let (pool, _temp_dir) = create_test_pool().await;
        let repo = MemberRepository::new(pool);

        // Create owner
        let owner_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 1,
            role: MemberRole::Owner,
        };
        repo.create(&owner_request).await.unwrap();

        // Create admin
        let admin_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 2,
            role: MemberRole::Admin,
        };
        repo.create(&admin_request).await.unwrap();

        // Create member
        let member_request = CreateMemberRequest {
            chat_id: 1,
            user_id: 3,
            role: MemberRole::Member,
        };
        repo.create(&member_request).await.unwrap();

        // Test permission checks
        assert!(repo.is_owner(1, 1).await.unwrap());
        assert!(!repo.is_owner(1, 2).await.unwrap());
        assert!(!repo.is_owner(1, 3).await.unwrap());

        assert!(repo.has_admin_permissions(1, 1).await.unwrap());
        assert!(repo.has_admin_permissions(1, 2).await.unwrap());
        assert!(!repo.has_admin_permissions(1, 3).await.unwrap());

        assert!(repo.is_member(1, 1).await.unwrap());
        assert!(repo.is_member(1, 2).await.unwrap());
        assert!(repo.is_member(1, 3).await.unwrap());
        assert!(!repo.is_member(1, 4).await.unwrap());
    }
}