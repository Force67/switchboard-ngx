//! Chat WebSocket handlers

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
        Query,
    },
    response::{Response, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use futures_util::{StreamExt, SinkExt};

use crate::state::GatewayState;
use crate::error::GatewayError;

/// WebSocket state for managing chat connections and broadcasts
#[derive(Clone)]
pub struct ChatWebSocketState {
    /// Active chat subscriptions
    pub chat_subscriptions: Arc<RwLock<HashMap<String, (i64, broadcast::Sender<ChatServerEvent>)>>>,
    /// Active user connections
    pub user_connections: Arc<RwLock<HashMap<i64, broadcast::Sender<ChatServerEvent>>>>,
    /// Gateway state with access to services
    pub gateway_state: Arc<GatewayState>,
}

impl ChatWebSocketState {
    pub fn new(gateway_state: Arc<GatewayState>) -> Self {
        Self {
            chat_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            gateway_state,
        }
    }

    /// Get or create a broadcaster for a specific chat
    pub async fn get_chat_broadcaster(&self, chat_id: &str) -> broadcast::Sender<ChatServerEvent> {
        let mut subscriptions = self.chat_subscriptions.write().await;
        let (sender, _) = tokio::sync::broadcast::channel(100);
        subscriptions
            .entry(chat_id.to_string())
            .or_insert_with(|| (0, sender))
            .1
            .clone()
    }

    /// Get or create a broadcaster for a specific user
    pub async fn get_user_broadcaster(&self, user_id: i64) -> broadcast::Sender<ChatServerEvent> {
        let mut connections = self.user_connections.write().await;
        connections
            .entry(user_id)
            .or_insert_with(|| tokio::sync::broadcast::channel(100).0)
            .clone()
    }

    /// Broadcast an event to a specific chat
    pub async fn broadcast_to_chat(&self, chat_id: &str, event: &ChatServerEvent) -> Result<(), GatewayError> {
        let broadcaster = self.get_chat_broadcaster(chat_id).await;
        let _ = broadcaster.send(event.clone());
        Ok(())
    }

    /// Broadcast an event to a specific user
    pub async fn broadcast_to_user(&self, user_id: i64, event: &ChatServerEvent) -> Result<(), GatewayError> {
        let broadcaster = self.get_user_broadcaster(user_id).await;
        let _ = broadcaster.send(event.clone());
        Ok(())
    }

    /// Subscribe a user to a chat
    pub async fn subscribe_to_chat(&self, chat_id: &str, user_id: i64) -> Result<(), GatewayError> {
        let broadcaster = self.get_chat_broadcaster(chat_id).await;
        let mut subscriptions = self.chat_subscriptions.write().await;
        subscriptions.insert(chat_id.to_string(), (user_id, broadcaster));
        Ok(())
    }

    /// Unsubscribe a user from a chat
    pub async fn unsubscribe_from_chat(&self, chat_id: &str) -> Result<(), GatewayError> {
        let mut subscriptions = self.chat_subscriptions.write().await;
        subscriptions.remove(chat_id);
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
pub enum ChatClientEvent {
    /// Heartbeat to keep connection alive
    Ping,
    /// Subscribe to chat events
    Subscribe {
        chat_id: String,
    },
    /// Unsubscribe from chat events
    Unsubscribe {
        chat_id: String,
    },
    /// Send a message
    SendMessage {
        chat_id: String,
        content: Option<String>,
        message_type: Option<String>,
        reply_to: Option<String>,
        thread_id: Option<String>,
    },
    /// Update a message
    UpdateMessage {
        chat_id: String,
        message_id: String,
        content: Option<String>,
    },
    /// Delete a message
    DeleteMessage {
        chat_id: String,
        message_id: String,
    },
    /// Typing indicator
    Typing {
        chat_id: String,
        is_typing: bool,
    },
    /// Create invite
    CreateInvite {
        chat_id: String,
        email: String,
        expires_in_hours: Option<i64>,
    },
    /// Get chat invites
    GetInvites {
        chat_id: String,
        status: Option<String>,
    },
    /// Respond to invite
    RespondToInvite {
        invite_id: String,
        action: String, // "accept" or "reject"
    },
    /// Get chat members
    GetMembers {
        chat_id: String,
        role: Option<String>,
    },
    /// Update member role
    UpdateMemberRole {
        chat_id: String,
        member_id: String,
        role: String,
    },
    /// Remove member
    RemoveMember {
        chat_id: String,
        member_id: String,
    },
    /// Leave chat
    LeaveChat {
        chat_id: String,
    },
}

/// Server events sent to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatServerEvent {
    /// Welcome message after successful connection
    Hello {
        user_id: String,
        message: String,
    },
    /// Heartbeat response
    Pong,
    /// Error response
    Error {
        error: String,
        message: String,
        request_id: Option<String>,
    },
    /// Subscription confirmation
    Subscribed {
        chat_id: String,
    },
    /// Unsubscription confirmation
    Unsubscribed {
        chat_id: String,
    },
    /// New message
    Message {
        chat_id: String,
        message: MessageResponse,
    },
    /// Message updated
    MessageUpdated {
        chat_id: String,
        message: MessageResponse,
    },
    /// Message deleted
    MessageDeleted {
        chat_id: String,
        message_id: String,
    },
    /// User is typing
    UserTyping {
        chat_id: String,
        user_id: String,
        is_typing: bool,
    },
    /// Invite created
    InviteCreated {
        invite: InviteResponse,
    },
    /// Invite updated
    InviteUpdated {
        invite: InviteResponse,
    },
    /// List of invites
    Invites {
        chat_id: String,
        invites: Vec<InviteResponse>,
    },
    /// Member joined
    MemberJoined {
        chat_id: String,
        member: MemberResponse,
    },
    /// Member left
    MemberLeft {
        chat_id: String,
        member_id: String,
    },
    /// Member role updated
    MemberRoleUpdated {
        chat_id: String,
        member: MemberResponse,
    },
    /// List of members
    Members {
        chat_id: String,
        members: Vec<MemberResponse>,
    },
    /// Attachment uploaded
    AttachmentUploaded {
        chat_id: String,
        message_id: String,
        attachment: AttachmentResponse,
    },
    /// Attachment deleted
    AttachmentDeleted {
        chat_id: String,
        message_id: String,
        attachment_id: String,
    },
}

// Response structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: Option<String>,
    pub message_type: String,
    pub reply_to: Option<String>,
    pub thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub edited: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteResponse {
    pub id: String,
    pub chat_id: String,
    pub chat_title: String,
    pub invited_by: String,
    pub invited_email: String,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub accepted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub chat_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub message_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_url: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: Option<String>,
}

/// Chat WebSocket connection handler
pub async fn chat_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<WebSocketQuery>,
) -> Result<Response, GatewayError> {
    // Create chat WebSocket state
    let chat_ws_state = ChatWebSocketState::new(state.clone());

    // Authenticate user
    let (user_id, _user) = match authenticate_user(&state, query.token).await {
        Ok(result) => result,
        Err(e) => {
            return Ok(axum::response::Json(serde_json::json!({
                "error": "Authentication failed",
                "message": e.to_string()
            }))
            .into_response());
        }
    };

    Ok(ws.on_upgrade(move |socket| handle_chat_websocket(socket, chat_ws_state, user_id)))
}

/// Authenticate user from token
async fn authenticate_user(
    state: &Arc<GatewayState>,
    token: Option<String>,
) -> Result<(i64, switchboard_database::User), GatewayError> {
    let token = token.ok_or(GatewayError::AuthenticationFailed("Missing token".to_string()))?;

    let session = state
        .session_service
        .validate_session(&token)
        .await
        .map_err(|e| GatewayError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

    // Get user from session
    let user = state
        .user_service
        .get_user(session.user_id)
        .await
        .map_err(|e| GatewayError::ServiceError(format!("Failed to get user: {}", e)))?;

    Ok((session.user_id, user))
}

/// Handle chat WebSocket connection
async fn handle_chat_websocket(
    socket: WebSocket,
    state: ChatWebSocketState,
    user_id: i64,
) {
    // Split WebSocket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Get user broadcaster
    let user_broadcaster = state.get_user_broadcaster(user_id).await;
    let mut user_broadcast_rx = user_broadcaster.subscribe();

    // Send welcome message
    let welcome_event = ChatServerEvent::Hello {
        user_id: user_id.to_string(),
        message: "Connected to chat WebSocket".to_string(),
    };

    if let Ok(text) = serde_json::to_string(&welcome_event) {
        let _ = sender.send(Message::Text(text)).await;
    }

    // Spawn task to handle incoming messages
    let state_clone = state.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        if let Ok(client_event) = serde_json::from_str::<ChatClientEvent>(&text) {
                            handle_chat_client_event(client_event, &state_clone, user_id).await;
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
        while let Ok(event) = user_broadcast_rx.recv().await {
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

/// Handle chat client events
async fn handle_chat_client_event(
    event: ChatClientEvent,
    state: &ChatWebSocketState,
    user_id: i64,
) {
    match event {
        ChatClientEvent::Ping => {
            let pong_event = ChatServerEvent::Pong;
            let _ = state.broadcast_to_user(user_id, &pong_event).await;
        }
        ChatClientEvent::Subscribe { chat_id } => {
            // Check if user is member of chat
            if let Ok(()) = state.gateway_state.member_service.check_chat_membership(&chat_id, user_id).await {
                let _ = state.subscribe_to_chat(&chat_id, user_id).await;
                let subscribe_event = ChatServerEvent::Subscribed { chat_id };
                let _ = state.broadcast_to_user(user_id, &subscribe_event).await;
            } else {
                let error_event = ChatServerEvent::Error {
                    error: "ACCESS_DENIED".to_string(),
                    message: "You are not a member of this chat".to_string(),
                    request_id: None,
                };
                let _ = state.broadcast_to_user(user_id, &error_event).await;
            }
        }
        ChatClientEvent::Unsubscribe { chat_id } => {
            let _ = state.unsubscribe_from_chat(&chat_id).await;
            let unsubscribe_event = ChatServerEvent::Unsubscribed { chat_id };
            let _ = state.broadcast_to_user(user_id, &unsubscribe_event).await;
        }
        ChatClientEvent::Typing { chat_id, is_typing } => {
            // Check if user is member of chat
            if let Ok(()) = state.gateway_state.member_service.check_chat_membership(&chat_id, user_id).await {
                let typing_event = ChatServerEvent::UserTyping {
                    chat_id: chat_id.clone(),
                    user_id: user_id.to_string(),
                    is_typing,
                };
                let _ = state.broadcast_to_chat(&chat_id, &typing_event).await;
            }
        }
        ChatClientEvent::SendMessage { chat_id, content, message_type, reply_to, thread_id } => {
            // Check if user is member of chat
            if let Ok(()) = state.gateway_state.member_service.check_chat_membership(&chat_id, user_id).await {
                let msg_type = match message_type.as_deref() {
                    Some("image") => switchboard_database::MessageType::Image,
                    Some("file") => switchboard_database::MessageType::File,
                    Some("system") => switchboard_database::MessageType::System,
                    _ => switchboard_database::MessageType::Text,
                };

                let chat_id_clone = chat_id.clone();
                let create_req = switchboard_database::CreateMessageRequest {
                    chat_id: 0, // This should be resolved from chat_public_id
                    sender_id: user_id,
                    chat_public_id: chat_id.clone(),
                    sender_public_id: user_id.to_string(),
                    content,
                    message_type: msg_type,
                    reply_to_public_id: reply_to,
                    thread_public_id: thread_id,
                };

                if let Ok(message) = state.gateway_state.message_service.create(&create_req, user_id).await {
                    let message_response = MessageResponse {
                        id: message.public_id,
                        chat_id: message.chat_public_id,
                        sender_id: message.sender_public_id,
                        content: message.content,
                        message_type: message.message_type.to_string(),
                        reply_to: message.reply_to_public_id,
                        thread_id: message.thread_public_id,
                        created_at: message.created_at.clone(),
                        updated_at: message.updated_at.clone(),
                        edited: message.updated_at.is_some(),
                        deleted: message.deleted_at.is_some(),
                    };

                    let message_event = ChatServerEvent::Message {
                        chat_id: chat_id_clone,
                        message: message_response,
                    };
                    let _ = state.broadcast_to_chat(&chat_id, &message_event).await;
                }
            }
        }
        // Add more event handlers as needed...
        _ => {
            // For unhandled events, send an error response
            let error_event = ChatServerEvent::Error {
                error: "UNHANDLED_EVENT".to_string(),
                message: "This event type is not yet implemented".to_string(),
                request_id: None,
            };
            let _ = state.broadcast_to_user(user_id, &error_event).await;
        }
    }
}