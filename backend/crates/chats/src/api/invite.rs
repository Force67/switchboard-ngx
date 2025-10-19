//! Invite API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use switchboard_database::{
    ChatInvite, CreateInviteRequest, InviteService, RepositoryError,
    InviteError, InviteResult, InviteStatus,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub id: String,
    pub chat_id: String,
    pub chat_title: String,
    pub invited_by: String,
    pub invited_email: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
    pub inviter: InviteInviterResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteInviterResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: Option<String>, // Will default to 'member'
    pub expires_in_hours: Option<i64>, // Optional custom expiration
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListInvitesQuery {
    pub status: Option<String>, // Filter by status: pending, accepted, rejected
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RespondToInviteRequest {
    pub action: String, // "accept" or "reject"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<ChatInvite> for InviteResponse {
    fn from(invite: ChatInvite) -> Self {
        Self {
            id: invite.public_id,
            chat_id: invite.chat_public_id,
            chat_title: invite.chat_title,
            invited_by: invite.invited_by_public_id,
            invited_email: invite.invited_email,
            status: invite.status.to_string(),
            created_at: invite.created_at.to_rfc3339(),
            expires_at: invite.expires_at.to_rfc3339(),
            accepted_at: invite.accepted_at.map(|dt| dt.to_rfc3339()),
            inviter: InviteInviterResponse {
                id: invite.invited_by_public_id,
                display_name: invite.inviter_display_name,
                avatar_url: invite.inviter_avatar_url,
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/invites",
    tag = "Invites",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListInvitesQuery
    ),
    responses(
        (status = 200, description = "List of chat invites", body = Vec<InviteResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_invites(
    Path(chat_id): Path<String>,
    Query(params): Query<ListInvitesQuery>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    user_id: String,
) -> Result<Json<Vec<InviteResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check if user is owner or admin
    invite_service
        .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let status_filter = match params.status.as_deref() {
        Some("pending") => Some(InviteStatus::Pending),
        Some("accepted") => Some(InviteStatus::Accepted),
        Some("rejected") => Some(InviteStatus::Rejected),
        Some("expired") => Some(InviteStatus::Expired),
        _ => None,
    };

    let invites = invite_service
        .list_by_chat(&chat_id, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let invite_responses: Vec<InviteResponse> = invites.into_iter().map(|invite| invite.into()).collect();
    Ok(Json(invite_responses))
}

#[utoipa::path(
    get,
    path = "/api/invites",
    tag = "Invites",
    params(ListInvitesQuery),
    responses(
        (status = 200, description = "List of user's invites", body = Vec<InviteResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_user_invites(
    Query(params): Query<ListInvitesQuery>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    user_id: String,
) -> Result<Json<Vec<InviteResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let status_filter = match params.status.as_deref() {
        Some("pending") => Some(InviteStatus::Pending),
        Some("accepted") => Some(InviteStatus::Accepted),
        Some("rejected") => Some(InviteStatus::Rejected),
        Some("expired") => Some(InviteStatus::Expired),
        _ => None,
    };

    let invites = invite_service
        .list_by_user(user_internal_id, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let invite_responses: Vec<InviteResponse> = invites.into_iter().map(|invite| invite.into()).collect();
    Ok(Json(invite_responses))
}

#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/invites",
    tag = "Invites",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite created successfully", body = InviteResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_invite(
    Path(chat_id): Path<String>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    Json(payload): Json<CreateInviteRequest>,
    user_id: String,
) -> Result<Json<InviteResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check if user is owner or admin
    invite_service
        .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    // Validate email format
    if !payload.email.contains('@') || payload.email.len() > 255 {
        return Err(ErrorResponse {
            error: "INVALID_EMAIL".to_string(),
            message: "Invalid email format".to_string(),
        });
    }

    let expires_in_hours = payload.expires_in_hours.unwrap_or(24 * 7); // Default 7 days
    if expires_in_hours <= 0 || expires_in_hours > 24 * 30 { // Max 30 days
        return Err(ErrorResponse {
            error: "INVALID_EXPIRATION".to_string(),
            message: "Expiration must be between 1 hour and 30 days".to_string(),
        });
    }

    let create_req = switchboard_database::CreateInviteRequest {
        chat_public_id: chat_id,
        invited_by_public_id: user_id,
        invited_email: payload.email,
        expires_in_hours,
    };

    let invite = invite_service
        .create(&create_req)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(Json(InviteResponse::from(invite)))
}

#[utoipa::path(
    get,
    path = "/api/invites/{invite_id}",
    tag = "Invites",
    params(
        ("invite_id" = String, Path, description = "Invite public ID")
    ),
    responses(
        (status = 200, description = "Invite details", body = InviteResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Invite not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_invite(
    Path(invite_id): Path<String>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    user_id: String,
) -> Result<Json<InviteResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let invite = invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "INVITE_NOT_FOUND".to_string(),
            message: "Invite not found".to_string(),
        })?;

    // Check if user is either the inviter or the invited user (by email)
    if invite.invited_by_public_id != user_id {
        // Check if user's email matches the invited email
        // This would require looking up the user, which is outside the scope of the invite service
        // For now, we'll only allow the inviter to view the invite
        return Err(ErrorResponse {
            error: "ACCESS_DENIED".to_string(),
            message: "Access denied".to_string(),
        });
    }

    Ok(Json(InviteResponse::from(invite)))
}

#[utoipa::path(
    post,
    path = "/api/invites/{invite_id}/respond",
    tag = "Invites",
    params(
        ("invite_id" = String, Path, description = "Invite public ID")
    ),
    request_body = RespondToInviteRequest,
    responses(
        (status = 200, description = "Invite response processed", body = InviteResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Invite not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn respond_to_invite(
    Path(invite_id): Path<String>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    Json(payload): Json<RespondToInviteRequest>,
    user_id: String,
) -> Result<Json<InviteResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let invite = invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "INVITE_NOT_FOUND".to_string(),
            message: "Invite not found".to_string(),
        })?;

    match payload.action.as_str() {
        "accept" => {
            let updated_invite = invite_service
                .accept_invite(invite.id, user_internal_id)
                .await
                .map_err(|e| ErrorResponse::from(&e))?;
            Ok(Json(InviteResponse::from(updated_invite)))
        },
        "reject" => {
            let updated_invite = invite_service
                .reject_invite(invite.id, user_internal_id)
                .await
                .map_err(|e| ErrorResponse::from(&e))?;
            Ok(Json(InviteResponse::from(updated_invite)))
        },
        _ => Err(ErrorResponse {
            error: "INVALID_ACTION".to_string(),
            message: "Action must be 'accept' or 'reject'".to_string(),
        }),
    }
}

#[utoipa::path(
    delete,
    path = "/api/invites/{invite_id}",
    tag = "Invites",
    params(
        ("invite_id" = String, Path, description = "Invite public ID")
    ),
    responses(
        (status = 204, description = "Invite deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Invite not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_invite(
    Path(invite_id): Path<String>,
    State(invite_service): State<InviteService<switchboard_database::InviteRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    let invite = invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "INVITE_NOT_FOUND".to_string(),
            message: "Invite not found".to_string(),
        })?;

    // Only the inviter or a chat owner/admin can delete the invite
    if invite.invited_by_public_id != user_id {
        invite_service
            .check_chat_role(&invite.chat_public_id, user_internal_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    invite_service
        .delete(invite.id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

impl From<&InviteError> for ErrorResponse {
    fn from(error: &InviteError) -> Self {
        match error {
            InviteError::NotFound => Self {
                error: "INVITE_NOT_FOUND".to_string(),
                message: "Invite not found".to_string(),
            },
            InviteError::AccessDenied => Self {
                error: "ACCESS_DENIED".to_string(),
                message: "Access denied".to_string(),
            },
            InviteError::InvalidInput(msg) => Self {
                error: "INVALID_INPUT".to_string(),
                message: format!("Invalid input: {}", msg),
            },
            InviteError::AlreadyExists => Self {
                error: "INVITE_ALREADY_EXISTS".to_string(),
                message: "User already has a pending invite for this chat".to_string(),
            },
            InviteError::AlreadyMember => Self {
                error: "ALREADY_MEMBER".to_string(),
                message: "User is already a member of this chat".to_string(),
            },
            InviteError::Expired => Self {
                error: "INVITE_EXPIRED".to_string(),
                message: "Invite has expired".to_string(),
            },
            InviteError::RepositoryError(_) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: "Internal server error".to_string(),
            },
            InviteError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}