//! Attachment API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use switchboard_database::{
    MessageAttachment, CreateAttachmentRequest, AttachmentService, RepositoryError,
    AttachmentError, AttachmentResult, AttachmentType,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub id: String,
    pub message_id: String,
    pub chat_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
    pub uploader: AttachmentUploaderResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentUploaderResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAttachmentRequest {
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_data: String, // Base64 encoded file data
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListAttachmentsQuery {
    pub message_id: Option<String>, // Filter by message
    pub file_type: Option<String>, // Filter by file type
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<MessageAttachment> for AttachmentResponse {
    fn from(attachment: MessageAttachment) -> Self {
        Self {
            id: attachment.public_id,
            message_id: attachment.message_public_id,
            chat_id: attachment.chat_public_id,
            file_name: attachment.file_name,
            file_type: attachment.file_type.to_string(),
            file_size: attachment.file_size,
            file_url: attachment.file_url,
            created_at: attachment.created_at.to_rfc3339(),
            uploader: AttachmentUploaderResponse {
                id: attachment.uploader_public_id,
                display_name: attachment.uploader_display_name,
                avatar_url: attachment.uploader_avatar_url,
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListAttachmentsQuery
    ),
    responses(
        (status = 200, description = "List of chat attachments", body = Vec<AttachmentResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_attachments(
    Path(chat_id): Path<String>,
    Query(params): Query<ListAttachmentsQuery>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    user_id: String,
) -> Result<Json<Vec<AttachmentResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    attachment_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let file_type_filter = match params.file_type.as_deref() {
        Some("image") => Some(AttachmentType::Image),
        Some("video") => Some(AttachmentType::Video),
        Some("audio") => Some(AttachmentType::Audio),
        Some("document") => Some(AttachmentType::Document),
        Some("other") => Some(AttachmentType::Other),
        _ => None,
    };

    let attachments = attachment_service
        .list_by_chat(&chat_id, params.message_id.as_deref(), file_type_filter, params.limit, params.offset)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let attachment_responses: Vec<AttachmentResponse> = attachments.into_iter().map(|attachment| attachment.into()).collect();
    Ok(Json(attachment_responses))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    responses(
        (status = 200, description = "List of message attachments", body = Vec<AttachmentResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_message_attachments(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    user_id: String,
) -> Result<Json<Vec<AttachmentResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    attachment_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let attachments = attachment_service
        .list_by_message(&message_id, None, None)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let attachment_responses: Vec<AttachmentResponse> = attachments.into_iter().map(|attachment| attachment.into()).collect();
    Ok(Json(attachment_responses))
}

#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    request_body = CreateAttachmentRequest,
    responses(
        (status = 201, description = "Attachment created successfully", body = AttachmentResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 413, description = "File too large", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_attachment(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    Json(payload): Json<CreateAttachmentRequest>,
    user_id: String,
) -> Result<Json<AttachmentResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    attachment_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    // Validate file size (max 50MB)
    if payload.file_size > 50 * 1024 * 1024 {
        return Err(ErrorResponse {
            error: "FILE_TOO_LARGE".to_string(),
            message: "File size cannot exceed 50MB".to_string(),
        });
    }

    // Validate file name
    if payload.file_name.is_empty() || payload.file_name.len() > 255 {
        return Err(ErrorResponse {
            error: "INVALID_FILENAME".to_string(),
            message: "File name must be between 1 and 255 characters".to_string(),
        });
    }

    // Determine file type from mime type or extension
    let file_type = determine_file_type(&payload.file_type, &payload.file_name);

    // Generate a unique file URL
    let file_url = format!("/attachments/{}_{}", chrono::Utc::now().timestamp(), payload.file_name);

    let create_req = switchboard_database::CreateAttachmentRequest {
        message_public_id: message_id,
        uploader_public_id: user_id,
        file_name: payload.file_name,
        file_type,
        file_size: payload.file_size,
        file_url: file_url.clone(),
    };

    let attachment = attachment_service
        .create(&create_req)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(AttachmentResponse::from(attachment)))
}

#[utoipa::path(
    get,
    path = "/api/attachments/{attachment_id}",
    tag = "Attachments",
    params(
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 200, description = "Attachment details", body = AttachmentResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Attachment not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_attachment(
    Path(attachment_id): Path<String>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    user_id: String,
) -> Result<Json<AttachmentResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let attachment = attachment_service
        .get_by_public_id(&attachment_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "ATTACHMENT_NOT_FOUND".to_string(),
            message: "Attachment not found".to_string(),
        })?;

    // Check chat membership
    attachment_service
        .check_chat_membership(&attachment.chat_public_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(AttachmentResponse::from(attachment)))
}

#[utoipa::path(
    get,
    path = "/api/attachments/{attachment_id}/download",
    tag = "Attachments",
    params(
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 200, description = "File download", body = Body),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Attachment not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn download_attachment(
    Path(attachment_id): Path<String>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    user_id: String,
) -> Result<Response<Body>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let attachment = attachment_service
        .get_by_public_id(&attachment_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "ATTACHMENT_NOT_FOUND".to_string(),
            message: "Attachment not found".to_string(),
        })?;

    // Check chat membership
    attachment_service
        .check_chat_membership(&attachment.chat_public_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    // In a real implementation, you would serve the actual file here
    // For now, we'll return a placeholder response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, attachment.file_type.to_string())
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", attachment.file_name))
        .header(header::CONTENT_LENGTH, attachment.file_size.to_string())
        .body(Body::from("File content would be here"))
        .map_err(|_| ErrorResponse {
            error: "INTERNAL_ERROR".to_string(),
            message: "Failed to create response".to_string(),
        })?;

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/api/attachments/{attachment_id}",
    tag = "Attachments",
    params(
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 204, description = "Attachment deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Attachment not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_attachment(
    Path(attachment_id): Path<String>,
    State(attachment_service): State<AttachmentService<switchboard_database::AttachmentRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let attachment = attachment_service
        .get_by_public_id(&attachment_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "ATTACHMENT_NOT_FOUND".to_string(),
            message: "Attachment not found".to_string(),
        })?;

    // Check if user can delete (owner of attachment or chat admin/owner)
    if attachment.uploader_public_id != user_id {
        attachment_service
            .check_chat_role(&attachment.chat_public_id, user_internal_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    attachment_service
        .delete(attachment.id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

fn determine_file_type(mime_type: &str, file_name: &str) -> AttachmentType {
    let mime_lower = mime_type.to_lowercase();
    let name_lower = file_name.to_lowercase();

    if mime_lower.starts_with("image/") || name_lower.ends_with(".jpg") || name_lower.ends_with(".jpeg") ||
       name_lower.ends_with(".png") || name_lower.ends_with(".gif") || name_lower.ends_with(".webp") {
        AttachmentType::Image
    } else if mime_lower.starts_with("video/") || name_lower.ends_with(".mp4") || name_lower.ends_with(".avi") ||
              name_lower.ends_with(".mov") || name_lower.ends_with(".mkv") {
        AttachmentType::Video
    } else if mime_lower.starts_with("audio/") || name_lower.ends_with(".mp3") || name_lower.ends_with(".wav") ||
              name_lower.ends_with(".ogg") || name_lower.ends_with(".flac") {
        AttachmentType::Audio
    } else if mime_lower.starts_with("application/pdf") || name_lower.ends_with(".pdf") ||
              name_lower.ends_with(".doc") || name_lower.ends_with(".docx") || name_lower.ends_with(".txt") {
        AttachmentType::Document
    } else {
        AttachmentType::Other
    }
}

impl From<&AttachmentError> for ErrorResponse {
    fn from(error: &AttachmentError) -> Self {
        match error {
            AttachmentError::NotFound => Self {
                error: "ATTACHMENT_NOT_FOUND".to_string(),
                message: "Attachment not found".to_string(),
            },
            AttachmentError::AccessDenied => Self {
                error: "ACCESS_DENIED".to_string(),
                message: "Access denied".to_string(),
            },
            AttachmentError::InvalidInput(msg) => Self {
                error: "INVALID_INPUT".to_string(),
                message: format!("Invalid input: {}", msg),
            },
            AttachmentError::FileTooLarge => Self {
                error: "FILE_TOO_LARGE".to_string(),
                message: "File size exceeds the maximum allowed limit".to_string(),
            },
            AttachmentError::InvalidFileType => Self {
                error: "INVALID_FILE_TYPE".to_string(),
                message: "File type is not allowed".to_string(),
            },
            AttachmentError::RepositoryError(_) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: "Internal server error".to_string(),
            },
            AttachmentError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}