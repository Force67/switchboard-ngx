//! Member service for managing chat members.

use switchboard_database::{ChatMember, CreateMemberRequest, UpdateMemberRoleRequest, MemberRepository, MemberRole, ChatResult};
use sqlx::SqlitePool;

/// Service for managing member operations
pub struct MemberService {
    member_repository: MemberRepository,
}

impl MemberService {
    /// Create a new member service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            member_repository: MemberRepository::new(pool),
        }
    }

    /// Check if a user is a member of a chat
    pub async fn check_chat_membership(&self, chat_public_id: &str, user_id: i64) -> ChatResult<()> {
        self.member_repository
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await
            .map(|_| ())
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// Check if a user has a specific role or higher in a chat
    pub async fn check_chat_role(&self, chat_public_id: &str, user_id: i64, required_role: MemberRole) -> ChatResult<()> {
        let member = self.member_repository
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))?
            .ok_or(switchboard_database::ChatError::MemberNotFound)?;

        if !member.has_role_or_higher(&required_role) {
            return Err(switchboard_database::ChatError::AccessDenied);
        }

        Ok(())
    }

    /// List members of a chat with optional role filtering
    pub async fn list_by_chat(
        &self,
        chat_public_id: &str,
        role_filter: Option<MemberRole>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ChatResult<Vec<ChatMember>> {
        self.member_repository
            .list_by_chat_public(chat_public_id, role_filter, limit, offset)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// Get a member by their public ID
    pub async fn get_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatMember>> {
        self.member_repository
            .find_by_public_id(public_id)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// Update a member's role
    pub async fn update_role(
        &self,
        member_id: i64,
        request: &UpdateMemberRoleRequest,
        updated_by: i64,
    ) -> ChatResult<ChatMember> {
        self.member_repository
            .update_role_by_id(member_id, &request.role, updated_by)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// Remove a member by their ID
    pub async fn remove(&self, member_id: i64, removed_by: i64) -> ChatResult<()> {
        self.member_repository
            .delete_by_id(member_id, removed_by)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// Remove a member by user and chat public IDs
    pub async fn remove_by_user_chat(&self, chat_public_id: &str, user_id: i64) -> ChatResult<()> {
        self.member_repository
            .delete_by_user_and_chat_public(chat_public_id, user_id)
            .await
            .map_err(|e| switchboard_database::ChatError::DatabaseError(e.to_string()))
    }

    /// List members of a chat (legacy method for compatibility)
    pub async fn list_members(&self, chat_id: &str, user_id: i64) -> ChatResult<Vec<ChatMember>> {
        todo!("Implement list_members")
    }

    /// Add a member to a chat
    pub async fn add_member(
        &self,
        chat_id: &str,
        user_id: i64,
        request: CreateMemberRequest,
    ) -> ChatResult<ChatMember> {
        todo!("Implement add_member")
    }

    /// Update member role (legacy method for compatibility)
    pub async fn update_member_role(
        &self,
        chat_id: &str,
        user_id: i64,
        member_user_id: i64,
        new_role: String,
    ) -> ChatResult<ChatMember> {
        todo!("Implement update_member_role")
    }

    /// Remove a member from a chat (legacy method for compatibility)
    pub async fn remove_member(
        &self,
        chat_id: &str,
        user_id: i64,
        member_user_id: i64,
    ) -> ChatResult<()> {
        todo!("Implement remove_member")
    }
}