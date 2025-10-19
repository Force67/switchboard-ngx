//! Chat service for managing chat operations.

use switchboard_database::{Chat, CreateChatRequest, UpdateChatRequest, ChatRepository, ChatResult, MemberRole, MemberRepository};
use sqlx::SqlitePool;

/// Service for managing chat operations
pub struct ChatService {
    chat_repository: ChatRepository,
    member_repository: MemberRepository,
}

impl ChatService {
    /// Create a new chat service instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            chat_repository: ChatRepository::new(pool.clone()),
            member_repository: MemberRepository::new(pool),
        }
    }

    /// List all chats for a user
    pub async fn list_chats(&self, user_id: i64) -> ChatResult<Vec<Chat>> {
        self.chat_repository.find_by_user_id(user_id).await
    }

    /// List user's chats by folder
    pub async fn list_user_chats(&self, user_id: i64, folder_id: Option<String>) -> ChatResult<Vec<Chat>> {
        let chats = self.chat_repository.find_by_user_id(user_id).await?;

        if let Some(folder_id) = folder_id {
            let filtered_chats = chats.into_iter()
                .filter(|chat| chat.folder_id.as_ref() == Some(&folder_id))
                .collect();
            Ok(filtered_chats)
        } else {
            Ok(chats)
        }
    }

    /// Create a new chat
    pub async fn create(&self, request: &CreateChatRequest) -> ChatResult<Chat> {
        let user_id = request.created_by.parse::<i64>()
            .map_err(|_| switchboard_database::ChatError::DatabaseError("Invalid created_by user ID".to_string()))?;

        self.chat_repository.create(user_id, request).await
    }

    /// Create a new chat (legacy method)
    pub async fn create_chat(&self, user_id: i64, request: CreateChatRequest) -> ChatResult<Chat> {
        let create_req = switchboard_database::CreateChatRequest {
            title: request.title,
            description: request.description,
            avatar_url: request.avatar_url,
            folder_id: request.folder_id,
            chat_type: request.chat_type,
            created_by: user_id.to_string(),
        };

        self.chat_repository.create(user_id, &create_req).await
    }

    /// Get a chat by public ID
    pub async fn get_by_public_id(&self, public_id: &str) -> ChatResult<Option<Chat>> {
        self.chat_repository.find_by_public_id(public_id).await
    }

    /// Get a specific chat
    pub async fn get_chat(&self, chat_id: &str, user_id: i64) -> ChatResult<Chat> {
        let chat = self.get_by_public_id(chat_id).await?
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        // Check if user is member or creator
        if chat.created_by != user_id.to_string() {
            self.check_membership(chat.id, user_id).await?;
        }

        Ok(chat)
    }

    /// Update a chat
    pub async fn update(&self, chat_id: i64, request: &UpdateChatRequest) -> ChatResult<Chat> {
        // First get the chat to find its public_id
        let chats = self.chat_repository.find_by_user_id(0).await?; // This is inefficient but needed for now
        let chat = chats.iter()
            .find(|c| c.id == chat_id)
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        let public_id = &chat.public_id;
        let user_id = chat.created_by.parse::<i64>()
            .map_err(|_| switchboard_database::ChatError::DatabaseError("Invalid created_by user ID".to_string()))?;

        self.chat_repository.update(public_id, user_id, request).await
    }

    /// Update a chat (legacy method)
    pub async fn update_chat(
        &self,
        chat_id: &str,
        user_id: i64,
        request: UpdateChatRequest,
    ) -> ChatResult<(Chat, Vec<i64>)> {
        let chat = self.chat_repository.update(chat_id, user_id, &request).await?;
        Ok((chat, vec![])) // Return empty member list for now
    }

    /// Delete a chat
    pub async fn delete(&self, chat_id: i64) -> ChatResult<()> {
        // First get the chat to find its public_id
        let chats = self.chat_repository.find_by_user_id(0).await?; // This is inefficient but needed for now
        let chat = chats.iter()
            .find(|c| c.id == chat_id)
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        let public_id = &chat.public_id;
        let user_id = chat.created_by.parse::<i64>()
            .map_err(|_| switchboard_database::ChatError::DatabaseError("Invalid created_by user ID".to_string()))?;

        self.chat_repository.delete(public_id, user_id).await
    }

    /// Delete a chat (legacy method)
    pub async fn delete_chat(&self, chat_id: &str, user_id: i64) -> ChatResult<Vec<i64>> {
        self.chat_repository.delete(chat_id, user_id).await?;
        Ok(vec![]) // Return empty member list for now
    }

    /// Check if user is a member of chat
    pub async fn check_membership(&self, chat_id: i64, user_id: i64) -> ChatResult<()> {
        // Get chat by numeric ID (this is a simplified approach)
        let chats = self.chat_repository.find_by_user_id(user_id).await?;
        let chat = chats.iter()
            .find(|c| c.id == chat_id)
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        // Check if user is creator
        if chat.created_by == user_id.to_string() {
            return Ok(());
        }

        // Check if user is a member via member repository
        let member = self.member_repository.find_by_user_and_chat(user_id, &chat.public_id).await?;
        if member.is_none() {
            return Err(switchboard_database::ChatError::AccessDenied);
        }

        Ok(())
    }

    /// Check if user has specific role or higher in chat
    pub async fn check_role(&self, chat_id: i64, user_id: i64, role: MemberRole) -> ChatResult<()> {
        // Get chat by numeric ID (this is a simplified approach)
        let chats = self.chat_repository.find_by_user_id(user_id).await?;
        let chat = chats.iter()
            .find(|c| c.id == chat_id)
            .ok_or(switchboard_database::ChatError::ChatNotFound)?;

        // Check if user is creator (owner role)
        if chat.created_by == user_id.to_string() {
            return Ok(());
        }

        // Check member role
        let member = self.member_repository.find_by_user_and_chat(user_id, &chat.public_id).await?
            .ok_or(switchboard_database::ChatError::AccessDenied)?;

        if !member.has_role_or_higher(&role) {
            return Err(switchboard_database::ChatError::AccessDenied);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_service_creation() {
        // TODO: Add tests when service is implemented
        assert!(true);
    }
}