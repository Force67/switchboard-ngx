//! Notifications service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::NotificationsServiceTrait;
use crate::rest::models::{
    to_api_notification, BulkUpdateResponse, ListNotificationsQuery, MarkNotificationReadRequest,
    Notification, NotificationResponse, NotificationsResponse, UnreadCountResponse,
};
use crate::state::GatewayState;

/// Implementation of NotificationsServiceTrait
pub struct NotificationsServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> NotificationsServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl NotificationsServiceTrait for NotificationsServiceImpl<'_> {
    async fn get_notifications(
        &self,
        user_id: i64,
        query: ListNotificationsQuery,
    ) -> GatewayResult<NotificationsResponse> {
        let db_notifications = self
            .state
            .notification_service
            .get_notifications(
                user_id,
                query.limit.unwrap_or(50) as u32,
                query.offset.unwrap_or(0) as u32,
            )
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch notifications: {}", e)))?;

        // Apply unread filter if requested
        let db_notifications: Vec<_> = if query.unread_only.unwrap_or(false) {
            db_notifications
                .into_iter()
                .filter(|n| !n.is_read)
                .collect()
        } else {
            db_notifications
        };

        let notifications: Vec<Notification> = db_notifications
            .into_iter()
            .map(to_api_notification)
            .collect();

        Ok(NotificationsResponse { notifications })
    }

    async fn get_unread_count(&self, user_id: i64) -> GatewayResult<UnreadCountResponse> {
        let unread_count = self
            .state
            .notification_service
            .get_unread_count(user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch unread count: {}", e)))?;

        Ok(UnreadCountResponse {
            unread_count: unread_count as i64,
        })
    }

    async fn mark_notification_read(
        &self,
        user_id: i64,
        notification_id: String,
        _req: MarkNotificationReadRequest,
    ) -> GatewayResult<NotificationResponse> {
        let notification_id: i64 = notification_id
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid notification ID".to_string()))?;

        // Mark as read (the service method marks read, not toggle)
        self.state
            .notification_service
            .mark_as_read(notification_id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to update notification: {}", e)))?;

        // Get the updated notification
        let notifications = self
            .state
            .notification_service
            .get_notifications(user_id, 100, 0)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to fetch notification: {}", e)))?;

        let notification = notifications
            .into_iter()
            .find(|n| n.id == Some(notification_id))
            .ok_or_else(|| GatewayError::NotFound("Notification not found".to_string()))?;

        Ok(NotificationResponse {
            notification: to_api_notification(notification),
        })
    }

    async fn mark_all_read(&self, user_id: i64) -> GatewayResult<BulkUpdateResponse> {
        let updated_count = self
            .state
            .notification_service
            .mark_all_as_read(user_id)
            .await
            .map_err(|e| {
                GatewayError::ServiceError(format!("Failed to mark all notifications as read: {}", e))
            })?;

        Ok(BulkUpdateResponse {
            updated_count: updated_count as u64,
        })
    }

    async fn delete_notification(&self, user_id: i64, notification_id: String) -> GatewayResult<()> {
        let notification_id: i64 = notification_id
            .parse()
            .map_err(|_| GatewayError::InvalidRequest("Invalid notification ID".to_string()))?;

        self.state
            .notification_service
            .delete_notification(notification_id, user_id)
            .await
            .map_err(|e| GatewayError::ServiceError(format!("Failed to delete notification: {}", e)))?;

        Ok(())
    }
}
