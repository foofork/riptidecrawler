use crate::errors::{ApiError, ApiResult};
use crate::models::{CrawlResult, DeepSearchBody, DeepSearchResponse, SearchResult};
use crate::pipeline::PipelineOrchestrator;
use crate::state::AppState;
use crate::validation::validate_deepsearch_request;
use axum::{extract::State, response::IntoResponse, Json};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Deep search endpoint using configurable search providers for web search and content extraction.
///
/// This endpoint:
/// 1. Validates the search query and parameters
/// 2. Performs a web search using the configured SearchProvider (Serper, None, etc.)
/// 3. Extracts URLs from search results
/// 4. Crawls the discovered URLs using the standard pipeline
/// 5. Returns combined search and content results
///
/// Supports multiple search backends configured via environment variables.
pub async fn deepsearch(
    State(state): State<AppState>,
    Json(body): Json<DeepSearchBody>,
) -> Result<impl IntoResponse, ApiError> {
    let start_time = Instant::now();

    info!(
        query = %body.query,
        limit = body.limit,
        "Received deep search request"
    );

    // Validate the request
    validate_deepsearch_request(&body)?;

    // Get search parameters
    let limit = body.limit.unwrap_or(10).min(50); // Cap at 50 results
    let include_content = body.include_content.unwrap_or(true);

    debug!(
        limit = limit,
        include_content = include_content,
        "Performing web search using SearchProvider"
    );

    // Perform web search using configured SearchProvider
    let search_results = perform_search_with_provider(
        &state,
        &body.query,
        limit,
        body.country.as_deref(),
        body.locale.as_deref(),
    ).await?;

    info!(
        query = %body.query,
        results_found = search_results.len(),
        "Web search completed"
    );

    // Extract URLs for crawling
    let urls: Vec<String> = search_results.iter().map(|r| r.url.clone()).collect();

    // If content extraction is requested, crawl the URLs
    let mut final_results = Vec::new();
    if include_content && !urls.is_empty() {
        let crawl_options = body.crawl_options.unwrap_or_default();
        let pipeline = PipelineOrchestrator::new(state.clone(), crawl_options);

        debug!(
            url_count = urls.len(),
            "Starting content extraction for search results"
        );

        let (pipeline_results, _) = pipeline.execute_batch(&urls).await;

        // Combine search results with crawled content
        for (index, search_result) in search_results.into_iter().enumerate() {
            let content = pipeline_results
                .get(index)
                .and_then(|r| r.as_ref().map(|pr| pr.document.clone()));

            let crawl_result = pipeline_results.get(index).and_then(|r| {
                r.as_ref().map(|pr| CrawlResult {
                    url: pr.document.url.clone(),
                    status: pr.http_status,
                    from_cache: pr.from_cache,
                    gate_decision: pr.gate_decision.clone(),
                    quality_score: pr.quality_score,
                    processing_time_ms: pr.processing_time_ms,
                    document: Some(pr.document.clone()),
                    error: None,
                    cache_key: pr.cache_key.clone(),
                })
            });

            final_results.push(SearchResult {
                url: search_result.url,
                rank: search_result.rank,
                search_title: search_result.search_title,
                search_snippet: search_result.search_snippet,
                content,
                crawl_result,
            });
        }
    } else {
        // Return search results without content
        final_results = search_results;
    }

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let query_clone = body.query.clone();
    let response = DeepSearchResponse {
        query: body.query,
        urls_found: urls.len(),
        urls_crawled: if include_content { urls.len() } else { 0 },
        results: final_results,
        status: "completed".to_string(),
        processing_time_ms,
    };

    info!(
        query = %query_clone,
        urls_found = urls.len(),
        processing_time_ms = processing_time_ms,
        "Deep search completed"
    );

    // Record metrics for deepsearch request
    state.metrics.record_http_request(
        "POST",
        "/deepsearch",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(Json(response))
}

/// Perform web search using configured SearchProvider with advanced factory.
pub(super) async fn perform_search_with_provider(
    _state: &AppState,
    query: &str,
    limit: u32,
    country: Option<&str>,
    locale: Option<&str>,
) -> ApiResult<Vec<SearchResult>> {
    use riptide_search::{SearchProviderFactory, SearchBackend};

    // Determine search backend from environment variable with validation
    let backend_str = std::env::var("SEARCH_BACKEND").unwrap_or_else(|_| "serper".to_string());
    let backend: SearchBackend = backend_str.parse().map_err(|e| ApiError::ConfigError {
        message: format!("Invalid search backend '{}': {}", backend_str, e),
    })?;

    debug!(
        backend = %backend,
        query = query,
        "Creating search provider using SearchProviderFactory"
    );

    // Create search provider using the advanced factory with environment configuration
    let provider = SearchProviderFactory::create_with_backend(backend).await.map_err(|e| {
        ApiError::dependency("search_provider", format!("SearchProviderFactory failed to create provider: {}", e))
    })?;

    // Perform health check to ensure provider is ready
    if let Err(health_error) = provider.health_check().await {
        warn!(
            backend = %provider.backend_type(),
            error = %health_error,
            "Search provider health check failed, but proceeding with request"
        );
        // Note: We continue with the request even if health check fails,
        // as the circuit breaker will handle provider failures gracefully
    }

    debug!(
        backend = %provider.backend_type(),
        query = query,
        "Using search provider with circuit breaker protection"
    );

    // Perform search using the provider with comprehensive error handling
    let search_hits = provider
        .search(
            query,
            limit,
            country.unwrap_or("us"),
            locale.unwrap_or("en"),
        )
        .await
        .map_err(|e| {
            let error_msg = e.to_string();

            // Provide specific error handling for circuit breaker states
            if error_msg.contains("circuit breaker is OPEN") {
                warn!(
                    backend = %provider.backend_type(),
                    error = %e,
                    "Search provider circuit breaker is open - provider is currently unavailable"
                );
                ApiError::service_unavailable(
                    "Search provider is temporarily unavailable due to repeated failures. Please try again later.".to_string()
                )
            } else if error_msg.contains("API key") {
                ApiError::ConfigError {
                    message: "Search provider API key is invalid or missing. Please check your configuration.".to_string()
                }
            } else if error_msg.contains("timeout") || error_msg.contains("Timeout") {
                ApiError::timeout("search_provider", "Search request timed out. Please try again or reduce the search limit.".to_string())
            } else {
                ApiError::dependency("search_provider", format!("Search operation failed: {}", e))
            }
        })?;

    // Convert SearchHit results to API SearchResult format
    let results: Vec<SearchResult> = search_hits
        .into_iter()
        .map(|hit| SearchResult {
            url: hit.url,
            rank: hit.rank,
            search_title: hit.title,
            search_snippet: hit.snippet,
            content: None,
            crawl_result: None,
        })
        .collect();

    info!(
        backend = %provider.backend_type(),
        results_count = results.len(),
        "Search completed successfully"
    );

    Ok(results)
}