//! Chats service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::ChatsServiceTrait;
use crate::rest::models::{
    ChatResponse, ChatsResponse, CreateChatRequest, ListChatsQuery, UpdateChatRequest,
};
use crate::state::GatewayState;

pub struct ChatsServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> ChatsServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl ChatsServiceTrait for ChatsServiceImpl<'_> {
    async fn list_chats(
        &self,
        user_id: i64,
        query: ListChatsQuery,
    ) -> GatewayResult<ChatsResponse> {
        let chats = self
            .state
            .chat_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list chats: {}", e)))?;

        // Filter by folder if specified
        let chats = if let Some(folder_id) = query.folder_id {
            chats
                .into_iter()
                .filter(|chat| chat.folder_id.as_ref() == Some(&folder_id))
                .collect()
        } else {
            chats
        };

        let chats: Vec<ChatResponse> = chats.into_iter().map(|chat| chat.into()).collect();
        Ok(ChatsResponse { chats })
    }

    async fn create_chat(
        &self,
        user_id: i64,
        req: CreateChatRequest,
    ) -> GatewayResult<ChatResponse> {
        let chat_type = if req.is_group {
            switchboard_database::ChatType::Group
        } else {
            switchboard_database::ChatType::Direct
        };

        let create_req = switchboard_database::CreateChatRequest {
            title: req.title,
            description: req.description,
            avatar_url: req.avatar_url,
            folder_id: req.folder_id,
            chat_type,
            created_by: user_id.to_string(),
        };

        let chat = self
            .state
            .chat_repo
            .create(user_id, &create_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to create chat: {}", e)))?;

        // Add the creator as an owner member
        let member_req = switchboard_database::CreateMemberRequest {
            chat_id: chat.id,
            user_id,
            role: switchboard_database::MemberRole::Owner,
        };
        let _ = self.state.member_repo.create(&member_req).await;

        Ok(ChatResponse::from(chat))
    }

    async fn get_chat(&self, user_id: i64, chat_id: String) -> GatewayResult<ChatResponse> {
        let chat = self
            .state
            .chat_repo
            .find_by_public_id(&chat_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
            .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

        // Check if user is a member or creator
        if chat.created_by != user_id.to_string() {
            self.state
                .permissions()
                .require_membership(&chat_id, user_id)
                .await
                .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
        }

        Ok(ChatResponse::from(chat))
    }

    async fn update_chat(
        &self,
        user_id: i64,
        chat_id: String,
        req: UpdateChatRequest,
    ) -> GatewayResult<ChatResponse> {
        let update_req = switchboard_database::UpdateChatRequest {
            title: req.title,
            description: req.description,
            avatar_url: req.avatar_url,
            folder_id: req.folder_id,
            status: None,
        };

        let updated_chat = self
            .state
            .chat_repo
            .update(&chat_id, user_id, &update_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to update chat: {}", e)))?;

        Ok(ChatResponse::from(updated_chat))
    }

    async fn delete_chat(&self, user_id: i64, chat_id: String) -> GatewayResult<()> {
        let chat = self
            .state
            .chat_repo
            .find_by_public_id(&chat_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
            .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

        if chat.created_by != user_id.to_string() {
            return Err(GatewayError::AuthorizationFailed(
                "Access denied: only chat creators can delete chats".to_string(),
            ));
        }

        self.state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Owner)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        self.state
            .chat_repo
            .delete(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to delete chat: {}", e)))?;

        Ok(())
    }
}
