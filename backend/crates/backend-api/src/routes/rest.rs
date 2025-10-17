use axum::{
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use denkwerk::{ChatMessage, CompletionRequest, TokenUsage as ProviderTokenUsage};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    routes::{
        auth::{GithubCallbackRequest, GithubLoginQuery, GithubLoginResponse, SessionResponse},
        chat::{ChatCompletionForm, ChatCompletionResponse},
        chats::{ChatDetailResponse, ChatsResponse},
        models::{
            Chat, ChatInvite, CreateChatRequest, CreateInviteRequest,
            CreateMessageRequest, InviteResponse, InvitesResponse, MemberResponse, MembersResponse,
            Message, MessageEditsResponse, MessageResponse, MessagesResponse, UpdateChatRequest,
            UpdateMemberRoleRequest, UpdateMessageRequest,
        },
    },
    state::ServerEvent,
    util::require_bearer,
    ApiError, AppState,
};

// Import all service modules
use crate::services::{auth, chat, invite, member, message};

// ===== AUTH REST ENDPOINTS =====

pub async fn github_login(
    State(state): State<AppState>,
    Query(params): Query<GithubLoginQuery>,
) -> Result<Json<GithubLoginResponse>, ApiError> {
    let authorize_url = auth::github_login_url(
        state.authenticator(),
        state.oauth_state(),
        params.redirect_uri,
    ).await?;

    Ok(Json(GithubLoginResponse { authorize_url }))
}

pub async fn github_callback(
    State(state): State<AppState>,
    Json(payload): Json<GithubCallbackRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let (session, user) = auth::github_callback(
        state.authenticator(),
        state.oauth_state(),
        payload.code,
        payload.state,
        payload.redirect_uri,
    ).await?;

    let response = auth::create_session_response(session, user);
    Ok(Json(response))
}

#[cfg(debug_assertions)]
pub async fn dev_token(State(state): State<AppState>) -> Result<Json<SessionResponse>, ApiError> {
    let (session, user) = auth::create_dev_token(state.db_pool()).await?;
    let response = auth::create_session_response(session, user);
    Ok(Json(response))
}

// ===== CHAT REST ENDPOINTS =====

pub async fn list_chats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChatsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chats = chat::list_chats(state.db_pool(), user.id).await?;

    Ok(Json(crate::routes::chats::ChatsResponse { chats }))
}

pub async fn create_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateChatRequest>,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chat = chat::create_chat(state.db_pool(), user.id, req).await?;

    let event = ServerEvent::ChatCreated { chat: chat.clone() };
    state.broadcast_to_user(user.id, &event).await;

    Ok(Json(ChatDetailResponse { chat }))
}

pub async fn get_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let chat = chat::get_chat(state.db_pool(), &chat_id, user.id).await?;

    Ok(Json(ChatDetailResponse { chat }))
}

pub async fn update_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateChatRequest>,
) -> Result<Json<ChatDetailResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (chat, member_ids) = chat::update_chat(state.db_pool(), &chat_id, user.id, req).await?;

    let event = ServerEvent::ChatUpdated { chat: chat.clone() };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(ChatDetailResponse { chat }))
}

pub async fn delete_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let member_ids = chat::delete_chat(state.db_pool(), &chat_id, user.id).await?;

    let event = ServerEvent::ChatDeleted {
        chat_id: chat_id.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    {
        let mut broadcasters = state.chat_broadcasters.lock().await;
        broadcasters.remove(&chat_id);
    }

    Ok(())
}

// ===== MESSAGE REST ENDPOINTS =====

pub async fn get_messages(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<MessagesResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let messages = message::get_messages(state.db_pool(), &chat_id, user.id).await?;

    Ok(Json(MessagesResponse { messages }))
}

pub async fn create_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (message, member_ids) = message::create_message(
        state.db_pool(),
        &chat_id,
        user.id,
        req,
    ).await?;

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

    Ok(Json(MessageResponse { message }))
}

pub async fn update_message(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (message, member_ids) = message::update_message(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id,
        req.content,
    ).await?;

    let event = ServerEvent::MessageUpdated {
        chat_id: chat_id.clone(),
        message: message.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(MessageResponse { message }))
}

pub async fn delete_message(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (member_ids, message_id) = message::delete_message(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id,
    ).await?;

    let event = ServerEvent::MessageDeleted {
        chat_id: chat_id.clone(),
        message_id: message_id.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}

pub async fn get_message_edits(
    State(state): State<AppState>,
    Path((chat_id, message_public_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<MessageEditsResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let edits = message::get_message_edits(
        state.db_pool(),
        &chat_id,
        &message_public_id,
        user.id,
    ).await?;

    Ok(Json(MessageEditsResponse { edits }))
}

// ===== INVITE REST ENDPOINTS =====

pub async fn create_invite(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (invite, member_ids) = invite::create_invite(
        state.db_pool(),
        &chat_id,
        user.id,
        req,
    ).await?;

    let event = ServerEvent::InviteCreated {
        chat_id: chat_id.clone(),
        invite: invite.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(InviteResponse { invite }))
}

pub async fn list_invites(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<InvitesResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let invites = invite::list_invites(state.db_pool(), &chat_id, user.id).await?;

    Ok(Json(InvitesResponse { invites }))
}

pub async fn accept_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (member, member_ids, chat_public_id) = invite::accept_invite(
        state.db_pool(),
        invite_id,
        user.id,
        user.email.clone(),
    ).await?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_public_id.clone(),
        member: member.clone(),
    };
    state.broadcast_to_chat(&chat_public_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}

pub async fn reject_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    invite::reject_invite(
        state.db_pool(),
        invite_id,
        user.email.clone(),
    ).await?;

    Ok(())
}

// ===== MEMBER REST ENDPOINTS =====

pub async fn list_members(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<MembersResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let members = member::list_members(state.db_pool(), &chat_id, user.id).await?;

    Ok(Json(MembersResponse { members }))
}

pub async fn update_member_role(
    State(state): State<AppState>,
    Path((chat_id, member_user_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<MemberResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let (member, member_ids) = member::update_member_role(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id,
        req,
    ).await?;

    let event = ServerEvent::MemberUpdated {
        chat_id: chat_id.clone(),
        member: member.clone(),
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(Json(MemberResponse { member }))
}

pub async fn remove_member(
    State(state): State<AppState>,
    Path((chat_id, member_user_id)): Path<(String, i64)>,
    headers: HeaderMap,
) -> Result<(), ApiError> {
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    let member_ids = member::remove_member(
        state.db_pool(),
        &chat_id,
        user.id,
        member_user_id,
    ).await?;

    let event = ServerEvent::MemberRemoved {
        chat_id: chat_id.clone(),
        user_id: member_user_id,
    };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    Ok(())
}

// ===== CHAT COMPLETION REST ENDPOINT =====

pub async fn chat_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ChatCompletionResponse>, ApiError> {
    let token = require_bearer(&headers)?;
    let _ = state.authenticate(&token).await?;

    let mut prompt = None;
    let mut model_field = None;
    let mut images: Vec<Bytes> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("invalid multipart"))?
    {
        let name = field.name().unwrap_or("");
        match name {
            "prompt" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| ApiError::bad_request("invalid prompt"))?;
                prompt = Some(text);
            }
            "model" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| ApiError::bad_request("invalid model"))?;
                model_field = Some(text);
            }
            "images" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| ApiError::bad_request("invalid image"))?;
                images.push(data);
            }
            _ => {}
        }
    }

    let prompt = prompt.ok_or_else(|| ApiError::bad_request("prompt is required"))?;
    let prompt_trimmed = prompt.trim();
    if prompt_trimmed.is_empty() && images.is_empty() {
        return Err(ApiError::bad_request("prompt or images are required"));
    }

    let model = model_field
        .filter(|value| !value.trim().is_empty())
        .or_else(|| state.orchestrator().active_model())
        .ok_or_else(|| {
            ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "no active model configured",
            )
        })?;

    let provider = state.orchestrator().provider_for_model(&model)?;

    let message = if images.is_empty() {
        ChatMessage::user(prompt_trimmed)
    } else {
        let image_parts: Vec<String> = images
            .iter()
            .map(|image| {
                format!(
                    "data:image/png;base64,{}",
                    general_purpose::STANDARD.encode(image.as_ref())
                )
            })
            .collect();
        let full_content = format!("{} {}", prompt_trimmed, image_parts.join(" "));
        ChatMessage::user(full_content)
    };

    let request = CompletionRequest::new(model.clone(), vec![message]);
    let completion = provider.complete(request).await?;

    let content = completion.message.text().unwrap_or_default().to_string();
    let reasoning = completion
        .reasoning
        .map(|steps| steps.into_iter().map(|step| step.content).collect());

    Ok(Json(ChatCompletionResponse {
        model,
        content,
        usage: completion.usage,
        reasoning,
    }))
}