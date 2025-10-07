use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::StatusCode,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

use crate::state::{AppState, ClientEvent, ServerEvent};

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: Option<String>,
}

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
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user)))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState, user: switchboard_auth::User) {
    let (mut ws_sender, mut receiver) = socket.split();
    let mut subscribed_chats = HashMap::new(); // chat_public_id -> (chat_db_id, broadcaster)

    let (out_tx, mut out_rx) = mpsc::channel::<ServerEvent>(100);
    let sender_task = tokio::spawn(async move {
        while let Some(event) = out_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if ws_sender.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                break;
            }
        }
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
                        if let Err(e) = handle_client_event(event, &out_tx, &state, &user, &mut subscribed_chats).await {
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
                tracing::debug!("WebSocket connection closed for user {}", user.id);
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

    tracing::debug!("WebSocket handler finished for user {}", user.id);
}

async fn handle_client_event(
    event: ClientEvent,
    out_tx: &mpsc::Sender<ServerEvent>,
    state: &AppState,
    user: &switchboard_auth::User,
    subscribed_chats: &mut HashMap<String, (i64, broadcast::Sender<ServerEvent>)>, // chat_public_id -> (chat_db_id, broadcaster)
) -> Result<(), anyhow::Error> {
    match event {
        ClientEvent::Subscribe { chat_id } => {
            // Find the chat by public_id
            let chat_db_id: Option<i64> = sqlx::query_scalar("SELECT id FROM chats WHERE public_id = ?")
                .bind(&chat_id)
                .fetch_optional(&state.db_pool)
                .await?;

            let chat_db_id = match chat_db_id {
                Some(id) => id,
                None => {
                    let error = ServerEvent::Error {
                        message: "Chat not found".to_string(),
                    };
                    out_tx.send(error).await?;
                    return Ok(());
                }
            };

            // Check if user is a member of the chat
            let is_member: Option<i64> = sqlx::query_scalar("SELECT 1 FROM chat_members WHERE chat_id = ? AND user_id = ?")
                .bind(chat_db_id)
                .bind(user.id)
                .fetch_optional(&state.db_pool)
                .await?;

            if is_member.is_none() {
                let error = ServerEvent::Error {
                    message: "Not a member of this chat".to_string(),
                };
                out_tx.send(error).await?;
                return Ok(());
            }

            // Get or create broadcaster for this chat
            let broadcaster = {
                let mut broadcasters = state.chat_broadcasters.lock().await;
                broadcasters.entry(chat_id.clone()).or_insert_with(|| broadcast::channel(100).0).clone()
            };

            // Start broadcasting task
            let tx = out_tx.clone();
            let broadcaster_clone = broadcaster.clone();
            tokio::spawn(async move {
                let mut receiver = broadcaster_clone.subscribe();
                while let Ok(event) = receiver.recv().await {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            });

            subscribed_chats.insert(chat_id.clone(), (chat_db_id, broadcaster));
            let response = ServerEvent::Subscribed { chat_id };
            out_tx.send(response).await?;
        }
        ClientEvent::Unsubscribe { chat_id } => {
            subscribed_chats.remove(&chat_id);
            let response = ServerEvent::Unsubscribed { chat_id };
            out_tx.send(response).await?;
        }
        ClientEvent::Message { chat_id, content } => {
            let (chat_db_id, broadcaster) = match subscribed_chats.get(&chat_id) {
                Some((id, sender)) => (*id, sender.clone()),
                None => {
                    let error = ServerEvent::Error {
                        message: "Not subscribed to chat".to_string(),
                    };
                    out_tx.send(error).await?;
                    return Ok(());
                }
            };

            // Save message to database
            let message_public_id = cuid2::create_id();
            let now = chrono::Utc::now().to_rfc3339();

            sqlx::query(
                r#"
                INSERT INTO messages (public_id, chat_id, user_id, content, message_type, created_at, updated_at)
                VALUES (?, ?, ?, ?, 'text', ?, ?)
                "#
            )
            .bind(&message_public_id)
            .bind(chat_db_id)
            .bind(user.id)
            .bind(&content)
            .bind(&now)
            .bind(&now)
            .execute(&state.db_pool)
            .await?;

            let message_event = ServerEvent::Message {
                chat_id: chat_id.clone(),
                message_id: message_public_id,
                user_id: user.id,
                content,
                timestamp: now,
                message_type: "text".to_string(),
            };
            // Send to self
            out_tx.send(message_event.clone()).await?;
            // Broadcast to others
            let _ = broadcaster.send(message_event);
        }
        ClientEvent::Typing { chat_id, is_typing } => {
            let broadcaster = match subscribed_chats.get(&chat_id) {
                Some((_, sender)) => sender.clone(),
                None => {
                    let error = ServerEvent::Error {
                        message: "Not subscribed to chat".to_string(),
                    };
                    out_tx.send(error).await?;
                    return Ok(());
                }
            };

            let typing_event = ServerEvent::Typing {
                chat_id: chat_id.clone(),
                user_id: user.id,
                is_typing,
            };
            // Send to self
            out_tx.send(typing_event.clone()).await?;
            // Broadcast to others
            let _ = broadcaster.send(typing_event);
        }
    }

    Ok(())
}

