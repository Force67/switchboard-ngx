use std::sync::Arc;

use axum::{
    extract::{Extension, Multipart, State},
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use denkwerk::{ChatMessage, CompletionRequest, TokenUsage as ProviderTokenUsage};
use serde::Serialize;
use tracing::warn;
use utoipa::ToSchema;

use crate::error::{GatewayError, GatewayResult};
use crate::state::GatewayState;

/// Routes for simple chat completion against the orchestrator.
pub fn create_chat_completion_routes() -> Router<Arc<GatewayState>> {
    Router::new().route("/chat", post(chat_completion))
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl From<ProviderTokenUsage> for TokenUsage {
    fn from(value: ProviderTokenUsage) -> Self {
        Self {
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub model: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
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
        (status = 400, description = "Invalid request payload", body = GatewayError),
        (status = 401, description = "Authentication required", body = GatewayError),
        (status = 503, description = "Provider unavailable", body = GatewayError),
        (status = 500, description = "Provider error", body = GatewayError)
    )
)]
pub async fn chat_completion(
    State(state): State<Arc<GatewayState>>,
    user: Option<Extension<i64>>,
    mut multipart: Multipart,
) -> GatewayResult<Json<ChatCompletionResponse>> {
    let _user_id = user
        .map(|Extension(id)| id)
        .ok_or_else(|| GatewayError::AuthenticationFailed("User not authenticated".to_string()))?;

    let mut prompt = None;
    let mut model_field = None;
    let mut images: Vec<Bytes> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| GatewayError::InvalidRequest("invalid multipart payload".to_string()))?
    {
        match field.name().unwrap_or("") {
            "prompt" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid prompt".to_string()))?;
                prompt = Some(text);
            }
            "model" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid model".to_string()))?;
                model_field = Some(text);
            }
            "images" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid image".to_string()))?;
                images.push(data);
            }
            _ => {}
        }
    }

    let prompt = prompt.ok_or_else(|| GatewayError::InvalidRequest("prompt is required".to_string()))?;
    let prompt_trimmed = prompt.trim();
    if prompt_trimmed.is_empty() && images.is_empty() {
        return Err(GatewayError::InvalidRequest(
            "prompt or images are required".to_string(),
        ));
    }

    let orchestrator = state
        .orchestrator()
        .ok_or_else(|| GatewayError::ServiceError("LLM orchestrator unavailable".to_string()))?;

    let model = model_field
        .filter(|value| !value.trim().is_empty())
        .or_else(|| orchestrator.active_model())
        .unwrap_or_else(|| "debug/echo".to_string());

    // Offline fallback that never hits a provider.
    if model == "debug/echo" {
        return Ok(Json(fallback_response(&model, prompt_trimmed, None)));
    }

    let provider = match orchestrator.provider_for_model(&model) {
        Ok(provider) => provider,
        Err(error) => {
            warn!(?error, %model, "LLM provider unavailable, returning fallback response");
            return Ok(Json(fallback_response(
                &model,
                prompt_trimmed,
                Some(error.to_string()),
            )));
        }
    };

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
    let completion = match provider.complete(request).await {
        Ok(value) => value,
        Err(error) => {
            warn!(?error, %model, "LLM completion failed, returning fallback response");
            return Ok(Json(fallback_response(
                &model,
                prompt_trimmed,
                Some(error.to_string()),
            )));
        }
    };

    let content = completion.message.text().unwrap_or_default().to_string();
    let reasoning = completion
        .reasoning
        .map(|steps| steps.into_iter().map(|step| step.content).collect());
    let usage = completion.usage.map(TokenUsage::from);

    Ok(Json(ChatCompletionResponse {
        model,
        content,
        usage,
        reasoning,
    }))
}

fn fallback_response(model: &str, prompt: &str, error: Option<String>) -> ChatCompletionResponse {
    let trimmed_prompt = prompt.trim();
    let preview = if trimmed_prompt.is_empty() {
        "".to_string()
    } else {
        format!(" Prompt: \"{}\"", trimmed_prompt.chars().take(200).collect::<String>())
    };

    let mut content = format!(
        "⚠️ LLM provider unavailable for model `{}`. Returning fallback response.",
        model
    );

    if let Some(err) = error {
        content.push_str(&format!(" Error: {}", err));
    }

    content.push_str(&preview);

    ChatCompletionResponse {
        model: model.to_string(),
        content,
        usage: None,
        reasoning: None,
    }
}
