//! Member REST endpoints

use axum::{
    extract::{Path, Query, State, Request},
    Json,
    response::IntoResponse,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::state::GatewayState;
use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;

#[derive(Debug, Serialize, ToSchema)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub chat_id: String,
    pub role: String,
    pub joined_at: String,
    pub user: MemberUserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MemberUserResponse {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: String, // "member", "admin", "owner"
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListMembersQuery {
    pub role: Option<String>, // Filter by role
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<switchboard_database::ChatMember> for MemberResponse {
    fn from(member: switchboard_database::ChatMember) -> Self {
        Self {
            id: member.public_id,
            user_id: member.user_public_id.clone(),
            chat_id: member.chat_public_id,
            role: member.role.to_string(),
            joined_at: member.joined_at,
            user: MemberUserResponse {
                id: member.user_public_id,
                display_name: member.user_display_name,
                avatar_url: member.user_avatar_url,
                email: member.user_email,
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Create member routes
pub fn create_member_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/chats/:chat_id/members", axum::routing::get(list_members))
        .route("/chats/:chat_id/members/:member_id", axum::routing::get(get_member).delete(remove_member))
        .route("/chats/:chat_id/members/:member_id/role", axum::routing::put(update_member_role))
        .route("/chats/:chat_id/leave", axum::routing::post(leave_chat))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/members",
    tag = "Members",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ListMembersQuery
    ),
    responses(
        (status = 200, description = "List of chat members", body = Vec<MemberResponse>),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn list_members(
    Path(chat_id): Path<String>,
    Query(params): Query<ListMembersQuery>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<Vec<MemberResponse>>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .member_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let role_filter = match params.role.as_deref() {
        Some("owner") => Some(switchboard_database::MemberRole::Owner),
        Some("admin") => Some(switchboard_database::MemberRole::Admin),
        Some("member") => Some(switchboard_database::MemberRole::Member),
        _ => None,
    };

    let members = state
        .member_service
        .list_by_chat(&chat_id, role_filter, params.limit, params.offset)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to list members: {}", e)))?;

    let member_responses: Vec<MemberResponse> = members.into_iter().map(|member| member.into()).collect();
    Ok(Json(member_responses))
}

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/members/{member_id}",
    tag = "Members",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("member_id" = String, Path, description = "Member public ID")
    ),
    responses(
        (status = 200, description = "Member details", body = MemberResponse),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Member not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn get_member(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<MemberResponse>> {
    let user_id = extract_user_id(&request)?;

    // Check chat membership
    state
        .member_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let member = state
        .member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
        .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(GatewayError::NotFound("Member does not belong to specified chat".to_string()));
    }

    Ok(Json(MemberResponse::from(member)))
}

#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}/members/{member_id}/role",
    tag = "Members",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("member_id" = String, Path, description = "Member public ID")
    ),
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Member role updated successfully", body = MemberResponse),
        (status = 400, description = "Invalid request", body = GatewayError),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Member not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn update_member_role(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    Json(payload): Json<UpdateMemberRoleRequest>,
    request: Request,
) -> GatewayResult<Json<MemberResponse>> {
    let user_id = extract_user_id(&request)?;

    // Check if user is owner or admin
    state
        .member_service
        .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let member = state
        .member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
        .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(GatewayError::NotFound("Member does not belong to specified chat".to_string()));
    }

    let new_role = match payload.role.as_str() {
        "owner" => switchboard_database::MemberRole::Owner,
        "admin" => switchboard_database::MemberRole::Admin,
        "member" => switchboard_database::MemberRole::Member,
        _ => return Err(GatewayError::InvalidRequest("Role must be 'owner', 'admin', or 'member'".to_string())),
    };

    // Additional checks: Only owners can promote others to owner, and owners cannot demote themselves
    if new_role == switchboard_database::MemberRole::Owner {
        state
            .member_service
            .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Owner)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    if member.user_public_id == user_id.to_string() && new_role != switchboard_database::MemberRole::Owner {
        return Err(GatewayError::InvalidRequest("You cannot demote yourself from owner role".to_string()));
    }

    let update_req = switchboard_database::UpdateMemberRoleRequest {
        role: new_role,
    };

    let updated_member = state
        .member_service
        .update_role(member.id, &update_req, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update member role: {}", e)))?;

    Ok(Json(MemberResponse::from(updated_member)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/members/{member_id}",
    tag = "Members",
    params(
        ("chat_id" = String, Path, description = "Chat public ID"),
        ("member_id" = String, Path, description = "Member public ID")
    ),
    responses(
        (status = 204, description = "Member removed successfully"),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Access denied", body = GatewayError),
        (status = 404, description = "Member not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn remove_member(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    // Check if user is owner or admin
    state
        .member_service
        .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    let member = state
        .member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get member: {}", e)))?
        .ok_or(GatewayError::NotFound("Member not found".to_string()))?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(GatewayError::NotFound("Member does not belong to specified chat".to_string()));
    }

    // Cannot remove the last owner
    if member.role == switchboard_database::MemberRole::Owner {
        return Err(GatewayError::InvalidRequest("Cannot remove the last owner from the chat".to_string()));
    }

    // Users can remove themselves, or admins/owners can remove others
    if member.user_public_id != user_id.to_string() {
        state
            .member_service
            .check_chat_role(&chat_id, user_id, switchboard_database::MemberRole::Admin)
            .await
            .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;
    }

    state
        .member_service
        .remove(member.id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to remove member: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/leave",
    tag = "Members",
    params(
        ("chat_id" = String, Path, description = "Chat public ID")
    ),
    responses(
        (status = 204, description = "Left chat successfully"),
        (status = 401, description = "Unauthorized", body = GatewayError),
        (status = 403, description = "Cannot leave chat", body = GatewayError),
        (status = 404, description = "Chat not found", body = GatewayError),
        (status = 500, description = "Internal server error", body = GatewayError)
    )
)]
pub async fn leave_chat(
    Path(chat_id): Path<String>,
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<impl IntoResponse> {
    let user_id = extract_user_id(&request)?;

    // Check if user is a member
    state
        .member_service
        .check_chat_membership(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::AuthorizationFailed(format!("Access denied: {}", e)))?;

    // Get user's membership
    let members = state
        .member_service
        .list_by_chat(&chat_id, Some(switchboard_database::MemberRole::Owner), None, None)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to check membership: {}", e)))?;

    // Check if user is the last owner
    let is_owner = members.iter().any(|m| m.user_public_id == user_id.to_string() && m.role == switchboard_database::MemberRole::Owner);
    if is_owner && members.len() == 1 {
        return Err(GatewayError::InvalidRequest("Cannot leave chat as the last owner".to_string()));
    }

    state
        .member_service
        .remove_by_user_chat(&chat_id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to leave chat: {}", e)))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
