//! Invite service for managing chat invitations.

use sqlx::SqlitePool;
use switchboard_database::{
    ChatInvite, ChatRepository, ChatResult, CreateInviteRequest, InviteRepository, InviteStatus,
    MemberRepository, MemberRole,
};

/// Service for managing chat invitation operations
pub struct InviteService {
    invite_repository: InviteRepository,
    member_repository: MemberRepository,
    chat_repository: ChatRepository,
}

impl InviteService {
    /// Create a new invite service instance
    pub fn new(pool: SqlitePool) -> Self {
        let member_pool = pool.clone();
        let chat_pool = pool.clone();
        Self {
            invite_repository: InviteRepository::new(pool),
            member_repository: MemberRepository::new(member_pool),
            chat_repository: ChatRepository::new(chat_pool),
        }
    }

    /// Create a new invitation
    pub async fn create_invite(
        &self,
        chat_id: &str,
        inviter_user_id: i64,
        request: CreateInviteRequest,
    ) -> ChatResult<ChatInvite> {
        // Check if user has permission to invite (admin or owner)
        self.check_chat_role(chat_id, inviter_user_id, MemberRole::Admin)
            .await?;

        self.invite_repository.create(inviter_user_id, &request).await
    }

    /// List invitations for a chat
    pub async fn list_invites(&self, chat_id: &str, user_id: i64) -> ChatResult<Vec<ChatInvite>> {
        // Check if user has permission to see invites
        self.check_chat_role(chat_id, user_id, MemberRole::Admin)
            .await?;

        // Get the chat to find the internal ID
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        self.invite_repository.find_by_chat_id(chat.id).await
    }

    /// Accept an invitation (legacy method)
    pub async fn accept_invite_legacy(
        &self,
        invite_id: &str,
        user_id: i64,
        _user_email: Option<&str>,
    ) -> ChatResult<()> {
        // Accept the invite
        let invite = self.invite_repository.accept(invite_id, user_id).await?;

        // Add the user as a member of the chat
        let member_request = switchboard_database::CreateMemberRequest {
            chat_id: invite.chat_id,
            user_id,
            role: MemberRole::Member,
        };
        self.member_repository.create(&member_request).await?;

        Ok(())
    }

    /// Decline an invitation
    pub async fn decline_invite(
        &self,
        invite_id: &str,
        user_id: i64,
        _user_email: Option<&str>,
    ) -> ChatResult<()> {
        self.invite_repository.decline(invite_id, user_id).await?;
        Ok(())
    }

    /// Check if user has specific role in chat
    pub async fn check_chat_role(
        &self,
        chat_id: &str,
        user_id: i64,
        role: MemberRole,
    ) -> ChatResult<()> {
        // Allow creator or members with required role
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        if chat.created_by == user_id.to_string() {
            return Ok(());
        }

        if let Some(member) = self
            .member_repository
            .find_by_chat_and_user(chat.id, user_id)
            .await?
        {
            if member.role.is_higher_or_equal(&role) {
                return Ok(());
            }
        }

        Err(switchboard_database::ChatError::Unauthorized)
    }

    /// List invitations by chat
    pub async fn list_by_chat(
        &self,
        chat_id: &str,
        status_filter: Option<InviteStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ChatResult<Vec<ChatInvite>> {
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        self.invite_repository
            .find_by_chat_id(chat.id)
            .await
            .map(|invites| {
                invites
                    .into_iter()
                    .filter(|invite| {
                        if let Some(filter) = status_filter.as_ref() {
                            &invite.status == filter
                        } else {
                            true
                        }
                    })
                    .skip(offset.unwrap_or(0) as usize)
                    .take(limit.unwrap_or(i64::MAX) as usize)
                    .collect()
            })
    }

    /// List invitations by user
    pub async fn list_by_user(
        &self,
        user_id: i64,
        status_filter: Option<InviteStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ChatResult<Vec<ChatInvite>> {
        self.invite_repository
            .find_by_inviter_id(user_id)
            .await
            .map(|invites| {
                invites
                    .into_iter()
                    .filter(|invite| {
                        if let Some(filter) = status_filter.as_ref() {
                            &invite.status == filter
                        } else {
                            true
                        }
                    })
                    .skip(offset.unwrap_or(0) as usize)
                    .take(limit.unwrap_or(i64::MAX) as usize)
                    .collect()
            })
    }

    /// Create a new invitation
    pub async fn create(&self, request: &CreateInviteRequest) -> ChatResult<ChatInvite> {
        // Resolve chat public ID to internal ID
        let chat = self
            .chat_repository
            .find_by_public_id(&request.chat_public_id)
            .await?
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        let req = switchboard_database::CreateInviteRequest {
            chat_id: chat.id,
            chat_public_id: request.chat_public_id.clone(),
            invited_by_public_id: request.invited_by_public_id.clone(),
            invited_email: request.invited_email.clone(),
            expires_in_hours: request.expires_in_hours,
        };

        self.invite_repository.create(chat.id, &req).await
    }

    /// Get an invitation by public ID
    pub async fn get_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatInvite>> {
        self.invite_repository.find_by_public_id(public_id).await
    }

    /// Accept an invitation
    pub async fn accept_invite(&self, invite_id: i64, user_id: i64) -> ChatResult<ChatInvite> {
        // Find the invite by ID (we need to get its public_id first)
        let invite = self
            .invite_repository
            .find_by_public_id(&format!("invite_{}", invite_id))
            .await?
            .ok_or(switchboard_database::ChatError::InviteNotFound)?;

        // Accept the invite using the repository
        let accepted_invite = self
            .invite_repository
            .accept(&invite.public_id, user_id)
            .await?;

        // Add the user as a member of the chat
        let member_request = switchboard_database::CreateMemberRequest {
            chat_id: accepted_invite.chat_id,
            user_id,
            role: MemberRole::Member,
        };
        self.member_repository.create(&member_request).await?;

        Ok(accepted_invite)
    }

    /// Reject an invitation
    pub async fn reject_invite(&self, invite_id: i64, user_id: i64) -> ChatResult<ChatInvite> {
        // Find the invite by ID (we need to get its public_id first)
        let invite = self
            .invite_repository
            .find_by_public_id(&format!("invite_{}", invite_id))
            .await?
            .ok_or(switchboard_database::ChatError::InviteNotFound)?;

        // Decline the invite using the repository
        self.invite_repository.decline(&invite.public_id, user_id).await
    }

    /// Delete an invitation
    pub async fn delete(&self, invite_id: i64) -> ChatResult<()> {
        // Find the invite to get its public_id and inviter_id
        let invite = self
            .invite_repository
            .find_by_public_id(&format!("invite_{}", invite_id))
            .await?
            .ok_or(switchboard_database::ChatError::InviteNotFound)?;

        // Cancel the invite (only the inviter can cancel)
        self.invite_repository
            .cancel(&invite.public_id, invite.inviter_id)
            .await
    }
}
