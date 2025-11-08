//! Search integration handler using SearchFacade
//!
//! Thin handler that delegates to SearchFacade for business logic.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::time::Instant;

use crate::state::AppState;
use riptide_types::{SearchQuery, SearchResponse, SearchResult};

/// Search using configured providers (thin handler pattern)
#[tracing::instrument(skip(state))]
pub async fn search(State(state): State<AppState>, Query(params): Query<SearchQuery>) -> Response {
    let start = Instant::now();
    let facade = match state.search_facade.as_ref() {
        Some(f) => f,
        None => return config_error().into_response(),
    };

    match facade
        .search_validated(&params.q, params.limit, &params.country, &params.language)
        .await
    {
        Ok(hits) => build_response(params.q, hits, start).into_response(),
        Err(e) => crate::errors::ApiError::from(e).into_response(),
    }
}

fn config_error() -> crate::errors::ApiError {
    crate::errors::ApiError::ConfigError {
        message: "Search functionality not available. SERPER_API_KEY not configured.".to_string(),
    }
}

fn build_response(
    query: String,
    hits: Vec<riptide_search::SearchHit>,
    start: Instant,
) -> (StatusCode, Json<SearchResponse>) {
    let results: Vec<SearchResult> = hits
        .into_iter()
        .enumerate()
        .map(|(idx, hit)| SearchResult {
            title: hit.title.unwrap_or_default(),
            url: hit.url,
            snippet: hit.snippet.unwrap_or_default(),
            position: (idx + 1) as u32,
        })
        .collect();

    (
        StatusCode::OK,
        Json(SearchResponse {
            query,
            total_results: results.len(),
            results,
            provider_used: "riptide-search".to_string(),
            search_time_ms: start.elapsed().as_millis() as u64,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let error = config_error();
        match error {
            crate::errors::ApiError::ConfigError { message } => {
                assert!(message.contains("Search functionality not available"));
            }
            _ => panic!("Expected ConfigError"),
        }
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
