use axum::{
    extract::{Extension, Path, Query, Request, State},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

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

/// Convert database notification to API response model
fn to_api_notification(n: switchboard_database::Notification) -> crate::rest::models::Notification {
    crate::rest::models::Notification {
        id: n.id.unwrap_or(0),
        user_id: n.user_id,
        r#type: n.notification_type.to_string(),
        title: n.title,
        body: n.message,
        read: n.is_read,
        created_at: n.created_at,
    }
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

    let limit = query.limit.unwrap_or(50) as u32;
    let offset = query.offset.unwrap_or(0) as u32;

    let notifications = state
        .notification_service()
        .get_notifications(user_id, limit, offset)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

    let api_notifications = notifications.into_iter().map(to_api_notification).collect();

    Ok(Json(NotificationsResponse {
        notifications: api_notifications,
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
    request: Request,
) -> GatewayResult<Json<UnreadCountResponse>> {
    let user_id = extract_user_id(&request)?;

    let count = state
        .notification_service()
        .get_unread_count(user_id)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

    Ok(Json(UnreadCountResponse {
        unread_count: count as i64,
    }))
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
    Extension(user_id): Extension<i64>,
    Json(payload): Json<MarkNotificationReadRequest>,
) -> GatewayResult<()> {
    if payload.read {
        state
            .notification_service()
            .mark_as_read(notification_id, user_id)
            .await
            .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;
    }

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
    request: Request,
) -> GatewayResult<Json<BulkUpdateResponse>> {
    let user_id = extract_user_id(&request)?;

    let updated_count = state
        .notification_service()
        .mark_all_as_read(user_id)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

    Ok(Json(BulkUpdateResponse {
        updated_count: updated_count as u64,
    }))
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
    request: Request,
) -> GatewayResult<()> {
    let user_id = extract_user_id(&request)?;

    state
        .notification_service()
        .delete_notification(notification_id, user_id)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

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