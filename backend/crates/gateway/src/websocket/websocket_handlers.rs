use crate::{
    routes::{
        models::{CreateMessageRequest, UpdateChatRequest, UpdateMemberRoleRequest},
    },
    services::{chat, folder, invite, member, message, notification, permission},
    state::{AppState, ServerEvent},
};
use tokio::sync::mpsc;
use std::collections::HashMap;

/// Handles all WebSocket client events using the service layer
pub async fn handle_client_event(
    event: crate::state::ClientEvent,
    out_tx: &mpsc::Sender<ServerEvent>,
    state: &AppState,
    user: &switchboard_auth::User,
    subscribed_chats: &mut HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::state::ClientEvent;

    match event {
        // WebSocket connection management
        ClientEvent::Subscribe { chat_id } => {
            handle_subscribe(&chat_id, state, user, out_tx, subscribed_chats).await?
        }
        ClientEvent::Unsubscribe { chat_id } => {
            handle_unsubscribe(&chat_id, out_tx, subscribed_chats).await?
        }

        // Chat operations
        ClientEvent::CreateChat { title, chat_type, folder_id, messages } => {
            handle_create_chat(title, chat_type, folder_id, messages, state, user, out_tx).await?
        }
        ClientEvent::UpdateChat { chat_id, title, folder_id } => {
            handle_update_chat(chat_id, title, folder_id, state, user, out_tx).await?
        }
        ClientEvent::DeleteChat { chat_id } => {
            handle_delete_chat(chat_id, state, user, out_tx).await?
        }
        ClientEvent::GetChats => {
            handle_get_chats(state, user, out_tx).await?
        }
        ClientEvent::GetChat { chat_id } => {
            handle_get_chat(chat_id, state, user, out_tx).await?
        }

        // Message operations
        ClientEvent::Message { chat_id, content, models } => {
            handle_message_with_llm(chat_id, content, models, state, user, out_tx, subscribed_chats).await?
        }
        ClientEvent::CreateMessage { chat_id, content, role, model, message_type, thread_id, reply_to_id } => {
            handle_create_message(chat_id, content, role, model, message_type, thread_id, reply_to_id, state, user, out_tx, subscribed_chats).await?
        }
        ClientEvent::UpdateMessage { chat_id, message_id, content } => {
            handle_update_message(chat_id, message_id, content, state, user, out_tx).await?
        }
        ClientEvent::DeleteMessage { chat_id, message_id } => {
            handle_delete_message(chat_id, message_id, state, user, out_tx).await?
        }
        ClientEvent::GetMessages { chat_id } => {
            handle_get_messages(chat_id, state, user, out_tx).await?
        }
        ClientEvent::GetMessageEdits { chat_id, message_id } => {
            handle_get_message_edits(chat_id, message_id, state, user, out_tx).await?
        }

        // Invite operations
        ClientEvent::CreateInvite { chat_id, email } => {
            handle_create_invite(chat_id, email, state, user, out_tx).await?
        }
        ClientEvent::ListInvites { chat_id } => {
            handle_list_invites(chat_id, state, user, out_tx).await?
        }
        ClientEvent::AcceptInvite { invite_id } => {
            handle_accept_invite(invite_id, state, user, out_tx).await?
        }
        ClientEvent::RejectInvite { invite_id } => {
            handle_reject_invite(invite_id, state, user, out_tx).await?
        }

        // Member operations
        ClientEvent::ListMembers { chat_id } => {
            handle_list_members(chat_id, state, user, out_tx).await?
        }
        ClientEvent::UpdateMemberRole { chat_id, member_user_id, role } => {
            handle_update_member_role(chat_id, member_user_id, role, state, user, out_tx).await?
        }
        ClientEvent::RemoveMember { chat_id, member_user_id } => {
            handle_remove_member(chat_id, member_user_id, state, user, out_tx).await?
        }

        // Folder operations
        ClientEvent::CreateFolder { name, color, parent_id } => {
            handle_create_folder(name, color, parent_id, state, user, out_tx).await?
        }
        ClientEvent::UpdateFolder { folder_id, name, color, collapsed } => {
            handle_update_folder(folder_id, name, color, collapsed, state, user, out_tx).await?
        }
        ClientEvent::DeleteFolder { folder_id } => {
            handle_delete_folder(folder_id, state, user, out_tx).await?
        }
        ClientEvent::GetFolders => {
            handle_get_folders(state, user, out_tx).await?
        }

        // Notification operations
        ClientEvent::GetNotifications { unread_only, limit, offset } => {
            handle_get_notifications(unread_only, limit, offset, state, user, out_tx).await?
        }
        ClientEvent::MarkNotificationRead { notification_id, read } => {
            handle_mark_notification_read(notification_id, read, state, user, out_tx).await?
        }
        ClientEvent::MarkAllNotificationsRead => {
            handle_mark_all_notifications_read(state, user, out_tx).await?
        }
        ClientEvent::DeleteNotification { notification_id } => {
            handle_delete_notification(notification_id, state, user, out_tx).await?
        }
        ClientEvent::GetUnreadCount => {
            handle_get_unread_count(state, user, out_tx).await?
        }

        // Permission operations
        ClientEvent::GetUserPermissions { user_id, resource_type, resource_id } => {
            handle_get_user_permissions(user_id, resource_type, resource_id, state, user, out_tx).await?
        }
        ClientEvent::GetResourcePermissions { resource_type, resource_id } => {
            handle_get_resource_permissions(resource_type, resource_id, state, user, out_tx).await?
        }
        ClientEvent::GrantPermission { resource_type, resource_id, user_id, permission_level } => {
            handle_grant_permission(resource_type, resource_id, user_id, permission_level, state, user, out_tx).await?
        }
        ClientEvent::RevokePermission { resource_type, resource_id, user_id } => {
            handle_revoke_permission(resource_type, resource_id, user_id, state, user, out_tx).await?
        }
        ClientEvent::CheckPermission { resource_type, resource_id, permission_level } => {
            handle_check_permission(resource_type, resource_id, permission_level, state, user, out_tx).await?
        }

        // Real-time events
        ClientEvent::Typing { chat_id, is_typing } => {
            handle_typing(chat_id, is_typing, state, user, out_tx, subscribed_chats).await?
        }
    }

    Ok(())
}

// WebSocket connection management
async fn handle_subscribe(
    chat_id: &str,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
    subscribed_chats: &mut HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if user is a member using service layer
    let chat_db_id = message::check_chat_membership(state.db_pool(), chat_id, user.id).await
        .map_err(|e| format!("Chat membership check failed: {}", e))?;

    // Get or create broadcaster for this chat
    let broadcaster = {
        let mut broadcasters = state.chat_broadcasters.lock().await;
        broadcasters
            .entry(chat_id.to_string())
            .or_insert_with(|| tokio::sync::broadcast::channel(100).0)
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

    subscribed_chats.insert(chat_id.to_string(), (chat_db_id, broadcaster));
    let response = ServerEvent::Subscribed { chat_id: chat_id.to_string() };
    out_tx.send(response).await?;
    Ok(())
}

async fn handle_unsubscribe(
    chat_id: &str,
    out_tx: &mpsc::Sender<ServerEvent>,
    subscribed_chats: &mut HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    subscribed_chats.remove(chat_id);
    let response = ServerEvent::Unsubscribed { chat_id: chat_id.to_string() };
    out_tx.send(response).await?;
    Ok(())
}

// Chat operations
async fn handle_create_chat(
    title: String,
    chat_type: String,
    folder_id: Option<String>,
    messages: Vec<CreateMessageRequest>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let create_req = crate::routes::models::CreateChatRequest {
        title,
        chat_type,
        folder_id,
        messages: messages.into_iter().map(|msg| crate::routes::models::ChatMessage {
            role: msg.role,
            content: msg.content,
            model: msg.model,
            usage: None,
            reasoning: None,
        }).collect(),
    };

    let chat = chat::create_chat(state.db_pool(), user.id, create_req).await
        .map_err(|e| format!("Failed to create chat: {}", e))?;

    let event = ServerEvent::ChatCreated { chat: chat.clone() };
    state.broadcast_to_user(user.id, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_update_chat(
    chat_id: String,
    title: Option<String>,
    folder_id: Option<String>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let update_req = UpdateChatRequest {
        title,
        folder_id,
        messages: None,
    };

    let (chat, member_ids) = chat::update_chat(state.db_pool(), &chat_id, user.id, update_req).await
        .map_err(|e| format!("Failed to update chat: {}", e))?;

    let event = ServerEvent::ChatUpdated { chat: chat.clone() };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_delete_chat(
    chat_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let member_ids = chat::delete_chat(state.db_pool(), &chat_id, user.id).await
        .map_err(|e| format!("Failed to delete chat: {}", e))?;

    let event = ServerEvent::ChatDeleted { chat_id: chat_id.clone() };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;

    {
        let mut broadcasters = state.chat_broadcasters.lock().await;
        broadcasters.remove(&chat_id);
    }
    Ok(())
}

async fn handle_get_chats(
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chats = chat::list_chats(state.db_pool(), user.id).await
        .map_err(|e| format!("Failed to get chats: {}", e))?;

    let response = ServerEvent::ChatsResponse { chats };
    out_tx.send(response).await?;
    Ok(())
}

async fn handle_get_chat(
    chat_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat = chat::get_chat(state.db_pool(), &chat_id, user.id).await
        .map_err(|e| format!("Failed to get chat: {}", e))?;

    let response = ServerEvent::ChatResponse { chat };
    out_tx.send(response).await?;
    Ok(())
}

// Message operations
async fn handle_create_message(
    chat_id: String,
    content: String,
    role: String,
    model: Option<String>,
    message_type: Option<String>,
    thread_id: Option<String>,
    reply_to_id: Option<String>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
    _subscribed_chats: &HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let create_req = CreateMessageRequest {
        content,
        role,
        model,
        message_type,
        thread_id,
        reply_to_id,
    };

    let (message, member_ids) = message::create_message(
        state.db_pool(),
        &chat_id,
        user.id,
        create_req,
    ).await
        .map_err(|e| format!("Failed to create message: {}", e))?;

    let event = ServerEvent::Message {
        chat_id: chat_id.clone(),
        message_id: message.public_id.clone(),
        user_id: message.user_id,
        content: message.content.clone(),
        model: message.model.clone(),
        timestamp: message.created_at.clone(),
        message_type: message.message_type.clone(),
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_update_message(
    chat_id: String,
    message_id: String,
    content: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (message, member_ids) = message::update_message(
        state.db_pool(),
        &chat_id,
        &message_id,
        user.id,
        content,
    ).await
        .map_err(|e| format!("Failed to update message: {}", e))?;

    let event = ServerEvent::MessageUpdated {
        chat_id: chat_id.clone(),
        message: message.clone(),
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_delete_message(
    chat_id: String,
    message_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (member_ids, message_id) = message::delete_message(
        state.db_pool(),
        &chat_id,
        &message_id,
        user.id,
    ).await
        .map_err(|e| format!("Failed to delete message: {}", e))?;

    let event = ServerEvent::MessageDeleted {
        chat_id: chat_id.clone(),
        message_id: message_id.clone(),
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_get_messages(
    chat_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let messages = message::get_messages(state.db_pool(), &chat_id, user.id).await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let response = ServerEvent::MessagesResponse { messages };
    out_tx.send(response).await?;
    Ok(())
}

async fn handle_get_message_edits(
    chat_id: String,
    message_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let edits = message::get_message_edits(state.db_pool(), &chat_id, &message_id, user.id).await
        .map_err(|e| format!("Failed to get message edits: {}", e))?;

    let response = ServerEvent::MessageEditsResponse { edits };
    out_tx.send(response).await?;
    Ok(())
}

// Special handler for Message event with LLM processing
async fn handle_message_with_llm(
    chat_id: String,
    content: String,
    models: Vec<String>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
    subscribed_chats: &HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create user message using service layer
    let create_req = CreateMessageRequest {
        content: content.clone(),
        role: "user".to_string(),
        model: None,
        message_type: Some("text".to_string()),
        thread_id: None,
        reply_to_id: None,
    };

    let (message, member_ids) = message::create_message(
        state.db_pool(),
        &chat_id,
        user.id,
        create_req,
    ).await
        .map_err(|e| format!("Failed to create message: {}", e))?;

    let message_event = ServerEvent::Message {
        chat_id: chat_id.clone(),
        message_id: message.public_id.clone(),
        user_id: message.user_id,
        content: message.content.clone(),
        model: message.model.clone(),
        timestamp: message.created_at.clone(),
        message_type: message.message_type.clone(),
    };

    state.broadcast_to_chat(&chat_id, &message_event).await;
    state.broadcast_to_users(member_ids, &message_event).await;
    out_tx.send(message_event).await?;

    // LLM processing (existing logic)
    let (_chat_db_id, broadcaster) = match subscribed_chats.get(&chat_id) {
        Some((id, sender)) => (*id, sender.clone()),
        None => {
            return Err("Not subscribed to chat".into());
        }
    };

    let mut requested_models: Vec<String> = models
        .into_iter()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .collect();

    if requested_models.is_empty() {
        if let Some(active) = state.orchestrator().active_model() {
            requested_models.push(active);
        }
    }

    let models_to_use: Vec<String> = requested_models;

    if models_to_use.is_empty() {
        let error_event = ServerEvent::Error {
            message: "No model configured".to_string(),
        };
        out_tx.send(error_event).await?;
        return Ok(());
    }

    for model_to_use in models_to_use {
        let state_clone = state.clone();
        let chat_id_clone = chat_id.clone();
        let content_clone = content.clone();
        let out_tx_clone = out_tx.clone();
        let broadcaster_clone = broadcaster.clone();
        let user_id = user.id;

        tokio::spawn(async move {
            let provider = match state_clone.orchestrator().provider_for_model(&model_to_use) {
                Ok(provider) => provider,
                Err(e) => {
                    let error_event = ServerEvent::Error {
                        message: format!("LLM provider not available for {}: {}", model_to_use, e),
                    };
                    let _ = out_tx_clone.send(error_event).await;
                    return;
                }
            };

            let message = denkwerk::ChatMessage::user(&content_clone);
            let request = denkwerk::CompletionRequest::new(model_to_use.clone(), vec![message]);

            match provider.complete(request).await {
                Ok(completion) => {
                    let response_content = completion.message.text().unwrap_or_default().to_string();
                    let _assistant_message_id = cuid2::create_id();
                    let assistant_timestamp = chrono::Utc::now().to_rfc3339();

                    // Create assistant message request
                    let assistant_create_req = CreateMessageRequest {
                        content: response_content.clone(),
                        role: "assistant".to_string(),
                        model: Some(model_to_use.clone()),
                        message_type: Some("text".to_string()),
                        thread_id: None,
                        reply_to_id: None,
                    };

                    if let Ok((assistant_message, _)) = message::create_message(
                        &state_clone.db_pool(),
                        &chat_id_clone,
                        user_id,
                        assistant_create_req,
                    ).await {
                        let assistant_event = ServerEvent::Message {
                            chat_id: chat_id_clone.clone(),
                            message_id: assistant_message.public_id.clone(),
                            user_id: assistant_message.user_id,
                            content: response_content,
                            model: Some(model_to_use.clone()),
                            timestamp: assistant_timestamp,
                            message_type: "text".to_string(),
                        };

                        let _ = out_tx_clone.send(assistant_event.clone()).await;
                        let _ = broadcaster_clone.send(assistant_event);
                    }
                }
                Err(e) => {
                    let error_event = ServerEvent::Error {
                        message: format!("LLM completion failed: {}", e),
                    };
                    let _ = out_tx_clone.send(error_event).await;
                }
            }
        });
    }

    Ok(())
}

// Invite operations
async fn handle_create_invite(
    chat_id: String,
    email: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let create_req = crate::routes::models::CreateInviteRequest { email };

    let (invite, member_ids) = invite::create_invite(
        state.db_pool(),
        &chat_id,
        user.id,
        create_req,
    ).await
        .map_err(|e| format!("Failed to create invite: {}", e))?;

    let event = ServerEvent::InviteCreated {
        chat_id: chat_id.clone(),
        invite: invite.clone(),
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_list_invites(
    chat_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let invites = invite::list_invites(state.db_pool(), &chat_id, user.id).await
        .map_err(|e| format!("Failed to list invites: {}", e))?;

    let response = ServerEvent::InvitesResponse { invites };
    out_tx.send(response).await?;
    Ok(())
}

async fn handle_accept_invite(
    invite_id: i64,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (member, member_ids, chat_public_id) = invite::accept_invite(
        state.db_pool(),
        invite_id,
        user.id,
        user.email.clone(),
    ).await
        .map_err(|e| format!("Failed to accept invite: {}", e))?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_public_id.clone(),
        member: member.clone(),
    };

    state.broadcast_to_chat(&chat_public_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_reject_invite(
    invite_id: i64,
    state: &AppState,
    user: &switchboard_auth::User,
    _out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    invite::reject_invite(
        state.db_pool(),
        invite_id,
        user.email.clone(),
    ).await
        .map_err(|e| format!("Failed to reject invite: {}", e))?;

    Ok(())
}

// Member operations
async fn handle_list_members(
    chat_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let members = member::list_members(state.db_pool(), &chat_id, user.id).await
        .map_err(|e| format!("Failed to list members: {}", e))?;

    let response = ServerEvent::MembersResponse { members };
    out_tx.send(response).await?;
    Ok(())
}

async fn handle_update_member_role(
    chat_id: String,
    member_user_id: i64,
    role: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let update_req = UpdateMemberRoleRequest { role };

    let (member, member_ids) = member::update_member_role(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id,
        update_req,
    ).await
        .map_err(|e| format!("Failed to update member role: {}", e))?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_id.clone(),
        member: member.clone(),
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_remove_member(
    chat_id: String,
    member_user_id: i64,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let member_ids = member::remove_member(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id,
    ).await
        .map_err(|e| format!("Failed to remove member: {}", e))?;

    let event = ServerEvent::MemberRemoved {
        chat_id: chat_id.clone(),
        user_id: member_user_id,
    };

    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

// Real-time events
async fn handle_typing(
    chat_id: String,
    is_typing: bool,
    _state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
    subscribed_chats: &HashMap<String, (i64, tokio::sync::broadcast::Sender<ServerEvent>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    out_tx.send(typing_event.clone()).await?;
    let _ = broadcaster.send(typing_event);
    Ok(())
}

// Folder operations
async fn handle_create_folder(
    name: String,
    color: Option<String>,
    parent_id: Option<String>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::routes::models::CreateFolderRequest;

    let req = CreateFolderRequest {
        name,
        color,
        parent_id,
    };

    let folder = folder::create_folder(state.db_pool(), user.id, req)
        .await
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    let event = ServerEvent::FolderCreated {
        folder: folder.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

    let response_event = ServerEvent::FolderResponse {
        folder,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_update_folder(
    folder_id: String,
    name: Option<String>,
    color: Option<String>,
    collapsed: Option<bool>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::routes::models::UpdateFolderRequest;

    let req = UpdateFolderRequest {
        name,
        color,
        collapsed,
    };

    let folder = folder::update_folder(state.db_pool(), user.id, &folder_id, req)
        .await
        .map_err(|e| format!("Failed to update folder: {}", e))?;

    let event = ServerEvent::FolderUpdated {
        folder: folder.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

    let response_event = ServerEvent::FolderResponse {
        folder,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_delete_folder(
    folder_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    folder::delete_folder(state.db_pool(), user.id, &folder_id)
        .await
        .map_err(|e| format!("Failed to delete folder: {}", e))?;

    let event = ServerEvent::FolderDeleted {
        folder_id: folder_id.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

    out_tx.send(event).await?;
    Ok(())
}

async fn handle_get_folders(
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let folders = folder::list_folders(state.db_pool(), user.id)
        .await
        .map_err(|e| format!("Failed to get folders: {}", e))?;

    let response_event = ServerEvent::FoldersResponse {
        folders,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

// Notification operations
async fn handle_get_notifications(
    unread_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let unread_only = unread_only.unwrap_or(false);
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let notifications = notification::list_notifications(
        state.db_pool(),
        user.id,
        unread_only,
        limit,
        offset,
    )
    .await
    .map_err(|e| format!("Failed to get notifications: {}", e))?;

    let response_event = ServerEvent::NotificationsResponse {
        notifications,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_mark_notification_read(
    notification_id: i64,
    read: bool,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::routes::models::MarkNotificationReadRequest;

    let req = MarkNotificationReadRequest { read };
    let notification = notification::mark_notification_read(
        state.db_pool(),
        user.id,
        notification_id,
        req,
    )
    .await
    .map_err(|e| format!("Failed to mark notification read: {}", e))?;

    // Broadcast the update to the user
    let event = ServerEvent::NotificationUpdated {
        notification: notification.clone(),
    };
    state.broadcast_to_user(user.id, &event).await;

    // Send direct response
    let response_event = ServerEvent::NotificationResponse {
        notification,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_mark_all_notifications_read(
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let updated_count = notification::mark_all_read(state.db_pool(), user.id)
        .await
        .map_err(|e| format!("Failed to mark all notifications read: {}", e))?;

    let response_event = ServerEvent::NotificationUpdated {
        notification: crate::routes::models::Notification {
            id: 0,
            user_id: user.id,
            r#type: "bulk_update".to_string(),
            title: format!("{} notifications marked as read", updated_count),
            body: "Bulk update completed".to_string(),
            read: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    };
    state.broadcast_to_user(user.id, &response_event).await;
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_delete_notification(
    notification_id: i64,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    notification::delete_notification(state.db_pool(), user.id, notification_id)
        .await
        .map_err(|e| format!("Failed to delete notification: {}", e))?;

    let event = ServerEvent::NotificationDeleted {
        notification_id,
    };
    state.broadcast_to_user(user.id, &event).await;
    out_tx.send(event).await?;
    Ok(())
}

async fn handle_get_unread_count(
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let unread_count = notification::get_unread_count(state.db_pool(), user.id)
        .await
        .map_err(|e| format!("Failed to get unread count: {}", e))?;

    let response_event = ServerEvent::UnreadCountResponse {
        unread_count,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

// Permission operations
async fn handle_get_user_permissions(
    user_id: String,
    resource_type: Option<String>,
    resource_id: Option<String>,
    state: &AppState,
    current_user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Resolve target user ID
    let target_user_id = permission::resolve_user_id(state.db_pool(), &user_id)
        .await
        .map_err(|e| format!("Failed to resolve user ID: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    // Users can only see their own permissions unless they're admin
    if current_user.id != target_user_id {
        return Err("Cannot view other users' permissions".into());
    }

    let resource_db_id = if let (Some(rt), Some(rid)) = (&resource_type, resource_id) {
        Some(permission::resolve_resource_id(state.db_pool(), rt, &rid)
            .await
            .map_err(|e| format!("Failed to resolve resource ID: {}", e))?
            .ok_or_else(|| "Resource not found".to_string())?)
    } else {
        None
    };

    let permissions = permission::get_user_permissions(
        state.db_pool(),
        target_user_id,
        resource_type.as_deref(),
        resource_db_id,
    )
    .await
    .map_err(|e| format!("Failed to get user permissions: {}", e))?;

    let response_event = ServerEvent::PermissionsResponse {
        permissions,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_get_resource_permissions(
    resource_type: String,
    resource_id: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Resolve resource ID from public ID
    let resource_db_id = permission::resolve_resource_id(state.db_pool(), &resource_type, &resource_id)
        .await
        .map_err(|e| format!("Failed to resolve resource ID: {}", e))?
        .ok_or_else(|| "Resource not found".to_string())?;

    // Check if user has admin permission for this resource
    if !permission::check_permission(
        state.db_pool(),
        user.id,
        &resource_type,
        resource_db_id,
        "admin",
    )
    .await
    .map_err(|e| format!("Failed to check admin permission: {}", e))?
    {
        return Err("Admin permission required".into());
    }

    let permissions = permission::get_resource_permissions(
        state.db_pool(),
        &resource_type,
        resource_db_id,
    )
    .await
    .map_err(|e| format!("Failed to get resource permissions: {}", e))?;

    let response_event = ServerEvent::PermissionsResponse {
        permissions,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_grant_permission(
    resource_type: String,
    resource_id: String,
    user_id: String,
    permission_level: String,
    state: &AppState,
    current_user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Resolve resource ID from public ID
    let resource_db_id = permission::resolve_resource_id(state.db_pool(), &resource_type, &resource_id)
        .await
        .map_err(|e| format!("Failed to resolve resource ID: {}", e))?
        .ok_or_else(|| "Resource not found".to_string())?;

    // Check if user has admin permission for this resource
    if !permission::check_permission(
        state.db_pool(),
        current_user.id,
        &resource_type,
        resource_db_id,
        "admin",
    )
    .await
    .map_err(|e| format!("Failed to check admin permission: {}", e))?
    {
        return Err("Admin permission required".into());
    }

    // Resolve target user ID
    let target_user_id = permission::resolve_user_id(state.db_pool(), &user_id)
        .await
        .map_err(|e| format!("Failed to resolve target user ID: {}", e))?
        .ok_or_else(|| "Target user not found".to_string())?;

    // Grant the permission
    let permission = permission::grant_permission(
        state.db_pool(),
        target_user_id,
        &resource_type,
        resource_db_id,
        &permission_level,
        current_user.id,
    )
    .await
    .map_err(|e| format!("Failed to grant permission: {}", e))?;

    // Broadcast the permission granted event
    let event = ServerEvent::PermissionGranted {
        permission: permission.clone(),
    };
    state.broadcast_to_user(current_user.id, &event).await;

    // Send direct response
    let response_event = ServerEvent::PermissionResponse {
        permission,
    };
    out_tx.send(response_event).await?;
    Ok(())
}

async fn handle_revoke_permission(
    resource_type: String,
    resource_id: String,
    user_id: String,
    state: &AppState,
    current_user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Resolve resource ID from public ID
    let resource_db_id = permission::resolve_resource_id(state.db_pool(), &resource_type, &resource_id)
        .await
        .map_err(|e| format!("Failed to resolve resource ID: {}", e))?
        .ok_or_else(|| "Resource not found".to_string())?;

    // Check if user has admin permission for this resource
    if !permission::check_permission(
        state.db_pool(),
        current_user.id,
        &resource_type,
        resource_db_id,
        "admin",
    )
    .await
    .map_err(|e| format!("Failed to check admin permission: {}", e))?
    {
        return Err("Admin permission required".into());
    }

    // Resolve target user ID
    let target_user_id = permission::resolve_user_id(state.db_pool(), &user_id)
        .await
        .map_err(|e| format!("Failed to resolve target user ID: {}", e))?
        .ok_or_else(|| "Target user not found".to_string())?;

    // Revoke the permission
    permission::revoke_permission(
        state.db_pool(),
        target_user_id,
        &resource_type,
        resource_db_id,
    )
    .await
    .map_err(|e| format!("Failed to revoke permission: {}", e))?;

    // Broadcast the permission revoked event
    let event = ServerEvent::PermissionRevoked {
        resource_type,
        resource_id: resource_db_id,
        user_id: target_user_id,
    };
    state.broadcast_to_user(current_user.id, &event).await;

    out_tx.send(event).await?;
    Ok(())
}

async fn handle_check_permission(
    resource_type: String,
    resource_id: String,
    permission_level: String,
    state: &AppState,
    user: &switchboard_auth::User,
    out_tx: &mpsc::Sender<ServerEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Resolve resource ID from public ID
    let resource_db_id = permission::resolve_resource_id(state.db_pool(), &resource_type, &resource_id)
        .await
        .map_err(|e| format!("Failed to resolve resource ID: {}", e))?
        .ok_or_else(|| "Resource not found".to_string())?;

    let has_permission = permission::check_permission(
        state.db_pool(),
        user.id,
        &resource_type,
        resource_db_id,
        &permission_level,
    )
    .await
    .map_err(|e| format!("Failed to check permission: {}", e))?;

    let response_event = ServerEvent::PermissionCheckResponse {
        has_permission,
    };
    out_tx.send(response_event).await?;
    Ok(())
}