use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use switchboard_auth::UpdateUserProfile;
use utoipa::ToSchema;

use crate::{routes::auth::UserResponse, util::require_bearer, ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserProfileResponse {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserProfileRequest {
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub display_name: Option<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub username: Option<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub bio: Option<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub avatar_url: Option<Option<String>>,
}

impl UpdateUserProfileRequest {
    fn into_update(self) -> UpdateUserProfile {
        UpdateUserProfile {
            username: self.username,
            display_name: self.display_name,
            bio: self.bio,
            avatar_url: self.avatar_url,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "Users",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = UserProfileResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    Ok(Json(UserProfileResponse { user: user.into() }))
}

#[utoipa::path(
    patch,
    path = "/api/users/me",
    tag = "Users",
    security(("bearerAuth" = [])),
    request_body = UpdateUserProfileRequest,
    responses(
        (status = 200, description = "Updated user profile", body = UserProfileResponse),
        (status = 400, description = "Invalid profile payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse)
    )
)]
pub async fn update_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateUserProfileRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;
    let update = payload.into_update();
    let updated = state
        .authenticator()
        .update_user_profile(user.id, update)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(UserProfileResponse {
        user: updated.into(),
    }))
}
