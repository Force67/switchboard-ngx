use axum::{
    extract::{Path, Query, Request, State},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::state::GatewayState;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
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

// DTOs
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub notification: crate::rest::models::Notification,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationsResponse {
    pub notifications: Vec<crate::rest::models::Notification>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkNotificationReadRequest {
    pub read: bool,
}

// Get user notifications
#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(ListNotificationsQuery),
    responses(
        (status = 200, description = "List notifications for authenticated user", body = NotificationsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch notifications", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_notifications(
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<ListNotificationsQuery>,
    request: Request,
) -> GatewayResult<Json<NotificationsResponse>> {
    let user_id = extract_user_id(&request)?;

    // For now, return empty list as notification service is not fully implemented
    Ok(Json(NotificationsResponse {
        notifications: vec![],
    }))
}

// Get unread count
#[utoipa::path(
    get,
    path = "/api/v1/notifications/unread-count",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Count of unread notifications", body = UnreadCountResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch unread count", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_unread_count(
    State(state): State<Arc<GatewayState>>,
    request: axum::http::Request<()>,
) -> GatewayResult<Json<UnreadCountResponse>> {
    let user_id = extract_user_id(&request)?;

    // For now, return 0 as notification service is not fully implemented
    Ok(Json(UnreadCountResponse { unread_count: 0 }))
}

// Mark notification as read
#[utoipa::path(
    put,
    path = "/api/v1/notifications/{notification_id}",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("notification_id" = i64, Path, description = "Notification ID")
    ),
    request_body = MarkNotificationReadRequest,
    responses(
        (status = 200, description = "Notification marked as read/unread"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_notification_read(
    State(state): State<Arc<GatewayState>>,
    Path(notification_id): Path<i64>,
    Json(payload): Json<MarkNotificationReadRequest>,
    request: axum::http::Request<()>,
) -> GatewayResult<()> {
    let user_id = extract_user_id(&request)?;

    // TODO: Implement actual notification update
    tracing::info!(
        "User {} marking notification {} as read={}",
        user_id,
        notification_id,
        payload.read
    );

    Ok(())
}

// Mark all notifications as read
#[utoipa::path(
    post,
    path = "/api/v1/notifications/mark-all-read",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "All notifications marked as read", body = BulkUpdateResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to mark all as read", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_all_read(
    State(state): State<Arc<GatewayState>>,
    request: axum::http::Request<()>,
) -> GatewayResult<Json<BulkUpdateResponse>> {
    let user_id = extract_user_id(&request)?;

    // TODO: Implement actual bulk update
    tracing::info!("User {} marking all notifications as read", user_id);

    Ok(Json(BulkUpdateResponse { updated_count: 0 }))
}

// Delete notification
#[utoipa::path(
    delete,
    path = "/api/v1/notifications/{notification_id}",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("notification_id" = i64, Path, description = "Notification ID")
    ),
    responses(
        (status = 204, description = "Notification deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_notification(
    State(state): State<Arc<GatewayState>>,
    Path(notification_id): Path<i64>,
    request: axum::http::Request<()>,
) -> GatewayResult<()> {
    let user_id = extract_user_id(&request)?;

    // TODO: Implement actual deletion
    tracing::info!(
        "User {} deleting notification {}",
        user_id,
        notification_id
    );

    Ok(())
}

/// Create notification routes
pub fn create_notification_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/notifications", axum::routing::get(get_notifications))
        .route("/notifications/unread-count", axum::routing::get(get_unread_count))
        .route("/notifications/:notification_id", axum::routing::put(mark_notification_read))
        .route("/notifications/:notification_id", axum::routing::delete(delete_notification))
        .route("/notifications/mark-all-read", axum::routing::post(mark_all_read))
}