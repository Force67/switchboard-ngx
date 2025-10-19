//! Attachment service for managing message attachments.

use switchboard_database::{MessageAttachment, CreateAttachmentRequest, AttachmentRepository, ChatResult, MessageRepository, MemberRepository, MemberRole};
use sqlx::SqlitePool;

/// Service for managing attachment operations
pub struct AttachmentService {
    attachment_repository: AttachmentRepository,
    message_repository: MessageRepository,
    member_repository: MemberRepository,
}

impl AttachmentService {
    /// Create a new attachment service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            attachment_repository: AttachmentRepository::new(pool.clone()),
            message_repository: MessageRepository::new(pool.clone()),
            member_repository: MemberRepository::new(pool),
        }
    }

    /// Get attachment by public ID
    pub async fn get_by_public_id(&self, public_id: &str) -> ChatResult<Option<MessageAttachment>> {
        self.attachment_repository.find_by_public_id(public_id).await
    }

    /// Get message by public ID
    pub async fn get_message_by_public_id(&self, public_id: &str) -> ChatResult<Option<switchboard_database::ChatMessage>> {
        self.message_repository.find_by_public_id(public_id).await
    }

    /// Create a new attachment
    pub async fn create(&self, request: &CreateAttachmentRequest) -> ChatResult<MessageAttachment> {
        self.attachment_repository.create(request).await
    }

    /// Delete an attachment by public ID
    pub async fn delete_by_public_id(&self, public_id: &str) -> ChatResult<()> {
        self.attachment_repository.delete(public_id).await
    }

    /// Delete an attachment
    pub async fn delete(&self, attachment_id: i64, user_id: i64) -> ChatResult<()> {
        // Convert numeric ID to public_id lookup
        // For now, we'll implement a simple lookup method
        // In a real implementation, you might add a method to find by numeric ID
        todo!("Implement delete by numeric ID with proper lookup")
    }

    /// List attachments by message ID
    pub async fn list_by_message(&self, message_public_id: &str, limit: Option<i64>, offset: Option<i64>) -> ChatResult<Vec<MessageAttachment>> {
        // Get the message by public ID first
        let message = self.message_repository.find_by_public_id(message_public_id)
            .await?
            .ok_or(switchboard_database::ChatError::MessageNotFound)?;

        // Then find attachments by the numeric message ID
        self.attachment_repository.find_by_message_id(message.id).await
    }

    /// List attachments by chat ID
    pub async fn list_by_chat(
        &self,
        chat_public_id: &str,
        message_public_id: Option<&str>,
        file_type_filter: Option<switchboard_database::AttachmentType>,
        limit: Option<i64>,
        offset: Option<i64>
    ) -> ChatResult<Vec<MessageAttachment>> {
        // If message_public_id is provided, list attachments for that message
        if let Some(msg_id) = message_public_id {
            return self.list_by_message(msg_id, limit, offset).await;
        }

        // Otherwise, list all attachments for the chat
        // This would require additional repository methods to join attachments with messages and chats
        // For now, return empty result as this is a more complex query
        Ok(vec![])
    }

    /// Check if user is a member of the chat
    pub async fn check_chat_membership(&self, chat_public_id: &str, user_id: i64) -> ChatResult<()> {
        let member = self.member_repository.find_by_user_and_chat_public(chat_public_id, user_id).await?;
        if member.is_none() {
            return Err(switchboard_database::ChatError::AccessDenied);
        }
        Ok(())
    }

    /// Check if user has specific role in chat
    pub async fn check_chat_role(&self, chat_public_id: &str, user_id: i64, required_role: MemberRole) -> ChatResult<()> {
        let member = self.member_repository.find_by_user_and_chat_public(chat_public_id, user_id)
            .await?
            .ok_or(switchboard_database::ChatError::AccessDenied)?;

        if !member.has_role_or_higher(&required_role) {
            return Err(switchboard_database::ChatError::AccessDenied);
        }

        Ok(())
    }
}