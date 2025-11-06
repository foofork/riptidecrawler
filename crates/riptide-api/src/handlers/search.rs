//! Search integration handler using SearchFacade
//!
//! Provides search functionality using riptide-facade SearchFacade with
//! support for multiple search providers (Serper, None, SearXNG).

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::state::AppState;

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Number of results
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Country code
    #[serde(default = "default_country")]
    pub country: String,
    /// Language code
    #[serde(default = "default_language")]
    pub language: String,
    /// Force specific provider
    pub provider: Option<String>,
}

fn default_limit() -> u32 {
    10
}

fn default_country() -> String {
    "us".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

/// Search result
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: u32,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: usize,
    pub provider_used: String,
    pub search_time_ms: u64,
}

/// Search using configured providers
///
/// This endpoint provides search functionality using the riptide-search
/// provider infrastructure. It supports multiple providers with automatic
/// fallback capabilities.
#[tracing::instrument(skip(state), fields(query = %params.q, limit = params.limit))]
pub async fn search(State(state): State<AppState>, Query(params): Query<SearchQuery>) -> Response {
    let start = Instant::now();

    // Validate query
    if params.q.trim().is_empty() {
        tracing::warn!("Empty search query provided");
        return crate::errors::ApiError::validation("Search query cannot be empty").into_response();
    }

    // Validate and cap limit
    let limit = params.limit.clamp(1, 50);

    tracing::info!(
        query = %params.q,
        limit = limit,
        country = %params.country,
        language = %params.language,
        provider = ?params.provider,
        "Processing search request"
    );

    // Facade temporarily unavailable during refactoring
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json("Facade temporarily unavailable during refactoring".to_string()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 10);
    }

    #[test]
    fn test_default_country() {
        assert_eq!(default_country(), "us");
    }

    #[test]
    fn test_default_language() {
        assert_eq!(default_language(), "en");
    }

    #[test]
    fn test_search_query_deserialization() {
        let query_str = "q=rust%20web%20scraping&limit=20&country=uk&language=en";
        let query: SearchQuery = serde_urlencoded::from_str(query_str).unwrap();
        assert_eq!(query.q, "rust web scraping");
        assert_eq!(query.limit, 20);
        assert_eq!(query.country, "uk");
        assert_eq!(query.language, "en");
    }

    #[test]
    fn test_search_query_defaults() {
        let query_str = "q=test";
        let query: SearchQuery = serde_urlencoded::from_str(query_str).unwrap();
        assert_eq!(query.q, "test");
        assert_eq!(query.limit, 10); // Default
        assert_eq!(query.country, "us"); // Default
        assert_eq!(query.language, "en"); // Default
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            position: 1,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("https://example.com"));
    }
}
