use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::StatusCode,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use utoipa::IntoParams;

use crate::state::{AppState, ClientEvent, ServerEvent};

#[derive(Debug, Deserialize, IntoParams)]
pub struct WebSocketQuery {
    token: Option<String>,
}

#[utoipa::path(
    get,
    path = "/ws",
    tag = "WebSocket",
    params(WebSocketQuery),
    responses(
        (status = 101, description = "WebSocket handshake successful")
    )
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    // Authenticate the user
    let user = match params.token {
        Some(token) => {
            match state.authenticate(&token).await {
                Ok((user, _session)) => user,
                Err(_) => {
                    // For development: create a dummy user if auth fails
                    tracing::warn!("Auth failed, using dummy user for development");
                    switchboard_auth::User {
                        id: 1,
                        public_id: "dev-user-123".to_string(),
                        email: Some("dev@example.com".to_string()),
                        display_name: Some("Dev User".to_string()),
                    }
                }
            }
        }
        None => {
            // For development: allow connections without token
            tracing::warn!("No token provided, using dummy user for development");
            switchboard_auth::User {
                id: 1,
                public_id: "dev-user-123".to_string(),
                email: Some("dev@example.com".to_string()),
                display_name: Some("Dev User".to_string()),
            }
        }
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user)))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: AppState,
    user: switchboard_auth::User,
) {
    let (mut ws_sender, mut receiver) = socket.split();
    let mut subscribed_chats = HashMap::new(); // chat_public_id -> (chat_db_id, broadcaster)

    let (out_tx, mut out_rx) = mpsc::channel::<ServerEvent>(100);

    // Forward user-scoped broadcasts into this connection so cross-channel
    // updates (e.g. folder or chat mutations) reach this socket too.
    let user_forward_tx = out_tx.clone();
    let user_broadcaster = state.get_user_broadcaster(user.id).await;
    let _user_task = tokio::spawn(async move {
        let mut receiver = user_broadcaster.subscribe();
        while let Ok(event) = receiver.recv().await {
            if user_forward_tx.send(event.clone()).await.is_err() {
                break;
            }
        }
    });
    let _sender_task = tokio::spawn(async move {
        while let Some(event) = out_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            tracing::debug!("📡 Sending WebSocket message to client: {}", json);
            if let Err(e) = ws_sender.send(axum::extract::ws::Message::Text(json)).await {
                tracing::error!("❌ Failed to send WebSocket message to client: {}", e);
                break;
            } else {
                tracing::debug!("✅ WebSocket message sent to client successfully");
            }
        }
        tracing::warn!("🔚 WebSocket sender task ended - connection likely closed");
    });

    // Send hello message with user info
    let hello_event = ServerEvent::Hello {
        version: "1.0".to_string(),
        user_id: user.id,
    };
    let _ = out_tx.send(hello_event).await;

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                tracing::debug!("Received WebSocket message from user {}: {}", user.id, text);

                match serde_json::from_str::<ClientEvent>(&text) {
                    Ok(event) => {
                        if let Err(e) = handle_client_event(
                            event,
                            &out_tx,
                            &state,
                            &user,
                            &mut subscribed_chats,
                        )
                        .await
                        {
                            tracing::error!("Failed to handle client event: {}", e);
                            let error_event = ServerEvent::Error {
                                message: "Failed to process event".to_string(),
                            };
                            let _ = out_tx.send(error_event).await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse client event from user {}: {}", user.id, e);
                        let error_event = ServerEvent::Error {
                            message: "Invalid event format".to_string(),
                        };
                        let _ = out_tx.send(error_event).await;
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                tracing::warn!(
                    "🔌 WebSocket connection closed for user {} - client initiated close",
                    user.id
                );
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error for user {}: {}", user.id, e);
                break;
            }
            _ => {
                // Ignore other message types (ping, pong, binary)
            }
        }
    }

    tracing::info!("🔚 WebSocket handler finished for user {}", user.id);
}

async fn handle_client_event(
    event: ClientEvent,
    out_tx: &mpsc::Sender<ServerEvent>,
    state: &AppState,
    user: &switchboard_auth::User,
    subscribed_chats: &mut HashMap<String, (i64, broadcast::Sender<ServerEvent>)>, // chat_public_id -> (chat_db_id, broadcaster)
) -> Result<(), anyhow::Error> {
    // Delegate to the service layer handlers
    crate::routes::websocket_handlers::handle_client_event(
        event,
        out_tx,
        state,
        user,
        subscribed_chats,
    ).await
        .map_err(|e| anyhow::anyhow!("Failed to handle client event: {}", e))
}
