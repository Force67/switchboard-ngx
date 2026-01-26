//! Attachment REST endpoints

use axum::{
    body::Body,
    extract::{Extension, Path, Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::rest::models::{AttachmentResponse, CreateAttachmentRequest, ListAttachmentsQuery};
use crate::state::GatewayState;

const MAX_ATTACHMENT_BYTES: i64 = 20 * 1024 * 1024;
const MAX_ATTACHMENT_B64_LEN: usize = 28 * 1024 * 1024;
const MAX_ATTACHMENT_NAME_LEN: usize = 255;

/// Create attachment routes
pub fn create_attachment_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route(
            "/chats/:chat_id/attachments",
            axum::routing::get(list_attachments),
        )
        .route(
            "/chats/:chat_id/messages/:message_id/attachments",
            axum::routing::get(list_message_attachments).post(create_attachment),
        )
        .route(
            "/chats/:chat_id/attachments/:attachment_id",
            axum::routing::get(get_attachment).delete(delete_attachment),
        )
        .route(
            "/chats/:chat_id/attachments/:attachment_id/download",
            axum::routing::get(download_attachment),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListAttachmentsQuery
    ),
    responses(
        (status = 200, description = "List of attachments", body = Vec<AttachmentResponse>),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_attachments(
    Path(chat_id): Path<String>,
    Query(params): Query<ListAttachmentsQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<AttachmentResponse>>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Parse file type filter if provided
    let file_type_filter = params.file_type.as_ref().map(|ft| {
        match ft.to_lowercase().as_str() {
            "image" => switchboard_database::AttachmentType::Image,
            "document" => switchboard_database::AttachmentType::Document,
            "video" => switchboard_database::AttachmentType::Video,
            "audio" => switchboard_database::AttachmentType::Audio,
            _ => switchboard_database::AttachmentType::Other,
        }
    });

    let attachments = state
        .attachment_repo
        .list_by_chat_public(&chat_id, params.message_id.as_deref(), file_type_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list attachments: {}", e)))?;

    let attachment_responses: Vec<AttachmentResponse> =
        attachments.into_iter().map(|a| a.into()).collect();
    Ok(Json(attachment_responses))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    responses(
        (status = 200, description = "List of message attachments", body = Vec<AttachmentResponse>),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat or message not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_message_attachments(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<AttachmentResponse>>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let attachments = state
        .attachment_repo
        .list_by_message_public(&message_id, None, None)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list attachments: {}", e)))?;

    let attachment_responses: Vec<AttachmentResponse> =
        attachments.into_iter().map(|a| a.into()).collect();
    Ok(Json(attachment_responses))
}

#[utoipa::path(
    post,
    path = "/api/v1/chats/{chat_id}/messages/{message_id}/attachments",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("message_id" = String, Path, description = "Message public ID")
    ),
    request_body = CreateAttachmentRequest,
    responses(
        (status = 201, description = "Attachment created successfully", body = AttachmentResponse),
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat or message not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn create_attachment(
    Path((chat_id, message_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateAttachmentRequest>,
) -> GatewayResult<impl IntoResponse> {
    if payload.file_size <= 0 || payload.file_size > MAX_ATTACHMENT_BYTES {
        return Err(GatewayError::InvalidRequest(format!(
            "invalid file_size (max {} bytes)",
            MAX_ATTACHMENT_BYTES
        )));
    }

    if payload.file_name.trim().is_empty() || payload.file_name.len() > MAX_ATTACHMENT_NAME_LEN {
        return Err(GatewayError::InvalidRequest(format!(
            "invalid file_name (max {} chars)",
            MAX_ATTACHMENT_NAME_LEN
        )));
    }

    if payload.file_data.trim().is_empty() || payload.file_data.len() > MAX_ATTACHMENT_B64_LEN {
        return Err(GatewayError::InvalidRequest(format!(
            "invalid file_data (max {} chars)",
            MAX_ATTACHMENT_B64_LEN
        )));
    }

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Get message to resolve IDs
    let message = state
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
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    // Parse file type
    let file_type = match payload.file_type.to_lowercase().as_str() {
        "image" | "image/png" | "image/jpeg" | "image/gif" => switchboard_database::AttachmentType::Image,
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
        file_name: payload.file_name,
        file_type,
        file_size: payload.file_size,
        file_url,
    };

    let attachment = state
        .attachment_repo
        .create(&create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create attachment: {}", e)))?;

    let response = AttachmentResponse::from(attachment);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/attachments/{attachment_id}",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 200, description = "Attachment details", body = AttachmentResponse),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Attachment not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn get_attachment(
    Path((chat_id, attachment_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<AttachmentResponse>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let attachment = state
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

    Ok(Json(AttachmentResponse::from(attachment)))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/attachments/{attachment_id}/download",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 200, description = "File download", content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Attachment not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn download_attachment(
    Path((chat_id, attachment_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Response> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let attachment = state
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

    // In a real implementation, this would fetch the file from storage
    // For now, we return a redirect to the file URL, but restrict it to http(s) to avoid
    // open redirects to unsafe schemes.
    let location = attachment.file_url.trim();
    if !(location.starts_with("https://") || location.starts_with("http://")) {
        return Err(GatewayError::ServiceUnavailable);
    }

    let location = HeaderValue::from_str(location)
        .map_err(|_| GatewayError::ServiceUnavailable)?;

    let content_disposition = build_content_disposition(&attachment.file_name);

    let response = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, location)
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Body::empty())
        .map_err(|e| GatewayError::InternalError(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

fn build_content_disposition(file_name: &str) -> HeaderValue {
    let file_name = sanitize_filename(file_name);
    let value = format!("attachment; filename=\"{}\"", file_name);
    HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

fn sanitize_filename(file_name: &str) -> String {
    let trimmed = file_name.trim();
    let mut out = String::with_capacity(trimmed.len().min(128));

    for ch in trimmed.chars().take(128) {
        match ch {
            // Prevent header injection and quoting issues.
            '\r' | '\n' | '"' | '\\' => out.push('_'),
            // Strip other ASCII control chars.
            ch if ch.is_ascii_control() => out.push('_'),
            ch => out.push(ch),
        }
    }

    let out = out.trim();
    if out.is_empty() {
        "attachment".to_string()
    } else {
        out.to_string()
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/chats/{chat_id}/attachments/{attachment_id}",
    tag = "Attachments",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("attachment_id" = String, Path, description = "Attachment public ID")
    ),
    responses(
        (status = 204, description = "Attachment deleted successfully"),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Attachment not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn delete_attachment(
    Path((chat_id, attachment_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .permissions()
        .require_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let attachment = state
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
        state
            .permissions()
            .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|_| {
                GatewayError::AuthorizationFailed(
                    "Cannot delete another user's attachment without admin permissions".to_string(),
                )
            })?;
    }

    state
        .attachment_repo
        .delete(&attachment_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete attachment: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
