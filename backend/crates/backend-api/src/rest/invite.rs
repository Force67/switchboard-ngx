//! Invite REST endpoints

use axum::{
    extract::{Extension, Path, Query, Request, State},
    response::IntoResponse,
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::rest::models::{
    CreateInviteRequest, InviteResponse, ListInvitesQuery, RespondToInviteRequest,
};
use crate::state::GatewayState;

/// Create invite routes
pub fn create_invite_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/chats/:chat_id/invites", axum::routing::get(list_invites).post(create_invite))
        .route("/invites", axum::routing::get(list_user_invites))
        .route(
            "/invites/:invite_id",
            axum::routing::get(get_invite).delete(delete_invite),
        )
        .route("/invites/:invite_id/respond", axum::routing::post(respond_to_invite))
}

#[utoipa::path(
    get,
    path = "/api/v1/chats/{chat_id}/invites",
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
        .permissions()
        .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Parse status filter
    let status_filter = params.status.as_ref().and_then(|s| {
        match s.to_lowercase().as_str() {
            "pending" => Some(switchboard_database::InviteStatus::Pending),
            "accepted" => Some(switchboard_database::InviteStatus::Accepted),
            "rejected" => Some(switchboard_database::InviteStatus::Rejected),
            "expired" => Some(switchboard_database::InviteStatus::Expired),
            _ => None,
        }
    });

    let invites = state
        .invite_repo
        .list_by_chat_public(&chat_id, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list invites: {}", e)))?;

    let invite_responses: Vec<InviteResponse> = invites.into_iter().map(|i| i.into()).collect();
    Ok(Json(invite_responses))
}

#[utoipa::path(
    get,
    path = "/api/v1/invites",
    tag = "Invites",
    params(ListInvitesQuery),
    responses(
        (status = 200, description = "List of user's pending invites", body = Vec<InviteResponse>),
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

    // Get user to find their email
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    let email = user.email.ok_or(GatewayError::InvalidRequest(
        "User does not have an email address".to_string(),
    ))?;

    // Parse status filter
    let status_filter = params.status.as_ref().and_then(|s| {
        match s.to_lowercase().as_str() {
            "pending" => Some(switchboard_database::InviteStatus::Pending),
            "accepted" => Some(switchboard_database::InviteStatus::Accepted),
            "rejected" => Some(switchboard_database::InviteStatus::Rejected),
            "expired" => Some(switchboard_database::InviteStatus::Expired),
            _ => None,
        }
    });

    let invites = state
        .invite_repo
        .list_by_user_email(&email, status_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list invites: {}", e)))?;

    let invite_responses: Vec<InviteResponse> = invites.into_iter().map(|i| i.into()).collect();
    Ok(Json(invite_responses))
}

#[utoipa::path(
    post,
    path = "/api/v1/chats/{chat_id}/invites",
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
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateInviteRequest>,
) -> GatewayResult<impl IntoResponse> {
    // Check if user is owner or admin
    state
        .permissions()
        .require_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Get chat to resolve ID
    let chat = state
        .chat_repo
        .find_by_public_id(&chat_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get chat: {}", e)))?
        .ok_or(GatewayError::NotFound("Chat not found".to_string()))?;

    // Get user's public_id
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to find user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    let create_req = switchboard_database::CreateInviteRequest {
        chat_id: chat.id,
        chat_public_id: chat_id.clone(),
        invited_by_public_id: user.public_id,
        invited_email: payload.email,
        expires_in_hours: payload.expires_in_hours.unwrap_or(24),
    };

    let invite = state
        .invite_repo
        .create(user_id, &create_req)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to create invite: {}", e)))?;

    let response = InviteResponse::from(invite);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/invites/{invite_id}",
    tag = "Invites",
    params(
        ("invite_id" = String, Path, description = "Invite public ID")
    ),
    responses(
        (status = 200, description = "Invite details", body = InviteResponse),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
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
        .invite_repo
        .find_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    // Get user email to check if they're the invitee
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    let is_invitee = user.email.as_ref().map_or(false, |email| email == &invite.invited_email);

    // User can view invite if they're the invitee or the inviter (or admin of the chat)
    let can_view = invite.inviter_id == user_id
        || is_invitee
        || state
            .permissions()
            .require_role(
                &invite.chat_public_id,
                user_id,
                switchboard_database::MemberRole::Admin,
            )
            .await
            .is_ok();

    if !can_view {
        return Err(GatewayError::AuthorizationFailed(
            "Access denied".to_string(),
        ));
    }

    Ok(Json(InviteResponse::from(invite)))
}

#[utoipa::path(
    post,
    path = "/api/v1/invites/{invite_id}/respond",
    tag = "Invites",
    params(
        ("invite_id" = String, Path, description = "Invite public ID")
    ),
    request_body = RespondToInviteRequest,
    responses(
        (status = 200, description = "Invite response processed", body = InviteResponse),
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Invite not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn respond_to_invite(
    Path(invite_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<RespondToInviteRequest>,
) -> GatewayResult<Json<InviteResponse>> {
    // Get the invite first
    let invite = state
        .invite_repo
        .find_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    // Verify user is the invitee by email
    let user = state
        .user_service
        .find_by_id(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?
        .ok_or(GatewayError::NotFound("User not found".to_string()))?;

    let is_invitee = user.email.as_ref().map_or(false, |email| email == &invite.invited_email);
    if !is_invitee {
        return Err(GatewayError::AuthorizationFailed(
            "Access denied: not the invitee".to_string(),
        ));
    }

    let invite = match payload.action.as_str() {
        "accept" => {
            state
                .invite_repo
                .accept_by_id(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to accept invite: {}", e)))?
        }
        "reject" => {
            state
                .invite_repo
                .decline_by_id(invite.id, user_id)
                .await
                .map_err(|e| GatewayError::ServiceError(format!("Failed to reject invite: {}", e)))?
        }
        _ => {
            return Err(GatewayError::InvalidRequest(
                "Action must be 'accept' or 'reject'".to_string(),
            ))
        }
    };

    Ok(Json(InviteResponse::from(invite)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/invites/{invite_id}",
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
        .invite_repo
        .find_by_public_id(&invite_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get invite: {}", e)))?
        .ok_or(GatewayError::NotFound("Invite not found".to_string()))?;

    // Only the inviter or a chat admin can delete the invite
    if invite.inviter_id != user_id {
        state
            .permissions()
            .require_role(
                &invite.chat_public_id,
                user_id,
                switchboard_database::MemberRole::Admin,
            )
            .await
            .map_err(|_| {
                GatewayError::AuthorizationFailed("Access denied: not authorized".to_string())
            })?;
    }

    state
        .invite_repo
        .delete_by_id(invite.id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete invite: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
