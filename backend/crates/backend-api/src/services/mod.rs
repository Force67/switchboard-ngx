pub mod web_search;

pub use web_search::{
    create_web_search_tool, SearchResponse, SearchResult, WebSearchError, WebSearchService,
    WEB_SEARCH_CONTENT_INJECTION_PROMPT, WEB_SEARCH_TOOL_SYSTEM_PROMPT,
};
