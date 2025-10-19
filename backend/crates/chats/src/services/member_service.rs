//! Member service for managing chat members.

use switchboard_database::{ChatMember, CreateMemberRequest, MemberRepository, ChatResult};
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

    /// List members of a chat
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

    /// Update member role
    pub async fn update_member_role(
        &self,
        chat_id: &str,
        user_id: i64,
        member_user_id: i64,
        new_role: String,
    ) -> ChatResult<ChatMember> {
        todo!("Implement update_member_role")
    }

    /// Remove a member from a chat
    pub async fn remove_member(
        &self,
        chat_id: &str,
        user_id: i64,
        member_user_id: i64,
    ) -> ChatResult<()> {
        todo!("Implement remove_member")
    }
}