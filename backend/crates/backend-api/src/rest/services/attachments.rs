//! Attachments service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::AttachmentsServiceTrait;
use crate::rest::models::{
    AttachmentResponse, AttachmentsResponse, CreateAttachmentRequest, ListAttachmentsQuery,
};
use crate::state::GatewayState;

const MAX_ATTACHMENT_BYTES: i64 = 20 * 1024 * 1024;
const MAX_ATTACHMENT_B64_LEN: usize = 28 * 1024 * 1024;
const MAX_ATTACHMENT_NAME_LEN: usize = 255;

/// Implementation of AttachmentsServiceTrait
pub struct AttachmentsServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> AttachmentsServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl AttachmentsServiceTrait for AttachmentsServiceImpl<'_> {
    async fn list_attachments(
        &self,
        user_id: i64,
        chat_id: String,
        query: ListAttachmentsQuery,
    ) -> GatewayResult<AttachmentsResponse> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        // Parse file type filter if provided
        let file_type_filter = query.file_type.as_ref().map(|ft| {
            match ft.to_lowercase().as_str() {
                "image" => switchboard_database::AttachmentType::Image,
                "document" => switchboard_database::AttachmentType::Document,
                "video" => switchboard_database::AttachmentType::Video,
                "audio" => switchboard_database::AttachmentType::Audio,
                _ => switchboard_database::AttachmentType::Other,
            }
        });

        let attachments = self
            .state
            .attachment_repo
            .list_by_chat_public(
                &chat_id,
                query.message_id.as_deref(),
                file_type_filter,
                query.limit,
                query.offset,
            )
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list attachments: {}", e)))?;

        let attachments: Vec<AttachmentResponse> = attachments.into_iter().map(|a| a.into()).collect();
        Ok(AttachmentsResponse { attachments })
    }

    async fn list_message_attachments(
        &self,
        user_id: i64,
        chat_id: String,
        message_id: String,
    ) -> GatewayResult<AttachmentsResponse> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let attachments = self
            .state
            .attachment_repo
            .list_by_message_public(&message_id, None, None)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to list attachments: {}", e)))?;

        let attachments: Vec<AttachmentResponse> = attachments.into_iter().map(|a| a.into()).collect();
        Ok(AttachmentsResponse { attachments })
    }

    async fn create_attachment(
        &self,
        user_id: i64,
        chat_id: String,
        message_id: String,
        req: CreateAttachmentRequest,
    ) -> GatewayResult<AttachmentResponse> {
        if req.file_size <= 0 || req.file_size > MAX_ATTACHMENT_BYTES {
            return Err(GatewayError::InvalidRequest(format!(
                "invalid file_size (max {} bytes)",
                MAX_ATTACHMENT_BYTES
            )));
        }

        if req.file_name.trim().is_empty() || req.file_name.len() > MAX_ATTACHMENT_NAME_LEN {
            return Err(GatewayError::InvalidRequest(format!(
                "invalid file_name (max {} chars)",
                MAX_ATTACHMENT_NAME_LEN
            )));
        }

        if req.file_data.trim().is_empty() || req.file_data.len() > MAX_ATTACHMENT_B64_LEN {
            return Err(GatewayError::InvalidRequest(format!(
                "invalid file_data (max {} chars)",
                MAX_ATTACHMENT_B64_LEN
            )));
        }

        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        // Get message to resolve IDs
        let message = self
            .state
            .message_repo
            .find_by_public_id(&message_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get message: {}", e)))?
            .ok_or_else(|| GatewayError::NotFound("Message not found".to_string()))?;

        // Verify message belongs to the chat
        if message.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Message does not belong to specified chat".to_string(),
            ));
        }

        // Get user's public_id
        let user = self
            .state
            .user_service
            .find_by_id(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
            .ok_or(GatewayError::NotFound("User not found".to_string()))?;

        // Parse file type
        let file_type = match req.file_type.to_lowercase().as_str() {
            "image" | "image/png" | "image/jpeg" | "image/gif" => {
                switchboard_database::AttachmentType::Image
            }
            "document" | "application/pdf" => switchboard_database::AttachmentType::Document,
            "video" | "video/mp4" => switchboard_database::AttachmentType::Video,
            "audio" | "audio/mp3" | "audio/wav" => switchboard_database::AttachmentType::Audio,
            _ => switchboard_database::AttachmentType::Other,
        };

        // TODO: In a real implementation, decode base64 file_data and store the file,
        // then set file_url to the storage location. For now, we use a placeholder URL.
        let file_url = format!("data:attachment/{}", cuid2::cuid());

        let create_req = switchboard_database::CreateAttachmentRequest {
            message_id: message.id,
            message_public_id: message_id.clone(),
            uploader_id: user_id,
            uploader_public_id: user.public_id,
            file_name: req.file_name,
            file_type,
            file_size: req.file_size,
            file_url,
        };

        let attachment = self
            .state
            .attachment_repo
            .create(&create_req)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to create attachment: {}", e)))?;

        Ok(AttachmentResponse::from(attachment))
    }

    async fn get_attachment(
        &self,
        user_id: i64,
        chat_id: String,
        attachment_id: String,
    ) -> GatewayResult<AttachmentResponse> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let attachment = self
            .state
            .attachment_repo
            .find_by_public_id(&attachment_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get attachment: {}", e)))?
            .ok_or(GatewayError::NotFound("Attachment not found".to_string()))?;

        // Verify attachment belongs to the specified chat
        if attachment.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Attachment does not belong to specified chat".to_string(),
            ));
        }

        Ok(AttachmentResponse::from(attachment))
    }

    async fn delete_attachment(
        &self,
        user_id: i64,
        chat_id: String,
        attachment_id: String,
    ) -> GatewayResult<()> {
        // Check chat membership
        self.state
            .permissions()
            .require_membership(&chat_id, user_id)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

        let attachment = self
            .state
            .attachment_repo
            .find_by_public_id(&attachment_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to get attachment: {}", e)))?
            .ok_or(GatewayError::NotFound("Attachment not found".to_string()))?;

        // Verify attachment belongs to the specified chat
        if attachment.chat_public_id != chat_id {
            return Err(GatewayError::NotFound(
                "Attachment does not belong to specified chat".to_string(),
            ));
        }

        // Verify user is the uploader or an admin
        if attachment.uploader_id != user_id {
            self.state
                .permissions()
                .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
                .await
                .map_err(|_| {
                    GatewayError::AuthorizationFailed(
                        "Cannot delete another user's attachment without admin permissions".to_string(),
                    )
                })?;
        }

        self.state
            .attachment_repo
            .delete(&attachment_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to delete attachment: {}", e)))?;

        Ok(())
    }
}
