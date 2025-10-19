//! Invite REST endpoints

use axum::{
    extract::{Path, Query, State, Request},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::state::GatewayState;
use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;

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

impl From<switchboard_database::ChatInvite> for InviteResponse {
    fn from(invite: switchboard_database::ChatInvite) -> Self {
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

/// Create invite routes
pub fn create_invite_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/invites", axum::routing::get(list_user_invites))
        .route("/invites/:invite_id", axum::routing::get(get_invite).delete(delete_invite))
        .route("/invites/:invite_id/respond", axum::routing::post(respond_to_invite))
        .route("/chats/:chat_id/invites", axum::routing::get(list_invites).post(create_invite))
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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_invites(
    Path(chat_id): Path<String>,
    Query(params): Query<ListInvitesQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<InviteResponse>>> {
    let user_id = extract_user_id(&request)?;

    // Check if user is owner or admin
    state
        .invite_service
        .check_chat_role(&chat_id, user_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let status_filter = match params.status.as_deref() {
        Some("pending") => Some(switchboard_database::InviteStatus::Pending),
        Some("accepted") => Some(switchboard_database::InviteStatus::Accepted),
        Some("rejected") => Some(switchboard_database::InviteStatus::Rejected),
        Some("expired") => Some(switchboard_database::InviteStatus::Expired),
        _ => None,
    };

    let invites = state
        .invite_service
        .list_by_chat(&chat_id, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list invites: {}", e)))?;

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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_user_invites(
    Query(params): Query<ListInvitesQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<InviteResponse>>> {
    let user_id = extract_user_id(&request)?;

    let status_filter = match params.status.as_deref() {
        Some("pending") => Some(switchboard_database::InviteStatus::Pending),
        Some("accepted") => Some(switchboard_database::InviteStatus::Accepted),
        Some("rejected") => Some(switchboard_database::InviteStatus::Rejected),
        Some("expired") => Some(switchboard_database::InviteStatus::Expired),
        _ => None,
    };

    let invites = state
        .invite_service
        .list_by_user(user_id, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list user invites: {}", e)))?;

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
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn create_invite(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<CreateInviteRequest>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    // Check if user is owner or admin
    state
        .invite_service
        .check_chat_role(&chat_id, user_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Validate email format
    if !payload.email.contains('@') || payload.email.len() > 255 {
        return Err(GatewayError::InvalidRequest("Invalid email format".to_string()));
    }

    let expires_in_hours = payload.expires_in_hours.unwrap_or(24 * 7); // Default 7 days
    if expires_in_hours <= 0 || expires_in_hours > 24 * 30 { // Max 30 days
        return Err(GatewayError::InvalidRequest("Expiration must be between 1 hour and 30 days".to_string()));
    }

    let create_req = switchboard_database::CreateInviteRequest {
        chat_public_id: chat_id,
        invited_by_public_id: user_id.to_string(),
        invited_email: payload.email,
        expires_in_hours,
    };

    let invite = state
        .invite_service
        .create(&create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create invite: {}", e)))?;

    let response = InviteResponse::from(invite);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 404, description = "Invite not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn get_invite(
    Path(invite_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<InviteResponse>> {
    let user_id = extract_user_id(&request)?;

    let invite = state
        .invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    // Check if user is either the inviter or the invited user (by email)
    if invite.invited_by_public_id != user_id.to_string() {
        // Check if user's email matches the invited email
        // This would require looking up the user, which is outside the scope of the invite service
        // For now, we'll only allow the inviter to view the invite
        return Err(GatewayError::AuthorizationFailed("Access denied".to_string()));
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
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 404, description = "Invite not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn respond_to_invite(
    Path(invite_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<RespondToInviteRequest>,
    request: Request,
) -> GatewayResult<Json<InviteResponse>> {
    let user_id = extract_user_id(&request)?;

    let invite = state
        .invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    match payload.action.as_str() {
        "accept" => {
            let updated_invite = state
                .invite_service
                .accept_invite(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to accept invite: {}", e)))?;
            Ok(Json(InviteResponse::from(updated_invite)))
        },
        "reject" => {
            let updated_invite = state
                .invite_service
                .reject_invite(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to reject invite: {}", e)))?;
            Ok(Json(InviteResponse::from(updated_invite)))
        },
        _ => Err(GatewayError::InvalidRequest("Action must be 'accept' or 'reject'".to_string())),
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
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Invite not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn delete_invite(
    Path(invite_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    let invite = state
        .invite_service
        .get_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    // Only the inviter or a chat owner/admin can delete the invite
    if invite.invited_by_public_id != user_id.to_string() {
        state
            .invite_service
            .check_chat_role(&invite.chat_public_id, user_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    state
        .invite_service
        .delete(invite.id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete invite: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}