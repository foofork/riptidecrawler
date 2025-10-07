use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use riptide_performance::limits::RequestPermit;
// use std::sync::Arc; // Unused
use tracing::{debug, warn};

use crate::state::AppState;

/// Rate limiting middleware that enforces request limits
///
/// This middleware uses the PerformanceManager's ResourceLimiter to:
/// - Enforce maximum concurrent requests
/// - Apply per-client rate limiting
/// - Track circuit breaker state
/// - Monitor resource usage
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract client ID from headers (X-Client-ID, X-API-Key, or X-Forwarded-For)
    let client_id = extract_client_id(&request);

    debug!(
        path = %request.uri().path(),
        client_id = ?client_id,
        "Rate limit check"
    );

    // Check rate limits for this client
    if let Err(e) = state
        .performance_manager
        .check_rate_limits(client_id.as_deref())
        .await
    {
        warn!(
            client_id = ?client_id,
            error = %e,
            "Rate limit exceeded"
        );

        return Err(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .header("Retry-After", "60") // Suggest retry after 60 seconds
            .body(Body::from(
                serde_json::json!({
                    "error": "RateLimitExceeded",
                    "message": format!("Rate limit exceeded: {}", e),
                    "retry_after_seconds": 60,
                })
                .to_string(),
            ))
            .unwrap()
            .into_response());
    }

    // Acquire request permit (enforces max concurrent requests)
    let _permit: RequestPermit = match state.performance_manager.acquire_request_permit().await {
        Ok(permit) => permit,
        Err(e) => {
            warn!(
                client_id = ?client_id,
                error = %e,
                "Failed to acquire request permit"
            );

            return Err(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("Content-Type", "application/json")
                .header("Retry-After", "30")
                .body(Body::from(
                    serde_json::json!({
                        "error": "ServiceUnavailable",
                        "message": "System at capacity, please retry later",
                        "retry_after_seconds": 30,
                    })
                    .to_string(),
                ))
                .unwrap()
                .into_response());
        }
    };

    // Process the request
    let response = next.run(request).await;

    // Record success/failure for circuit breaker
    let service = "api";
    if response.status().is_success() {
        let _ = state.performance_manager.record_success(service).await;
    } else if response.status().is_server_error() {
        let _ = state.performance_manager.record_failure(service).await;
    }

    // Permit is automatically released when dropped
    Ok(response)
}

/// Extract client ID from request headers
fn extract_client_id(request: &Request) -> Option<String> {
    // Try X-Client-ID header first
    if let Some(client_id) = request
        .headers()
        .get("X-Client-ID")
        .and_then(|h| h.to_str().ok())
    {
        return Some(client_id.to_string());
    }

    // Try X-API-Key header
    if let Some(api_key) = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
    {
        return Some(api_key.to_string());
    }

    // Fall back to X-Forwarded-For or X-Real-IP
    if let Some(forwarded_for) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // Take the first IP in the list
        return Some(
            forwarded_for
                .split(',')
                .next()
                .unwrap_or(forwarded_for)
                .trim()
                .to_string(),
        );
    }

    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        return Some(real_ip.to_string());
    }

    // No client identification available
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    #[test]
    fn test_extract_client_id_from_headers() {
        // Test X-Client-ID
        let request = Request::builder()
            .header("X-Client-ID", "client-123")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("client-123".to_string()));

        // Test X-API-Key
        let request = Request::builder()
            .header("X-API-Key", "api-key-456")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("api-key-456".to_string()));

        // Test X-Forwarded-For
        let request = Request::builder()
            .header("X-Forwarded-For", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("192.168.1.1".to_string()));

        // Test no headers
        let request = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_client_id(&request), None);
    }
}
