//! Repository for member data access operations.

use crate::entities::ChatMember;
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Repository for member database operations
pub struct MemberRepository {
    pool: SqlitePool,
}

impl MemberRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_chat_id(&self, chat_id: i64) -> ChatResult<Vec<ChatMember>> {
        todo!("Implement find_by_chat_id")
    }

    pub async fn create(&self, member: &ChatMember) -> ChatResult<ChatMember> {
        todo!("Implement create")
    }

    pub async fn update(&self, member: &ChatMember) -> ChatResult<ChatMember> {
        todo!("Implement update")
    }

    pub async fn delete(&self, chat_id: i64, user_id: i64) -> ChatResult<()> {
        todo!("Implement delete")
    }
}