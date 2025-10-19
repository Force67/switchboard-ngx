//! Repository for invite data access operations.

use crate::entities::ChatInvite;
use crate::types::{ChatResult, ChatError};
use sqlx::SqlitePool;

/// Repository for invite database operations
pub struct InviteRepository {
    pool: SqlitePool,
}

impl InviteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatInvite>> {
        todo!("Implement find_by_public_id")
    }

    pub async fn find_by_chat_id(&self, chat_id: i64) -> ChatResult<Vec<ChatInvite>> {
        todo!("Implement find_by_chat_id")
    }

    pub async fn create(&self, invite: &ChatInvite) -> ChatResult<ChatInvite> {
        todo!("Implement create")
    }

    pub async fn update(&self, invite: &ChatInvite) -> ChatResult<ChatInvite> {
        todo!("Implement update")
    }
}