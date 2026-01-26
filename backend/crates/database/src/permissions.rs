//! Permission utilities for chat access control.
//!
//! Simple functions for upfront permission checks in handlers.

use crate::repos::MemberRepository;
use crate::types::{ChatError, ChatResult};
use crate::MemberRole;

/// Permission checking utilities that wrap MemberRepository.
pub struct Permissions<'a> {
    member_repo: &'a MemberRepository,
}

impl<'a> Permissions<'a> {
    /// Create a new Permissions instance.
    pub fn new(member_repo: &'a MemberRepository) -> Self {
        Self { member_repo }
    }

    /// Require that a user is a member of a chat.
    /// Returns an error if the user is not a member.
    pub async fn require_membership(&self, chat_public_id: &str, user_id: i64) -> ChatResult<()> {
        let member = self
            .member_repo
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await?;

        if member.is_none() {
            return Err(ChatError::AccessDenied);
        }

        Ok(())
    }

    /// Require that a user has at least the specified role in a chat.
    /// Returns an error if the user doesn't have the required role.
    pub async fn require_role(
        &self,
        chat_public_id: &str,
        user_id: i64,
        min_role: MemberRole,
    ) -> ChatResult<()> {
        let member = self
            .member_repo
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await?
            .ok_or(ChatError::AccessDenied)?;

        if !member.has_role_or_higher(&min_role) {
            return Err(ChatError::AccessDenied);
        }

        Ok(())
    }

    /// Get the role of a user in a chat, if they are a member.
    pub async fn get_role(
        &self,
        chat_public_id: &str,
        user_id: i64,
    ) -> ChatResult<Option<MemberRole>> {
        let member = self
            .member_repo
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await?;

        Ok(member.map(|m| m.role))
    }

    /// Check if a user is a member of a chat (without erroring if not).
    pub async fn is_member(&self, chat_public_id: &str, user_id: i64) -> ChatResult<bool> {
        let member = self
            .member_repo
            .find_by_user_and_chat_public(chat_public_id, user_id)
            .await?;

        Ok(member.is_some())
    }
}
