use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    routes::models::{
        MarkNotificationReadRequest, Notification, NotificationResponse, NotificationsResponse,
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

    let unread_only = query.unread_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let notifications = notification_service::list_notifications(
        state.db_pool(),
        user.id,
        unread_only,
        limit,
        offset,
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
) -> Result<Json<serde_json::Value>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let count = notification_service::get_unread_count(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch unread notification count: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(serde_json::json!({ "unread_count": count })))
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

    let notification = notification_service::mark_notification_read(
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
) -> Result<Json<serde_json::Value>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let updated_count = notification_service::mark_all_read(state.db_pool(), user.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark all notifications as read: {}", e);
            ApiError::from(e)
        })?;

    Ok(Json(serde_json::json!({
        "updated_count": updated_count
    })))
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

    notification_service::delete_notification(state.db_pool(), user.id, notification_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete notification: {}", e);
            ApiError::from(e)
        })?;

    Ok(())
}

// Notification service for creating notifications
pub struct NotificationService;

impl NotificationService {
    // Create a notification for a user
    pub async fn create_notification(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        user_id: i64,
        notification_type: &str,
        title: &str,
        body: &str,
    ) -> Result<i64, ApiError> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, title, body, read, created_at)
            VALUES (?, ?, ?, ?, FALSE, ?)
            "#,
        )
        .bind(user_id)
        .bind(notification_type)
        .bind(title)
        .bind(body)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create notification: {}", e);
            ApiError::internal_server_error("Failed to create notification")
        })?;

        Ok(result.last_insert_rowid())
    }

    // Notify users in a chat about a new message
    pub async fn notify_new_message(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        chat_id: i64,
        sender_user_id: i64,
        sender_name: &str,
        chat_title: &str,
    ) -> Result<(), ApiError> {
        // Get all members of the chat except the sender
        let members = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT u.id
            FROM users u
            JOIN chat_members cm ON u.id = cm.user_id
            WHERE cm.chat_id = ? AND cm.user_id != ?
            "#,
        )
        .bind(chat_id)
        .bind(sender_user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch chat members for notification: {}", e);
            ApiError::internal_server_error("Failed to fetch chat members for notification")
        })?;

        let now = chrono::Utc::now().to_rfc3339();

        // Create notifications for all members
        for user_id in members {
            let title = format!("New message in {}", chat_title);
            let body = format!("{} sent a message", sender_name);

            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, title, body, read, created_at)
                VALUES (?, ?, ?, ?, FALSE, ?)
                "#,
            )
            .bind(user_id)
            .bind("new_message")
            .bind(&title)
            .bind(&body)
            .bind(&now)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create message notification: {}", e);
                ApiError::internal_server_error("Failed to create message notification")
            })?;
        }

        Ok(())
    }

    // Notify user about chat invite
    pub async fn notify_chat_invite(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        invited_user_id: i64,
        chat_title: &str,
        inviter_name: &str,
    ) -> Result<(), ApiError> {
        let title = format!("Chat invite: {}", chat_title);
        let body = format!("{} invited you to join a chat", inviter_name);

        Self::create_notification(pool, invited_user_id, "chat_invite", &title, &body).await?;

        Ok(())
    }

    // Notify user about accepted invite
    pub async fn notify_invite_accepted(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        _chat_id: i64,
        inviter_user_id: i64,
        accepted_user_name: &str,
        chat_title: &str,
    ) -> Result<(), ApiError> {
        let title = format!("Invite accepted for {}", chat_title);
        let body = format!("{} accepted your chat invite", accepted_user_name);

        Self::create_notification(pool, inviter_user_id, "invite_accepted", &title, &body).await?;

        Ok(())
    }

    // Notify user about message mention
    pub async fn notify_mention(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        mentioned_user_id: i64,
        sender_name: &str,
        chat_title: &str,
    ) -> Result<(), ApiError> {
        let title = format!("You were mentioned in {}", chat_title);
        let body = format!("{} mentioned you in a message", sender_name);

        Self::create_notification(pool, mentioned_user_id, "mention", &title, &body).await?;

        Ok(())
    }
}
