use axum::{
    extract::{Multipart, State},
    http::HeaderMap,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use denkwerk::{ChatMessage, CompletionRequest, TokenUsage as ProviderTokenUsage};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{util::require_bearer, ApiError, AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub model: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<crate::routes::models::TokenUsage>)]
    pub usage: Option<ProviderTokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Vec<String>>,
}

#[derive(Debug, ToSchema)]
pub struct ChatCompletionForm {
    /// Natural language prompt content.
    pub prompt: String,
    /// Optional model identifier. Defaults to the server configured model.
    #[schema(nullable)]
    pub model: Option<String>,
    /// Optional image attachments encoded as data URLs or binary uploads.
    #[schema(nullable, value_type = Vec<String>)]
    pub images: Option<Vec<String>>,
}

#[utoipa::path(
    post,
    path = "/api/chat",
    tag = "Chat",
    security(("bearerAuth" = [])),
    request_body(
        content = ChatCompletionForm,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "LLM chat completion", body = ChatCompletionResponse),
        (status = 400, description = "Invalid request payload", body = crate::error::ErrorResponse),
        (status = 401, description = "Authentication required", body = crate::error::ErrorResponse),
        (status = 500, description = "Provider error", body = crate::error::ErrorResponse)
    )
)]
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
