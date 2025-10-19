//! Member API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use switchboard_database::{
    ChatMember, UpdateMemberRoleRequest, MemberService, RepositoryError,
    MemberError, MemberResult, ChatRole,
};

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

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListMembersQuery {
    pub role: Option<String>, // Filter by role
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl From<ChatMember> for MemberResponse {
    fn from(member: ChatMember) -> Self {
        Self {
            id: member.public_id,
            user_id: member.user_public_id,
            chat_id: member.chat_public_id,
            role: member.role.to_string(),
            joined_at: member.joined_at.to_rfc3339(),
            user: MemberUserResponse {
                id: member.user_public_id,
                display_name: member.user_display_name,
                avatar_url: member.user_avatar_url,
                email: member.user_email,
            },
        }
    }
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_members(
    Path(chat_id): Path<String>,
    Query(params): Query<ListMembersQuery>,
    State(member_service): State<MemberService<switchboard_database::MemberRepository>>,
    user_id: String,
) -> Result<Json<Vec<MemberResponse>>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    member_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let role_filter = match params.role.as_deref() {
        Some("owner") => Some(ChatRole::Owner),
        Some("admin") => Some(ChatRole::Admin),
        Some("member") => Some(ChatRole::Member),
        _ => None,
    };

    let members = member_service
        .list_by_chat(&chat_id, role_filter, params.limit, params.offset)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Member not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_member(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(member_service): State<MemberService<switchboard_database::MemberRepository>>,
    user_id: String,
) -> Result<Json<MemberResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check chat membership
    member_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let member = member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MEMBER_NOT_FOUND".to_string(),
            message: "Member not found".to_string(),
        })?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(ErrorResponse {
            error: "MEMBER_NOT_IN_CHAT".to_string(),
            message: "Member does not belong to specified chat".to_string(),
        });
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
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Member not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn update_member_role(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(member_service): State<MemberService<switchboard_database::MemberRepository>>,
    Json(payload): Json<UpdateMemberRoleRequest>,
    user_id: String,
) -> Result<Json<MemberResponse>, ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check if user is owner or admin
    member_service
        .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let member = member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MEMBER_NOT_FOUND".to_string(),
            message: "Member not found".to_string(),
        })?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(ErrorResponse {
            error: "MEMBER_NOT_IN_CHAT".to_string(),
            message: "Member does not belong to specified chat".to_string(),
        });
    }

    let new_role = match payload.role.as_str() {
        "owner" => ChatRole::Owner,
        "admin" => ChatRole::Admin,
        "member" => ChatRole::Member,
        _ => return Err(ErrorResponse {
            error: "INVALID_ROLE".to_string(),
            message: "Role must be 'owner', 'admin', or 'member'".to_string(),
        }),
    };

    // Additional checks: Only owners can promote others to owner, and owners cannot demote themselves
    if new_role == ChatRole::Owner {
        member_service
            .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Owner)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    if member.user_public_id == user_internal_id && new_role != ChatRole::Owner {
        return Err(ErrorResponse {
            error: "CANNOT_DEMOTE_SELF".to_string(),
            message: "You cannot demote yourself from owner role".to_string(),
        });
    }

    let update_req = switchboard_database::UpdateMemberRoleRequest {
        role: new_role,
    };

    let updated_member = member_service
        .update_role(member.id, &update_req, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Access denied", body = ErrorResponse),
        (status = 404, description = "Member not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn remove_member(
    Path((chat_id, member_id)): Path<(String, String)>,
    State(member_service): State<MemberService<switchboard_database::MemberRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check if user is owner or admin
    member_service
        .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    let member = member_service
        .get_by_public_id(&member_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?
        .ok_or_else(|| ErrorResponse {
            error: "MEMBER_NOT_FOUND".to_string(),
            message: "Member not found".to_string(),
        })?;

    // Verify member belongs to the specified chat
    if member.chat_public_id != chat_id {
        return Err(ErrorResponse {
            error: "MEMBER_NOT_IN_CHAT".to_string(),
            message: "Member does not belong to specified chat".to_string(),
        });
    }

    // Cannot remove the last owner
    if member.role == ChatRole::Owner {
        return Err(ErrorResponse {
            error: "CANNOT_REMOVE_OWNER".to_string(),
            message: "Cannot remove the last owner from the chat".to_string(),
        });
    }

    // Users can remove themselves, or admins/owners can remove others
    if member.user_public_id != user_internal_id {
        member_service
            .check_chat_role(&chat_id, user_internal_id, switchboard_database::ChatRole::Admin)
            .await
            .map_err(|e| ErrorResponse::from(&e))?;
    }

    member_service
        .remove(member.id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
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
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Cannot leave chat", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn leave_chat(
    Path(chat_id): Path<String>,
    State(member_service): State<MemberService<switchboard_database::MemberRepository>>,
    user_id: String,
) -> Result<(), ErrorResponse> {
    let user_internal_id = user_id.parse::<i64>()
        .map_err(|_| ErrorResponse {
            error: "INVALID_USER_ID".to_string(),
            message: "Invalid user ID format".to_string(),
        })?;

    // Check if user is a member
    member_service
        .check_chat_membership(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    // Get user's membership
    let members = member_service
        .list_by_chat(&chat_id, Some(ChatRole::Owner), None, None)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    // Check if user is the last owner
    let is_owner = members.iter().any(|m| m.user_public_id == user_internal_id && m.role == ChatRole::Owner);
    if is_owner && members.len() == 1 {
        return Err(ErrorResponse {
            error: "CANNOT_LEAVE_AS_LAST_OWNER".to_string(),
            message: "Cannot leave chat as the last owner".to_string(),
        });
    }

    member_service
        .remove_by_user_chat(&chat_id, user_internal_id)
        .await
        .map_err(|e| ErrorResponse::from(&e))?;

    Ok(())
}

impl From<&MemberError> for ErrorResponse {
    fn from(error: &MemberError) -> Self {
        match error {
            MemberError::NotFound => Self {
                error: "MEMBER_NOT_FOUND".to_string(),
                message: "Member not found".to_string(),
            },
            MemberError::AccessDenied => Self {
                error: "ACCESS_DENIED".to_string(),
                message: "Access denied".to_string(),
            },
            MemberError::InvalidInput(msg) => Self {
                error: "INVALID_INPUT".to_string(),
                message: format!("Invalid input: {}", msg),
            },
            MemberError::CannotRemoveLastOwner => Self {
                error: "CANNOT_REMOVE_LAST_OWNER".to_string(),
                message: "Cannot remove the last owner from the chat".to_string(),
            },
            MemberError::CannotDemoteSelf => Self {
                error: "CANNOT_DEMOTE_SELF".to_string(),
                message: "You cannot demote yourself from owner role".to_string(),
            },
            MemberError::RepositoryError(_) => Self {
                error: "INTERNAL_ERROR".to_string(),
                message: "Internal server error".to_string(),
            },
            MemberError::DatabaseError(msg) => Self {
                error: "DATABASE_ERROR".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}