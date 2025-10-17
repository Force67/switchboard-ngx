use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    routes::models::{
        MarkNotificationReadRequest, NotificationResponse, NotificationsResponse,
    },
    services::notification as notification_service,
    util::require_bearer,
    ApiError, AppState,
};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnreadCountResponse {
    pub unread_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkUpdateResponse {
    pub updated_count: u64,
}

// Get user notifications
#[utoipa::path(
    get,
    path = "/api/notifications",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(ListNotificationsQuery),
    responses(
        (status = 200, description = "List notifications for the authenticated user", body = NotificationsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch notifications", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<NotificationsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let notifications = notification_service::get_user_notifications(
        state.db_pool(),
        user.id,
        query.unread_only,
        query.limit,
        query.offset,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch notifications: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(NotificationsResponse { notifications }))
}

// Get unread notification count
#[utoipa::path(
    get,
    path = "/api/notifications/unread-count",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Unread notification count", body = UnreadCountResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch unread count", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_unread_count(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UnreadCountResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let count = notification_service::get_user_unread_count(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch unread notification count: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(UnreadCountResponse { unread_count: count }))
}

// Mark notification(s) as read/unread
#[utoipa::path(
    put,
    path = "/api/notifications/{notification_id}",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("notification_id" = i64, Path, description = "Notification identifier")
    ),
    request_body = MarkNotificationReadRequest,
    responses(
        (status = 200, description = "Notification updated", body = NotificationResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_notification_read(
    State(state): State<AppState>,
    Path(notification_id): Path<i64>,
    headers: HeaderMap,
    Json(req): Json<MarkNotificationReadRequest>,
) -> Result<Json<NotificationResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let notification = notification_service::mark_notification_as_read(
        state.db_pool(),
        user.id,
        notification_id,
        req,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update notification: {}", e);
        ApiError::from(e)
    })?;

    Ok(Json(NotificationResponse { notification }))
}

// Mark all notifications as read
#[utoipa::path(
    post,
    path = "/api/notifications/mark-all-read",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Notifications marked as read", body = BulkUpdateResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update notifications", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_all_read(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<BulkUpdateResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let updated_count = notification_service::mark_all_user_notifications_read(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark all notifications as read: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(BulkUpdateResponse { updated_count }))
}

// Delete a notification
#[utoipa::path(
    delete,
    path = "/api/notifications/{notification_id}",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("notification_id" = i64, Path, description = "Notification identifier")
    ),
    responses(
        (status = 200, description = "Notification deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_notification(
    State(state): State<AppState>,
    Path(notification_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    notification_service::delete_user_notification(state.db_pool(), user.id, notification_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete notification: {}", e);
            ApiError::from(e)
        })?;

    Ok(())
}
