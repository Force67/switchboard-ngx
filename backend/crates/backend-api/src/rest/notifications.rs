//! Notification REST endpoints

use axum::{
    extract::{Extension, Path, Query, Request, State},
    Json, Router,
};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::middleware::extract_user_id;
use crate::rest::models::{
    to_api_notification, BulkUpdateResponse, ListNotificationsQuery, MarkNotificationReadRequest,
    Notification, NotificationResponse, NotificationsResponse, UnreadCountResponse,
};
use crate::state::GatewayState;

pub fn create_notification_routes() -> Router<Arc<GatewayState>> {
    Router::new()
        .route(
            "/notifications",
            axum::routing::get(get_notifications).put(mark_all_read),
        )
        .route("/notifications/count", axum::routing::get(get_unread_count))
        .route(
            "/notifications/:notification_id",
            axum::routing::put(mark_notification_read).delete(delete_notification),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(ListNotificationsQuery),
    responses(
        (status = 200, description = "Notifications for current user", body = NotificationsResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch notifications", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_notifications(
    State(state): State<Arc<GatewayState>>,
    Query(params): Query<ListNotificationsQuery>,
    request: Request,
) -> GatewayResult<Json<NotificationsResponse>> {
    let user_id = extract_user_id(&request)?;

    let db_notifications = state
        .notification_service
        .get_notifications(
            user_id,
            params.limit.unwrap_or(50) as u32,
            params.offset.unwrap_or(0) as u32,
        )
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch notifications: {}", e)))?;

    // Apply unread filter if requested
    let db_notifications: Vec<_> = if params.unread_only.unwrap_or(false) {
        db_notifications.into_iter().filter(|n| !n.is_read).collect()
    } else {
        db_notifications
    };

    let notifications: Vec<Notification> = db_notifications.into_iter().map(to_api_notification).collect();

    Ok(Json(NotificationsResponse { notifications }))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/count",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Count of unread notifications", body = UnreadCountResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to fetch count", body = crate::error::ErrorResponse)
    )
)]
pub async fn get_unread_count(
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<UnreadCountResponse>> {
    let user_id = extract_user_id(&request)?;

    let unread_count = state
        .notification_service
        .get_unread_count(user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch unread count: {}", e)))?;

    Ok(Json(UnreadCountResponse { unread_count: unread_count as i64 }))
}

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
        (status = 200, description = "Notification updated", body = NotificationResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_notification_read(
    State(state): State<Arc<GatewayState>>,
    Path(notification_id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(_req): Json<MarkNotificationReadRequest>,
) -> GatewayResult<Json<NotificationResponse>> {
    // Mark as read (the service method marks read, not toggle)
    state
        .notification_service
        .mark_as_read(notification_id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to update notification: {}", e)))?;

    // Get the updated notification
    let notifications = state
        .notification_service
        .get_notifications(user_id, 100, 0)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch notification: {}", e)))?;

    let notification = notifications
        .into_iter()
        .find(|n| n.id == Some(notification_id))
        .ok_or_else(|| GatewayError::NotFound("Notification not found".to_string()))?;

    Ok(Json(NotificationResponse {
        notification: to_api_notification(notification),
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/notifications",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "All notifications marked as read", body = BulkUpdateResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to update notifications", body = crate::error::ErrorResponse)
    )
)]
pub async fn mark_all_read(
    State(state): State<Arc<GatewayState>>,
    request: Request,
) -> GatewayResult<Json<BulkUpdateResponse>> {
    let user_id = extract_user_id(&request)?;

    let updated_count = state
        .notification_service
        .mark_all_as_read(user_id)
        .await
        .map_err(|e| {
            GatewayError::ServiceError(format!("Failed to mark all notifications as read: {}", e))
        })?;

    Ok(Json(BulkUpdateResponse { updated_count: updated_count as u64 }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notifications/{notification_id}",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("notification_id" = i64, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification deleted"),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 404, description = "Notification not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Failed to delete notification", body = crate::error::ErrorResponse)
    )
)]
pub async fn delete_notification(
    State(state): State<Arc<GatewayState>>,
    Path(notification_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> GatewayResult<()> {
    state
        .notification_service
        .delete_notification(notification_id, user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to delete notification: {}", e)))?;

    Ok(())
}
