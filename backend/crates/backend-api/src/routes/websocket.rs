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
    match event {
        ClientEvent::Subscribe { chat_id } => {
            // Find the chat by public_id
            let chat_db_id: Option<i64> =
                sqlx::query_scalar("SELECT id FROM chats WHERE public_id = ?")
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
            let is_member: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM chat_members WHERE chat_id = ? AND user_id = ?")
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
                broadcasters
                    .entry(chat_id.clone())
                    .or_insert_with(|| broadcast::channel(100).0)
                    .clone()
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
        ClientEvent::Message {
            chat_id,
            content,
            model,
        } => {
            tracing::info!(
                "📨 Received chat message from user {} in chat {}: {}",
                user.id,
                chat_id,
                content
            );

            // Check if out_tx is still connected (channel not closed)
            tracing::info!("🔍 Checking WebSocket connection state before processing message...");
            if out_tx.is_closed() {
                tracing::error!(
                    "❌ WebSocket connection is already closed, cannot process message"
                );
                return Ok(());
            } else {
                tracing::info!(
                    "✅ WebSocket connection is open, proceeding with message processing"
                );
            }

            let (chat_db_id, broadcaster) = match subscribed_chats.get(&chat_id) {
                Some((id, sender)) => (*id, sender.clone()),
                None => {
                    tracing::warn!(
                        "❌ User {} tried to send message to unsubscribed chat {}",
                        user.id,
                        chat_id
                    );
                    let error = ServerEvent::Error {
                        message: "Not subscribed to chat".to_string(),
                    };
                    out_tx.send(error).await?;
                    return Ok(());
                }
            };

            tracing::debug!("💾 Saving user message to database...");
            // Save user message to database
            let message_public_id = cuid2::create_id();
            let now = chrono::Utc::now().to_rfc3339();

            sqlx::query(
                r#"
                INSERT INTO messages (public_id, chat_id, user_id, content, message_type, role, model, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&message_public_id)
            .bind(chat_db_id)
            .bind(user.id)
            .bind(&content)
            .bind("text")
            .bind("user")
            .bind(Option::<String>::None)
            .bind(&now)
            .bind(&now)
            .execute(&state.db_pool)
            .await?;

            tracing::debug!(
                "✅ User message saved to database with ID: {}",
                message_public_id
            );

            let message_event = ServerEvent::Message {
                chat_id: chat_id.clone(),
                message_id: message_public_id,
                user_id: user.id,
                content: content.clone(),
                model: None,
                timestamp: now.clone(),
                message_type: "text".to_string(),
            };
            // Send user message to self
            tracing::debug!("📤 Sending user message echo to sender via out_tx");
            if let Err(e) = out_tx.send(message_event.clone()).await {
                tracing::error!("❌ Failed to send user message echo: {}", e);
                return Ok(());
            } else {
                tracing::debug!("✅ User message echo sent successfully");
            }
            // Broadcast user message to others
            tracing::debug!("📡 Broadcasting user message to other subscribers");
            if let Err(e) = broadcaster.send(message_event) {
                tracing::error!("❌ Failed to broadcast user message: {}", e);
            } else {
                tracing::debug!("✅ User message broadcasted successfully");
            }

            tracing::info!(
                "🤖 Starting LLM processing for message in chat {}...",
                chat_id
            );
            // Process message with LLM
            let state_clone = state.clone();
            let chat_id_clone = chat_id.clone();
            let content_clone = content.clone();
            let out_tx_clone = out_tx.clone();
            let broadcaster_clone = broadcaster.clone();
            let requested_model = model.clone();
            let chat_db_id = chat_db_id; // Move into the async block
            let user_id = user.id; // Clone the user ID for the async block

            tokio::spawn(async move {
                let chosen_model = requested_model
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(String::from)
                    .or_else(|| state_clone.orchestrator().active_model())
                    .filter(|value| !value.trim().is_empty());

                let model_to_use = match chosen_model {
                    Some(model_id) => model_id,
                    None => {
                        tracing::error!("❌ No model provided and no active model configured");
                        let error_event = ServerEvent::Error {
                            message: "No model configured".to_string(),
                        };
                        let _ = out_tx_clone.send(error_event).await;
                        return;
                    }
                };

                tracing::info!("🧠 Using model {} for chat {}", model_to_use, chat_id_clone);
                tracing::debug!("🔧 Getting LLM provider for model {}", model_to_use);
                let provider = match state_clone.orchestrator().provider_for_model(&model_to_use) {
                    Ok(provider) => {
                        tracing::debug!(
                            "✅ LLM provider obtained successfully for {}",
                            model_to_use
                        );
                        provider
                    }
                    Err(e) => {
                        tracing::error!(
                            "❌ Failed to get LLM provider for {}: {}",
                            model_to_use,
                            e
                        );
                        let error_event = ServerEvent::Error {
                            message: format!(
                                "LLM provider not available for {}: {}",
                                model_to_use, e
                            ),
                        };
                        let _ = out_tx_clone.send(error_event).await;
                        return;
                    }
                };

                tracing::debug!("📝 Preparing completion request for model {}", model_to_use);
                let message = denkwerk::ChatMessage::user(&content_clone);
                let request = denkwerk::CompletionRequest::new(model_to_use.clone(), vec![message]);

                tracing::info!("🚀 Sending request to LLM...");
                match provider.complete(request).await {
                    Ok(completion) => {
                        tracing::info!("✅ LLM response received successfully");
                        let response_content =
                            completion.message.text().unwrap_or_default().to_string();
                        let _reasoning: Option<Vec<String>> = completion
                            .reasoning
                            .map(|steps| steps.into_iter().map(|step| step.content).collect());

                        tracing::debug!("💾 Saving assistant response to database...");
                        // Save assistant response to database
                        let assistant_message_id = cuid2::create_id();
                        let assistant_timestamp = chrono::Utc::now().to_rfc3339();

                        if let Err(e) = sqlx::query(
                            r#"
                            INSERT INTO messages (public_id, chat_id, user_id, content, message_type, role, model, created_at, updated_at)
                            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                            "#
                        )
                        .bind(&assistant_message_id)
                        .bind(chat_db_id)
                        .bind(user_id) // Use the same user ID for assistant messages in development
                        .bind(&response_content)
                        .bind("text")
                        .bind("assistant")
                        .bind(Some(model_to_use.clone()))
                        .bind(&assistant_timestamp)
                        .bind(&assistant_timestamp)
                        .execute(&state_clone.db_pool)
                        .await {
                            tracing::error!("❌ Failed to save assistant message: {}", e);
                            return;
                        }

                        tracing::debug!(
                            "✅ Assistant response saved to database with ID: {}",
                            assistant_message_id
                        );
                        tracing::info!(
                            "📤 Broadcasting assistant response to chat {}",
                            chat_id_clone
                        );

                        let assistant_event = ServerEvent::Message {
                            chat_id: chat_id_clone.clone(),
                            message_id: assistant_message_id,
                            user_id: user_id, // Use the same user ID for assistant messages in development
                            content: response_content,
                            model: Some(model_to_use.clone()),
                            timestamp: assistant_timestamp,
                            message_type: "text".to_string(),
                        };

                        // Send assistant response to self
                        tracing::debug!(
                            "📤 Sending assistant response directly to sender via out_tx"
                        );
                        // Check if the channel is still open (connection hasn't closed)
                        match out_tx_clone.send(assistant_event.clone()).await {
                            Ok(_) => {
                                tracing::debug!("✅ Assistant response sent to sender via out_tx");
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ Failed to send assistant response to sender: {}",
                                    e
                                );
                                tracing::warn!(
                                    "⚠️ WebSocket connection may have closed during LLM processing"
                                );
                                // Don't try to broadcast if we can't send to the original sender
                                return;
                            }
                        }
                        // Broadcast assistant response to others
                        tracing::debug!("📡 Broadcasting assistant response to other subscribers");
                        if let Err(e) = broadcaster_clone.send(assistant_event) {
                            tracing::error!("❌ Failed to broadcast assistant response: {}", e);
                        } else {
                            tracing::debug!("✅ Assistant response broadcasted successfully");
                        }

                        tracing::info!(
                            "✅ Message processing completed for chat {}",
                            chat_id_clone
                        );
                    }
                    Err(e) => {
                        tracing::error!("❌ LLM completion failed: {}", e);
                        let error_message = format!("LLM completion failed: {}", e);
                        let error_event = ServerEvent::Error {
                            message: error_message,
                        };
                        let _ = out_tx_clone.send(error_event).await;
                    }
                }
            });
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
