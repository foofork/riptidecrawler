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
use std::time::Instant;

use crate::state::AppState;

// Import HTTP DTOs from riptide-types (Phase 2C.1 - breaking circular dependency)
use riptide_types::{SearchQuery, SearchResponse, SearchResult};

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

    // Phase 2C.2: Call SearchFacade (restored after circular dependency fix)
    let search_facade = match state.search_facade.as_ref() {
        Some(facade) => facade,
        None => {
            tracing::warn!("SearchFacade not initialized. Set SERPER_API_KEY environment variable to enable search.");
            return crate::errors::ApiError::ConfigError {
                message: "Search functionality not available. SERPER_API_KEY not configured."
                    .to_string(),
            }
            .into_response();
        }
    };

    // Call search facade
    let hits = match search_facade
        .search_with_options(&params.q, limit as u32, &params.country, &params.language)
        .await
    {
        Ok(hits) => hits,
        Err(e) => {
            tracing::error!(error = %e, "Search failed");
            return crate::errors::ApiError::from(e).into_response();
        }
    };

    // Map SearchHit to SearchResult
    let results: Vec<SearchResult> = hits
        .into_iter()
        .enumerate()
        .map(|(idx, hit)| SearchResult {
            title: hit.title,
            url: hit.link,
            snippet: hit.snippet.unwrap_or_default(),
            position: (idx + 1) as u32,
        })
        .collect();

    let response = SearchResponse {
        query: params.q.clone(),
        results,
        total_results: results.len() as u32,
        search_time_ms: start.elapsed().as_millis() as u64,
    };

    tracing::info!(
        query = %params.q,
        results_count = response.total_results,
        search_time_ms = response.search_time_ms,
        "Search completed successfully"
    );

    (StatusCode::OK, Json(response)).into_response()
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
