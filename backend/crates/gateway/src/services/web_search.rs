use std::time::Duration;

use denkwerk::{
    functions::{FunctionDefinition, FunctionParameter},
    Tool,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use switchboard_config::WebSearchConfig;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum WebSearchError {
    #[error("web search is not configured")]
    NotConfigured,
    #[error("missing API key for provider {0}")]
    MissingApiKey(String),
    #[error("missing Google CX for google provider")]
    MissingGoogleCx,
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("failed to parse search response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("search provider returned error: {0}")]
    ProviderError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

impl SearchResponse {
    /// Format search results as a context string for LLM consumption.
    pub fn to_context_string(&self) -> String {
        if self.results.is_empty() {
            return format!("No search results found for: {}", self.query);
        }

        let mut context = format!(
            "Web search results for \"{}\":\n\n",
            self.query
        );

        for (i, result) in self.results.iter().enumerate() {
            context.push_str(&format!(
                "[{}] {}\nURL: {}\n{}\n\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            ));
        }

        context
    }
}

pub struct WebSearchService {
    config: WebSearchConfig,
    client: Client,
}

impl WebSearchService {
    pub fn new(config: WebSearchConfig) -> Result<Self, WebSearchError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { config, client })
    }

    pub fn is_configured(&self) -> bool {
        match self.config.provider.as_str() {
            "google" => {
                self.config.api_key.is_some() && self.config.google_cx.is_some()
            }
            "bing" => self.config.api_key.is_some(),
            "serper" => self.config.api_key.is_some(),
            "searxng" => self.config.base_url.is_some(),
            _ => false,
        }
    }

    pub async fn search(&self, query: &str) -> Result<SearchResponse, WebSearchError> {
        match self.config.provider.as_str() {
            "google" => self.search_google(query).await,
            "bing" => self.search_bing(query).await,
            "serper" => self.search_serper(query).await,
            "searxng" => self.search_searxng(query).await,
            provider => {
                warn!(provider, "unknown search provider, falling back to google");
                self.search_google(query).await
            }
        }
    }

    async fn search_google(&self, query: &str) -> Result<SearchResponse, WebSearchError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| WebSearchError::MissingApiKey("google".to_string()))?;
        let cx = self
            .config
            .google_cx
            .as_ref()
            .ok_or(WebSearchError::MissingGoogleCx)?;

        let url = format!(
            "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&num={}",
            api_key,
            cx,
            urlencoding::encode(query),
            self.config.max_results
        );

        debug!(query, "executing Google Custom Search");

        let response = self.client.get(&url).send().await?.error_for_status()?;
        let body: GoogleSearchResponse = response.json().await?;

        let results = body
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|item| SearchResult {
                title: item.title,
                url: item.link,
                snippet: item.snippet.unwrap_or_default(),
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
        })
    }

    async fn search_bing(&self, query: &str) -> Result<SearchResponse, WebSearchError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| WebSearchError::MissingApiKey("bing".to_string()))?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.bing.microsoft.com/v7.0");

        let url = format!(
            "{}/search?q={}&count={}",
            base_url.trim_end_matches('/'),
            urlencoding::encode(query),
            self.config.max_results
        );

        debug!(query, "executing Bing Web Search");

        let response = self
            .client
            .get(&url)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .send()
            .await?
            .error_for_status()?;

        let body: BingSearchResponse = response.json().await?;

        let results = body
            .web_pages
            .map(|pages| pages.value)
            .unwrap_or_default()
            .into_iter()
            .map(|item| SearchResult {
                title: item.name,
                url: item.url,
                snippet: item.snippet.unwrap_or_default(),
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
        })
    }

    async fn search_serper(&self, query: &str) -> Result<SearchResponse, WebSearchError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| WebSearchError::MissingApiKey("serper".to_string()))?;

        let base_url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://google.serper.dev");

        let url = format!("{}/search", base_url);

        debug!(query, "executing Serper search");

        let response = self
            .client
            .post(&url)
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "q": query,
                "num": self.config.max_results
            }))
            .send()
            .await?
            .error_for_status()?;

        let body: SerperSearchResponse = response.json().await?;

        let results = body
            .organic
            .unwrap_or_default()
            .into_iter()
            .map(|item| SearchResult {
                title: item.title,
                url: item.link,
                snippet: item.snippet.unwrap_or_default(),
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
        })
    }

    async fn search_searxng(&self, query: &str) -> Result<SearchResponse, WebSearchError> {
        let base_url = self
            .config
            .base_url
            .as_ref()
            .ok_or(WebSearchError::NotConfigured)?;

        let url = format!(
            "{}/search?q={}&format=json&categories=general",
            base_url.trim_end_matches('/'),
            urlencoding::encode(query)
        );

        debug!(query, base_url, "executing SearXNG search");

        let response = self.client.get(&url).send().await?.error_for_status()?;
        let body: SearxngSearchResponse = response.json().await?;

        let results = body
            .results
            .into_iter()
            .take(self.config.max_results as usize)
            .map(|item| SearchResult {
                title: item.title,
                url: item.url,
                snippet: item.content.unwrap_or_default(),
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
        })
    }
}

// Google Custom Search API response types
#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
}

// Bing Web Search API response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BingSearchResponse {
    web_pages: Option<BingWebPages>,
}

#[derive(Debug, Deserialize)]
struct BingWebPages {
    value: Vec<BingSearchItem>,
}

#[derive(Debug, Deserialize)]
struct BingSearchItem {
    name: String,
    url: String,
    snippet: Option<String>,
}

// Serper API response types
#[derive(Debug, Deserialize)]
struct SerperSearchResponse {
    organic: Option<Vec<SerperSearchItem>>,
}

#[derive(Debug, Deserialize)]
struct SerperSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
}

// SearXNG response types
#[derive(Debug, Deserialize)]
struct SearxngSearchResponse {
    results: Vec<SearxngSearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearxngSearchItem {
    title: String,
    url: String,
    content: Option<String>,
}

/// Create the web_search tool definition for function calling.
pub fn create_web_search_tool() -> Tool {
    let mut definition = FunctionDefinition::new("web_search")
        .with_description("Search the web for current information. Use this when the user asks about recent events, needs up-to-date information, or when you need to verify facts.");

    definition.add_parameter(
        FunctionParameter::new(
            "query",
            json!({
                "type": "string"
            }),
        )
        .with_description("The search query to execute"),
    );

    definition.to_tool()
}

/// System prompt addition for models that support function calling.
pub const WEB_SEARCH_TOOL_SYSTEM_PROMPT: &str = r#"You have access to a web_search tool that allows you to search the internet for current information.

When to use web_search:
- When the user asks about recent events, news, or current information
- When you need to verify facts or get up-to-date data
- When the user explicitly asks you to search for something

When using web_search:
1. Formulate a clear, specific search query
2. Call the web_search function with the query
3. Analyze the search results to answer the user's question
4. Cite the sources in your response when relevant

Always prefer using the search tool over making up information when dealing with current events or factual queries."#;

/// System prompt addition for models that don't support function calling (content injection mode).
pub const WEB_SEARCH_CONTENT_INJECTION_PROMPT: &str = r#"The following web search results have been retrieved to help answer the user's question. Use this information to provide an accurate and up-to-date response. Cite sources when relevant.

"#;
