//! Message service for managing message operations.

use sqlx::SqlitePool;
use switchboard_database::{
    ChatError, ChatMessage, ChatRepository, ChatResult, CreateMessageRequest, MemberRepository,
    MemberRole, MessageRepository, UpdateMessageRequest,
};

/// Service for managing message operations
pub struct MessageService {
    message_repository: MessageRepository,
    member_repository: MemberRepository,
    chat_repository: ChatRepository,
}

impl MessageService {
    /// Create a new message service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            message_repository: MessageRepository::new(pool.clone()),
            member_repository: MemberRepository::new(pool.clone()),
            chat_repository: ChatRepository::new(pool),
        }
    }

    /// Get messages for a chat
    pub async fn get_messages(&self, chat_id: &str, user_id: i64) -> ChatResult<Vec<ChatMessage>> {
        // First check if user is a member of the chat
        self.check_chat_membership(chat_id, user_id).await?;

        // Get the chat to find the internal ID
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(ChatError::ChatNotFound)?;

        self.message_repository
            .find_by_chat_id(chat.id, None, None)
            .await
    }

    /// Create a new message
    pub async fn create_message(
        &self,
        chat_id: &str,
        user_id: i64,
        request: CreateMessageRequest,
    ) -> ChatResult<ChatMessage> {
        // Check if user is a member of the chat
        self.check_chat_membership(chat_id, user_id).await?;

        self.message_repository.create(user_id, &request).await
    }

    /// Update a message
    pub async fn update_message(
        &self,
        message_id: &str,
        user_id: i64,
        request: UpdateMessageRequest,
    ) -> ChatResult<ChatMessage> {
        self.message_repository.update(message_id, user_id, &request).await
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: &str, user_id: i64) -> ChatResult<()> {
        self.message_repository.delete(message_id, user_id).await
    }

    /// Check if user is a member of chat
    pub async fn check_chat_membership(&self, chat_id: &str, user_id: i64) -> ChatResult<()> {
        // Get the chat to check if user is the creator
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(ChatError::ChatNotFound)?;

        // Check if user is the creator
        if chat.created_by == user_id.to_string() {
            return Ok(());
        }

        // Check if user is a member
        let member = self
            .member_repository
            .find_by_user_and_chat_public(chat_id, user_id)
            .await?;

        if member.is_none() {
            return Err(ChatError::AccessDenied);
        }

        Ok(())
    }

    /// Check if user has specific role in chat
    pub async fn check_chat_role(
        &self,
        chat_id: &str,
        user_id: i64,
        role: MemberRole,
    ) -> ChatResult<()> {
        // Get the chat to check if user is the creator
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(ChatError::ChatNotFound)?;

        // Creator has all permissions
        if chat.created_by == user_id.to_string() {
            return Ok(());
        }

        // Check if user has the required role
        let member = self
            .member_repository
            .find_by_user_and_chat_public(chat_id, user_id)
            .await?
            .ok_or(ChatError::AccessDenied)?;

        if !member.has_role_or_higher(&role) {
            return Err(ChatError::AccessDenied);
        }

        Ok(())
    }

    /// List messages by chat with pagination
    pub async fn list_by_chat(
        &self,
        chat_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
        _before: Option<&str>,
        _after: Option<&str>,
    ) -> ChatResult<Vec<ChatMessage>> {
        // Get the chat to find the internal ID
        let chat = self
            .chat_repository
            .find_by_public_id(chat_id)
            .await?
            .ok_or(ChatError::ChatNotFound)?;

        self.message_repository
            .find_by_chat_id(chat.id, limit, offset)
            .await
    }

    /// Create a new message
    pub async fn create(
        &self,
        request: &CreateMessageRequest,
        user_id: i64,
    ) -> ChatResult<ChatMessage> {
        self.message_repository.create(user_id, request).await
    }

    /// Get a message by public ID
    pub async fn get_by_public_id(&self, public_id: &str) -> ChatResult<Option<ChatMessage>> {
        self.message_repository.find_by_public_id(public_id).await
    }

    /// Update a message
    pub async fn update(
        &self,
        message_id: i64,
        request: &UpdateMessageRequest,
        user_id: i64,
    ) -> ChatResult<ChatMessage> {
        // First find the message to get its public_id
        let message = self
            .message_repository
            .find_by_public_id(&format!("{}", message_id))
            .await?;

        // If not found by numeric id as string, we need to look it up differently
        // For now, we'll use the public_id directly if available
        if let Some(msg) = message {
            self.message_repository.update(&msg.public_id, user_id, request).await
        } else {
            Err(ChatError::MessageNotFound)
        }
    }

    /// Delete a message by ID
    pub async fn delete(&self, message_id: i64, user_id: i64) -> ChatResult<()> {
        // Find the message to get its public_id
        let messages = self.message_repository.find_by_chat_id(message_id, Some(1), None).await?;
        if let Some(msg) = messages.first() {
            self.message_repository.delete(&msg.public_id, user_id).await
        } else {
            Err(ChatError::MessageNotFound)
        }
    }
}
