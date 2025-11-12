use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use riptide_performance::limits::RequestPermit;
// use std::sync::Arc; // Unused
use tracing::{debug, warn};

use crate::context::ApplicationContext;

/// Rate limiting middleware that enforces request limits
///
/// This middleware uses the PerformanceManager's ResourceLimiter to:
/// - Enforce maximum concurrent requests
/// - Apply per-client rate limiting
/// - Track circuit breaker state
/// - Monitor resource usage
pub async fn rate_limit_middleware(
    State(state): State<ApplicationContext>,
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

        // Use ApiError for consistent error response format
        let api_error =
            crate::errors::ApiError::rate_limited(format!("Resource limit exceeded: {}", e));
        let mut response = api_error.into_response();

        // Add Retry-After header
        if let Ok(retry_value) = "60".parse() {
            response.headers_mut().insert("Retry-After", retry_value);
        }

        return Err(response);
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

            // Use ApiError for consistent error response format
            let api_error =
                crate::errors::ApiError::service_unavailable(format!("System at capacity: {}", e));
            let mut response = api_error.into_response();

            // Add Retry-After header
            if let Ok(retry_value) = "30".parse() {
                response.headers_mut().insert("Retry-After", retry_value);
            }

            return Err(response.into_response());
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

/// Extract client ID from request headers for rate limiting.
///
/// **Rate Limiting Strategy: API Key First**
///
/// This function extracts the client identifier with the following priority:
/// 1. **X-API-Key header** - Primary identifier (per-customer rate limiting)
/// 2. **Authorization: Bearer token** - Alternative API key format
/// 3. **IP address (X-Forwarded-For or X-Real-IP)** - Fallback for public endpoints
///
/// ## Why API Key First?
///
/// - **Fairer**: Shared IPs (corporate proxies, NAT) won't be unfairly limited
/// - **Better abuse prevention**: Tracks by customer, not IP
/// - **Accurate attribution**: Each customer gets their own quota
///
/// ## Security Note
///
/// - **Removed X-Client-ID**: Clients should not be able to override their identifier
///   (this was a potential security issue allowing quota bypass)
fn extract_client_id(request: &Request) -> Option<String> {
    // PRIORITY 1: X-API-Key header (most common)
    if let Some(api_key) = request
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
    {
        return Some(api_key.to_string());
    }

    // PRIORITY 2: Authorization: Bearer token (alternative format)
    if let Some(auth_header) = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    // PRIORITY 3: IP address fallback (for public endpoints only)
    // Note: By the time rate limiting runs, most requests should have an API key
    if let Some(forwarded_for) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // Take the first IP in the list (client IP)
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

    // No client identification available - this should be rare
    // (only for misconfigured clients or public endpoints)
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    #[test]
    fn test_extract_client_id_from_headers() {
        // Test PRIORITY 1: X-API-Key (most common)
        let request = Request::builder()
            .header("X-API-Key", "api-key-456")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("api-key-456".to_string()));

        // Test PRIORITY 2: Authorization: Bearer token
        let request = Request::builder()
            .header("Authorization", "Bearer token-789")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("token-789".to_string()));

        // Test PRIORITY 3: X-Forwarded-For (IP fallback for public endpoints)
        let request = Request::builder()
            .header("X-Forwarded-For", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("192.168.1.1".to_string()));

        // Test X-Real-IP fallback
        let request = Request::builder()
            .header("X-Real-IP", "10.0.0.5")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_client_id(&request), Some("10.0.0.5".to_string()));

        // Test API key takes priority over IP (most important test!)
        let request = Request::builder()
            .header("X-API-Key", "api-key-123")
            .header("X-Forwarded-For", "192.168.1.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_id(&request),
            Some("api-key-123".to_string()),
            "API key should take priority over IP address"
        );

        // Test Bearer token takes priority over IP
        let request = Request::builder()
            .header("Authorization", "Bearer token-456")
            .header("X-Real-IP", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_id(&request),
            Some("token-456".to_string()),
            "Bearer token should take priority over IP address"
        );

        // Test no headers (should be rare)
        let request = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_client_id(&request), None);
    }
}
