use std::sync::Arc;

use axum::{
    extract::{Extension, Multipart, State},
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use denkwerk::{
    ChatMessage, CompletionRequest, FunctionCall, ToolCall, ToolChoice,
    TokenUsage as ProviderTokenUsage,
};
use serde::Serialize;
use tracing::{debug, info, warn};
use utoipa::ToSchema;

use crate::error::{GatewayError, GatewayResult};
use crate::services::{
    create_web_search_tool, SearchResult, WebSearchService, WEB_SEARCH_CONTENT_INJECTION_PROMPT,
    WEB_SEARCH_TOOL_SYSTEM_PROMPT,
};
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

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct WebSearchSource {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

impl From<SearchResult> for WebSearchSource {
    fn from(result: SearchResult) -> Self {
        Self {
            title: result.title,
            url: result.url,
            snippet: result.snippet,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_sources: Option<Vec<WebSearchSource>>,
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
    /// Enable web search grounding for this request.
    #[schema(nullable)]
    pub web_search: Option<bool>,
    /// Temperature for response generation (0.0 - 2.0).
    #[schema(nullable)]
    pub temperature: Option<f32>,
    /// Maximum tokens in the response.
    #[schema(nullable)]
    pub max_tokens: Option<u32>,
    /// Custom system prompt for this request.
    #[schema(nullable)]
    pub system_prompt: Option<String>,
}

/// Parsed form fields for chat completion.
struct ParsedChatRequest {
    prompt: String,
    model: Option<String>,
    images: Vec<Bytes>,
    web_search: bool,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    system_prompt: Option<String>,
}

async fn parse_multipart(mut multipart: Multipart) -> GatewayResult<ParsedChatRequest> {
    let mut prompt = None;
    let mut model = None;
    let mut images: Vec<Bytes> = Vec::new();
    let mut web_search = false;
    let mut temperature = None;
    let mut max_tokens = None;
    let mut system_prompt = None;

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
                model = Some(text);
            }
            "images" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid image".to_string()))?;
                images.push(data);
            }
            "web_search" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid web_search".to_string()))?;
                web_search = text.to_lowercase() == "true" || text == "1";
            }
            "temperature" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid temperature".to_string()))?;
                temperature = text.parse::<f32>().ok();
            }
            "max_tokens" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid max_tokens".to_string()))?;
                max_tokens = text.parse::<u32>().ok();
            }
            "system_prompt" => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| GatewayError::InvalidRequest("invalid system_prompt".to_string()))?;
                if !text.trim().is_empty() {
                    system_prompt = Some(text);
                }
            }
            _ => {}
        }
    }

    let prompt =
        prompt.ok_or_else(|| GatewayError::InvalidRequest("prompt is required".to_string()))?;

    Ok(ParsedChatRequest {
        prompt,
        model,
        images,
        web_search,
        temperature,
        max_tokens,
        system_prompt,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/chat",
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
    multipart: Multipart,
) -> GatewayResult<Json<ChatCompletionResponse>> {
    let _user_id = user
        .map(|Extension(id)| id)
        .ok_or_else(|| GatewayError::AuthenticationFailed("User not authenticated".to_string()))?;

    let request = parse_multipart(multipart).await?;
    let prompt_trimmed = request.prompt.trim();

    if prompt_trimmed.is_empty() && request.images.is_empty() {
        return Err(GatewayError::InvalidRequest(
            "prompt or images are required".to_string(),
        ));
    }

    let orchestrator = state
        .orchestrator()
        .ok_or_else(|| GatewayError::ServiceError("LLM orchestrator unavailable".to_string()))?;

    let model = request
        .model
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

    // Get web search configuration and check if model supports tools
    let web_search_config = state.web_search_config();
    let web_search_enabled = request.web_search && web_search_config.is_some();

    // Determine if model supports function calling (simplified check based on common patterns)
    let model_supports_tools = model_supports_function_calling(&model);

    // Build messages
    let mut messages = Vec::new();
    let mut web_search_sources: Option<Vec<WebSearchSource>> = None;

    // Add system prompt if provided
    let mut system_prompt_content = String::new();

    if let Some(ref custom_prompt) = request.system_prompt {
        system_prompt_content.push_str(custom_prompt);
    }

    // Handle web search
    if web_search_enabled {
        let config = web_search_config.unwrap();
        let search_service = match WebSearchService::new(config.clone()) {
            Ok(service) if service.is_configured() => Some(service),
            Ok(_) => {
                warn!("web search requested but provider is not fully configured");
                None
            }
            Err(e) => {
                warn!(?e, "failed to initialize web search service");
                None
            }
        };

        if let Some(search_service) = search_service {
            if model_supports_tools {
                // Function calling mode: add the tool and let the model decide to search
                debug!(model = %model, "using function calling mode for web search");

                // Add web search tool instructions to system prompt
                if !system_prompt_content.is_empty() {
                    system_prompt_content.push_str("\n\n");
                }
                system_prompt_content.push_str(WEB_SEARCH_TOOL_SYSTEM_PROMPT);

                if !system_prompt_content.is_empty() {
                    messages.push(ChatMessage::system(&system_prompt_content));
                }

                // Add user message
                let user_message = build_user_message(prompt_trimmed, &request.images);
                messages.push(user_message);

                // Build request with tools
                let mut completion_request = CompletionRequest::new(model.clone(), messages.clone())
                    .with_tool(create_web_search_tool())
                    .with_tool_choice(ToolChoice::auto());

                if let Some(temp) = request.temperature {
                    completion_request = completion_request.with_temperature(temp);
                }
                if let Some(max) = request.max_tokens {
                    completion_request = completion_request.with_max_tokens(max);
                }

                // Execute the completion
                let completion = match provider.complete(completion_request).await {
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

                // Check if the model wants to call the web_search tool
                if !completion.message.tool_calls.is_empty() {
                    let tool_call = &completion.message.tool_calls[0];
                    if tool_call.function.name == "web_search" {
                        // Extract query from arguments
                        let query = tool_call
                            .function
                            .arguments
                            .get("query")
                            .and_then(|v| v.as_str())
                            .unwrap_or(prompt_trimmed);

                        info!(query = %query, "executing web search via function call");

                        // Execute the search
                        match search_service.search(query).await {
                            Ok(search_response) => {
                                // Store sources for the response
                                web_search_sources = Some(
                                    search_response
                                        .results
                                        .iter()
                                        .cloned()
                                        .map(WebSearchSource::from)
                                        .collect(),
                                );

                                // Add the tool call and result to messages
                                messages
                                    .push(completion.message.clone().with_tool_calls(vec![ToolCall::new(
                                        FunctionCall::new(
                                            "web_search",
                                            tool_call.function.arguments.clone(),
                                        ),
                                    )
                                    .with_id(
                                        tool_call.id.clone().unwrap_or_else(|| "call_1".to_string()),
                                    )]));

                                messages.push(ChatMessage::tool(
                                    tool_call.id.clone().unwrap_or_else(|| "call_1".to_string()),
                                    search_response.to_context_string(),
                                ));

                                // Make a follow-up completion with the search results
                                let mut follow_up_request =
                                    CompletionRequest::new(model.clone(), messages);

                                if let Some(temp) = request.temperature {
                                    follow_up_request = follow_up_request.with_temperature(temp);
                                }
                                if let Some(max) = request.max_tokens {
                                    follow_up_request = follow_up_request.with_max_tokens(max);
                                }

                                match provider.complete(follow_up_request).await {
                                    Ok(final_completion) => {
                                        let content = final_completion
                                            .message
                                            .text()
                                            .unwrap_or_default()
                                            .to_string();
                                        let reasoning = final_completion.reasoning.map(|steps| {
                                            steps.into_iter().map(|step| step.content).collect()
                                        });
                                        let usage = final_completion.usage.map(TokenUsage::from);

                                        return Ok(Json(ChatCompletionResponse {
                                            model,
                                            content,
                                            usage,
                                            reasoning,
                                            web_search_sources,
                                        }));
                                    }
                                    Err(error) => {
                                        warn!(?error, "follow-up completion failed after web search");
                                        return Ok(Json(fallback_response(
                                            &model,
                                            prompt_trimmed,
                                            Some(error.to_string()),
                                        )));
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(?e, "web search failed, proceeding without search results");
                            }
                        }
                    }
                }

                // If we reach here, the model didn't use web search or search failed
                let content = completion.message.text().unwrap_or_default().to_string();
                let reasoning = completion
                    .reasoning
                    .map(|steps| steps.into_iter().map(|step| step.content).collect());
                let usage = completion.usage.map(TokenUsage::from);

                return Ok(Json(ChatCompletionResponse {
                    model,
                    content,
                    usage,
                    reasoning,
                    web_search_sources,
                }));
            } else {
                // Content injection mode: perform search first and inject results
                debug!(model = %model, "using content injection mode for web search");

                info!(query = %prompt_trimmed, "executing web search via content injection");

                match search_service.search(prompt_trimmed).await {
                    Ok(search_response) => {
                        // Store sources for the response
                        web_search_sources = Some(
                            search_response
                                .results
                                .iter()
                                .cloned()
                                .map(WebSearchSource::from)
                                .collect(),
                        );

                        // Add search results to system prompt
                        if !system_prompt_content.is_empty() {
                            system_prompt_content.push_str("\n\n");
                        }
                        system_prompt_content.push_str(WEB_SEARCH_CONTENT_INJECTION_PROMPT);
                        system_prompt_content.push_str(&search_response.to_context_string());
                    }
                    Err(e) => {
                        warn!(?e, "web search failed, proceeding without search results");
                    }
                }
            }
        }
    }

    // Add system prompt if we have any content
    if !system_prompt_content.is_empty() {
        messages.push(ChatMessage::system(&system_prompt_content));
    }

    // Add user message
    let user_message = build_user_message(prompt_trimmed, &request.images);
    messages.push(user_message);

    // Build the completion request
    let mut completion_request = CompletionRequest::new(model.clone(), messages);

    if let Some(temp) = request.temperature {
        completion_request = completion_request.with_temperature(temp);
    }
    if let Some(max) = request.max_tokens {
        completion_request = completion_request.with_max_tokens(max);
    }

    let completion = match provider.complete(completion_request).await {
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
        web_search_sources,
    }))
}

fn build_user_message(prompt: &str, images: &[Bytes]) -> ChatMessage {
    if images.is_empty() {
        ChatMessage::user(prompt)
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
        let full_content = format!("{} {}", prompt, image_parts.join(" "));
        ChatMessage::user(full_content)
    }
}

/// Check if a model likely supports function calling based on its ID.
/// This is a heuristic - the actual capability should come from the model metadata.
fn model_supports_function_calling(model: &str) -> bool {
    let model_lower = model.to_lowercase();

    // Models known to support function calling
    let tool_capable_patterns = [
        "gpt-4",
        "gpt-3.5-turbo",
        "claude-3",
        "claude-3.5",
        "gemini",
        "mistral",
        "mixtral",
        "llama-3.1",
        "llama-3.2",
        "qwen",
        "deepseek",
        "command-r",
    ];

    // Check if the model matches any known pattern
    tool_capable_patterns
        .iter()
        .any(|pattern| model_lower.contains(pattern))
}

fn fallback_response(model: &str, prompt: &str, error: Option<String>) -> ChatCompletionResponse {
    let trimmed_prompt = prompt.trim();
    let preview = if trimmed_prompt.is_empty() {
        "".to_string()
    } else {
        format!(
            " Prompt: \"{}\"",
            trimmed_prompt.chars().take(200).collect::<String>()
        )
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
        web_search_sources: None,
    }
}
