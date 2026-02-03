//! Messages service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::MessagesServiceTrait;
use crate::rest::models::{
    CreateMessageRequest, ListMessagesQuery, MessageResponse, MessagesResponse,
    UpdateMessageRequest,
};
use crate::state::GatewayState;

pub struct MessagesServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> MessagesServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl MessagesServiceTrait for MessagesServiceImpl<'_> {
    async fn list_messages(
        &self,
        user_id: i64,
        chat_id: String,
        query: ListMessagesQuery,
    ) -> GatewayResult<MessagesResponse> {
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let chat = self
            .state
            .chat_repo
            .find_by_public_id(&chat_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
            .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

        let messages = self
            .state
            .message_repo
            .find_by_chat_id(chat.id, query.limit, query.offset)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list messages: {}", e)))?;

        let messages: Vec<MessageResponse> = messages.into_iter().map(|msg| msg.into()).collect();
        Ok(MessagesResponse { messages })
    }

    async fn create_message(
        &self,
        user_id: i64,
        chat_id: String,
        req: CreateMessageRequest,
    ) -> GatewayResult<MessageResponse> {
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let message_type = match req.message_type.as_deref() {
            Some("system") => switchboard_database::MessageType::System,
            _ => switchboard_database::MessageType::Text,
        };

        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        let chat = self
            .state
            .chat_repo
            .find_by_public_id(&chat_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
            .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

        let create_req = switchboard_database::CreateMessageRequest {
            chat_id: chat.id,
            chat_public_id: chat_id.clone(),
            sender_id: user_id,
            sender_public_id: user.public_id,
            content: req.content.clone(),
            message_type,
            reply_to_public_id: req.reply_to.clone(),
            thread_public_id: req.thread_id.clone(),
        };

        let message = self
            .state
            .message_repo
            .create(user_id, &create_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to create message: {}", e)))?;

        Ok(MessageResponse::from(message))
    }

    async fn get_message(
        &self,
        user_id: i64,
        chat_id: String,
        message_id: String,
    ) -> GatewayResult<MessageResponse> {
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let message = self
            .state
            .message_repo
            .find_by_public_id(&message_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
            .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

        if message.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Message does not belong to specified chat".to_string(),
            ));
        }

        Ok(MessageResponse::from(message))
    }

    async fn update_message(
        &self,
        user_id: i64,
        chat_id: String,
        message_id: String,
        req: UpdateMessageRequest,
    ) -> GatewayResult<MessageResponse> {
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let message = self
            .state
            .message_repo
            .find_by_public_id(&message_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
            .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

        if message.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Message does not belong to specified chat".to_string(),
            ));
        }

        let update_req = switchboard_database::UpdateMessageRequest {
            content: req.content.clone(),
            status: None,
        };

        let updated_message = self
            .state
            .message_repo
            .update(&message_id, user_id, &update_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to update message: {}", e)))?;

        Ok(MessageResponse::from(updated_message))
    }

    async fn delete_message(
        &self,
        user_id: i64,
        chat_id: String,
        message_id: String,
    ) -> GatewayResult<()> {
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let message = self
            .state
            .message_repo
            .find_by_public_id(&message_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
            .ok_or(GatewayError::NotFound("Message not found".to_string()))?;

        if message.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Message does not belong to specified chat".to_string(),
            ));
        }

        if message.sender_id != user_id {
            self.state
                .permissions()
                .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
                .await
                .map_err(|_| {
                    GatewayError::AuthorizationFailed(
                        "Cannot delete another user's message without admin permissions".to_string(),
                    )
                })?;
        }

        self.state
            .message_repo
            .delete(&message_id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to delete message: {}", e)))?;

        Ok(())
    }
}
