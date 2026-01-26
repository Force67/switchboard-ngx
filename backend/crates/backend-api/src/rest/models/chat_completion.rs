//! Chat completion request and response types

use crate::services::SearchResult;
use denkwerk::TokenUsage as ProviderTokenUsage;
use serde::Serialize;
use utoipa::ToSchema;

/// Token usage statistics
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

/// Web search source information
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

/// Response from a chat completion request
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

/// Form fields for chat completion request
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
