//! Member service for managing chat membership.

use crate::entities::{ChatMember, MemberRole};
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Service for managing chat membership operations
pub struct MemberService {
    pool: SqlitePool,
}

impl MemberService {
    /// Create a new member service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// List members of a chat
    pub async fn list_members(&self, chat_id: &str, user_id: i64) -> ChatResult<Vec<ChatMember>> {
        todo!("Implement list_members")
    }

    /// Update member role
    pub async fn update_member_role(
        &self,
        chat_id: &str,
        member_user_id: i64,
        requester_user_id: i64,
        new_role: MemberRole,
    ) -> ChatResult<ChatMember> {
        todo!("Implement update_member_role")
    }

    /// Remove member from chat
    pub async fn remove_member(
        &self,
        chat_id: &str,
        member_user_id: i64,
        requester_user_id: i64,
    ) -> ChatResult<()> {
        todo!("Implement remove_member")
    }
}