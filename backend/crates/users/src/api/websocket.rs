//! WebSocket handlers for user-related real-time functionality

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use switchboard_database::{User, AuthSession, SessionService, UserError, UserResult};

/// WebSocket state for managing user connections and broadcasts
#[derive(Clone)]
pub struct UserWebSocketState {
    /// Active user connections
    pub user_connections: Arc<RwLock<HashMap<i64, broadcast::Sender<UserServerEvent>>>>,
    /// Session service for authentication
    pub session_service: SessionService,
}

impl UserWebSocketState {
    pub fn new(session_service: SessionService) -> Self {
        Self {
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            session_service,
        }
    }

    /// Get or create a broadcaster for a specific user
    pub async fn get_user_broadcaster(&self, user_id: i64) -> broadcast::Sender<UserServerEvent> {
        let mut connections = self.user_connections.write().await;
        connections
            .entry(user_id)
            .or_insert_with(|| tokio::sync::broadcast::channel(100).0)
            .clone()
    }

    /// Broadcast an event to a specific user
    pub async fn broadcast_to_user(&self, user_id: i64, event: &UserServerEvent) -> UserResult<()> {
        let broadcaster = self.get_user_broadcaster(user_id).await;
        let _ = broadcaster.send(event.clone());
        Ok(())
    }

    /// Remove user connection
    pub async fn remove_user_connection(&self, user_id: i64) {
        let mut connections = self.user_connections.write().await;
        connections.remove(&user_id);
    }
}

/// Client events received from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserClientEvent {
    /// Heartbeat to keep connection alive
    Ping,
    /// Subscribe to user notifications
    Subscribe,
    /// Unsubscribe from user notifications
    Unsubscribe,
    /// Update user presence status
    UpdatePresence {
        status: String, // "online", "away", "busy", "offline"
    },
    /// Get user profile
    GetUserProfile {
        user_id: String,
    },
    /// Update user profile
    UpdateUserProfile {
        display_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    },
    /// Get user settings
    GetUserSettings,
    /// Update user settings
    UpdateUserSettings {
        /// Theme preference
        theme: Option<String>,
        /// Language preference
        language: Option<String>,
        /// Email notifications enabled
        email_notifications: Option<bool>,
        /// Push notifications enabled
        push_notifications: Option<bool>,
    },
    /// Get user notifications
    GetNotifications {
        limit: Option<i64>,
        offset: Option<i64>,
        unread_only: Option<bool>,
    },
    /// Mark notification as read
    MarkNotificationRead {
        notification_id: String,
    },
    /// Mark all notifications as read
    MarkAllNotificationsRead,
    /// Delete notification
    DeleteNotification {
        notification_id: String,
    },
}

/// Server events sent to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserServerEvent {
    /// Welcome message after successful connection
    Hello {
        user_id: String,
        message: String,
    },
    /// Subscription confirmation
    Subscribed {
        user_id: String,
    },
    /// Unsubscription confirmation
    Unsubscribed {
        user_id: String,
    },
    /// Heartbeat response
    Pong,
    /// Error response
    Error {
        error: String,
        message: String,
        request_id: Option<String>,
    },
    /// User profile data
    UserProfile {
        user: UserProfileResponse,
    },
    /// User settings data
    UserSettings {
        settings: UserSettingsResponse,
    },
    /// User notifications
    Notifications {
        notifications: Vec<NotificationResponse>,
        total_count: i64,
        unread_count: i64,
    },
    /// New notification
    NewNotification {
        notification: NotificationResponse,
    },
    /// User presence update
    PresenceUpdate {
        user_id: String,
        status: String,
        last_seen: String,
    },
    /// User updated their profile
    UserUpdated {
        user: UserProfileResponse,
    },
    /// User online status change
    UserOnline {
        user_id: String,
    },
    /// User offline status change
    UserOffline {
        user_id: String,
    },
}

/// User profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub email: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub status: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
    pub email_verified: bool,
    pub is_active: bool,
}

/// User settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub theme: String,
    pub language: String,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub timezone: String,
    pub updated_at: String,
}

/// Notification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub notification_type: String,
    pub read: bool,
    pub data: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// WebSocket connection handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<UserWebSocketState>,
    token: Option<String>,
) -> Response {
    // Authenticate user
    let (user_id, user) = match authenticate_user(&state, token).await {
        Ok(result) => result,
        Err(e) => {
            return axum::response::Json(serde_json::json!({
                "error": "Authentication failed",
                "message": e.to_string()
            }))
            .into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_websocket(socket, state, user_id, user))
}

/// Authenticate user from token
async fn authenticate_user(
    state: &UserWebSocketState,
    token: Option<String>,
) -> UserResult<(i64, User)> {
    let token = token.ok_or(UserError::AuthenticationFailed)?;

    let session = state
        .session_service
        .validate_session(&token)
        .await
        .map_err(|_| UserError::AuthenticationFailed)?;

    // In a real implementation, you would fetch the user from the user service
    // For now, we'll return a placeholder user
    let user = switchboard_database::User {
        id: session.user_id,
        public_id: Uuid::new_v4().to_string(),
        email: None,
        username: "user".to_string(),
        display_name: None,
        avatar_url: None,
        bio: None,
        status: "active".to_string(),
        role: "user".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login_at: Some(chrono::Utc::now()),
        email_verified: false,
        is_active: true,
        password_hash: None,
    };

    Ok((session.user_id, user))
}

/// Handle WebSocket connection
async fn handle_websocket(
    socket: WebSocket,
    state: UserWebSocketState,
    user_id: i64,
    user: User,
) {
    // Split WebSocket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Get user broadcaster
    let broadcaster = state.get_user_broadcaster(user_id).await;
    let mut broadcast_rx = broadcaster.subscribe();

    // Send welcome message
    let welcome_event = UserServerEvent::Hello {
        user_id: user.public_id.clone(),
        message: "Connected to user WebSocket".to_string(),
    };

    if let Ok(text) = serde_json::to_string(&welcome_event) {
        let _ = sender.send(Message::Text(text)).await;
    }

    // Spawn task to handle incoming messages
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        if let Ok(client_event) = serde_json::from_str::<UserClientEvent>(&text) {
                            handle_client_event(client_event, &state, user_id, &user).await;
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    // Spawn task to handle outgoing broadcasts
    let send_task = tokio::spawn(async move {
        while let Ok(event) = broadcast_rx.recv().await {
            if let Ok(text) = serde_json::to_string(&event) {
                let _ = sender.send(Message::Text(text)).await;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {},
        _ = send_task => {},
    }

    // Clean up connection
    state.remove_user_connection(user_id).await;
}

/// Handle client events
async fn handle_client_event(
    event: UserClientEvent,
    state: &UserWebSocketState,
    user_id: i64,
    user: &User,
) {
    match event {
        UserClientEvent::Ping => {
            let pong_event = UserServerEvent::Pong;
            let _ = state.broadcast_to_user(user_id, &pong_event).await;
        }
        UserClientEvent::Subscribe => {
            let subscribe_event = UserServerEvent::Subscribed {
                user_id: user.public_id.clone(),
            };
            let _ = state.broadcast_to_user(user_id, &subscribe_event).await;
        }
        UserClientEvent::Unsubscribe => {
            let unsubscribe_event = UserServerEvent::Unsubscribed {
                user_id: user.public_id.clone(),
            };
            let _ = state.broadcast_to_user(user_id, &unsubscribe_event).await;
        }
        UserClientEvent::UpdatePresence { status } => {
            // In a real implementation, update user presence in database
            let presence_event = UserServerEvent::PresenceUpdate {
                user_id: user.public_id.clone(),
                status: status.clone(),
                last_seen: chrono::Utc::now().to_rfc3339(),
            };
            let _ = state.broadcast_to_user(user_id, &presence_event).await;
        }
        UserClientEvent::GetUserProfile { user_id: target_user_id } => {
            // In a real implementation, fetch user profile
            let profile_response = UserProfileResponse {
                id: user.public_id.clone(),
                email: user.email.clone(),
                username: user.username.clone(),
                display_name: user.display_name.clone(),
                avatar_url: user.avatar_url.clone(),
                bio: user.bio.clone(),
                status: user.status.clone(),
                role: user.role.clone(),
                created_at: user.created_at.to_rfc3339(),
                updated_at: user.updated_at.to_rfc3339(),
                last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
                email_verified: user.email_verified,
                is_active: user.is_active,
            };

            let profile_event = UserServerEvent::UserProfile {
                user: profile_response,
            };
            let _ = state.broadcast_to_user(user_id, &profile_event).await;
        }
        UserClientEvent::UpdateUserProfile { display_name, avatar_url, bio } => {
            // In a real implementation, update user profile in database
            let updated_user = UserProfileResponse {
                id: user.public_id.clone(),
                email: user.email.clone(),
                username: user.username.clone(),
                display_name: display_name.or(user.display_name.clone()),
                avatar_url: avatar_url.or(user.avatar_url.clone()),
                bio: bio.or(user.bio.clone()),
                status: user.status.clone(),
                role: user.role.clone(),
                created_at: user.created_at.to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
                email_verified: user.email_verified,
                is_active: user.is_active,
            };

            let update_event = UserServerEvent::UserUpdated {
                user: updated_user,
            };
            let _ = state.broadcast_to_user(user_id, &update_event).await;
        }
        UserClientEvent::GetUserSettings => {
            // In a real implementation, fetch user settings
            let settings_response = UserSettingsResponse {
                theme: "dark".to_string(),
                language: "en".to_string(),
                email_notifications: true,
                push_notifications: true,
                timezone: "UTC".to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let settings_event = UserServerEvent::UserSettings {
                settings: settings_response,
            };
            let _ = state.broadcast_to_user(user_id, &settings_event).await;
        }
        UserClientEvent::UpdateUserSettings { theme, language, email_notifications, push_notifications } => {
            // In a real implementation, update user settings in database
            let settings_response = UserSettingsResponse {
                theme: theme.unwrap_or("dark".to_string()),
                language: language.unwrap_or("en".to_string()),
                email_notifications: email_notifications.unwrap_or(true),
                push_notifications: push_notifications.unwrap_or(true),
                timezone: "UTC".to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let settings_event = UserServerEvent::UserSettings {
                settings: settings_response,
            };
            let _ = state.broadcast_to_user(user_id, &settings_event).await;
        }
        UserClientEvent::GetNotifications { limit, offset, unread_only } => {
            // In a real implementation, fetch notifications from database
            let notifications = vec![]; // Placeholder
            let notifications_event = UserServerEvent::Notifications {
                notifications,
                total_count: 0,
                unread_count: 0,
            };
            let _ = state.broadcast_to_user(user_id, &notifications_event).await;
        }
        UserClientEvent::MarkNotificationRead { notification_id } => {
            // In a real implementation, mark notification as read
            // For now, just acknowledge the operation
        }
        UserClientEvent::MarkAllNotificationsRead => {
            // In a real implementation, mark all notifications as read
        }
        UserClientEvent::DeleteNotification { notification_id } => {
            // In a real implementation, delete notification
        }
    }
}