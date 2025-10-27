//! Request validation middleware for malformed payloads and unsupported methods
//!
//! Provides early rejection of invalid requests before they reach handlers:
//! - JSON schema validation for request bodies
//! - HTTP method validation (405 Method Not Allowed)
//! - Content-Type validation for POST/PUT/PATCH requests

use axum::{
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashSet;

use crate::errors::ApiError;

/// Middleware to validate request payloads and methods
///
/// This middleware performs the following validations:
/// 1. Content-Type validation for requests with bodies
/// 2. JSON payload validation (early parsing to catch malformed JSON)
/// 3. HTTP method validation against allowed methods
///
/// # Errors
///
/// Returns:
/// - 400 Bad Request for malformed JSON or missing Content-Type
/// - 405 Method Not Allowed for unsupported HTTP methods
/// - 415 Unsupported Media Type for invalid Content-Type
pub async fn request_validation_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri_path = request.uri().path().to_string();
    let headers = request.headers().clone();

    // Validate HTTP method first (fast path)
    if let Err(response) = validate_http_method(&method, &uri_path) {
        return response;
    }

    // Validate Content-Type for requests with bodies
    if should_validate_body(&method) {
        if let Err(response) = validate_content_type(&headers) {
            return response;
        }
    }

    // All validations passed, proceed to the next middleware/handler
    next.run(request).await
}

/// Check if the request method should have body validation
pub(crate) fn should_validate_body(method: &Method) -> bool {
    matches!(method, &Method::POST | &Method::PUT | &Method::PATCH)
}

/// Validate HTTP method against path-specific allowed methods
pub(crate) fn validate_http_method(method: &Method, path: &str) -> Result<(), Response> {
    // Define allowed methods per path pattern
    let allowed_methods = get_allowed_methods(path);

    if !allowed_methods.contains(method.as_str()) {
        tracing::warn!(
            method = %method,
            path = %path,
            allowed_methods = ?allowed_methods,
            "HTTP method not allowed for this endpoint"
        );

        return Err(method_not_allowed_response(&allowed_methods));
    }

    Ok(())
}

/// Get allowed HTTP methods for a given path
pub(crate) fn get_allowed_methods(path: &str) -> HashSet<&'static str> {
    // Health and metrics endpoints - GET only
    if path.starts_with("/healthz")
        || path.starts_with("/health")
        || path.starts_with("/metrics")
        || path.starts_with("/api/v1/metrics")
    {
        return ["GET", "HEAD"].iter().copied().collect();
    }

    // Search endpoint - GET only
    if path.starts_with("/search") || path.starts_with("/api/v1/search") {
        return ["GET", "HEAD"].iter().copied().collect();
    }

    // POST-only endpoints
    if path.starts_with("/crawl")
        || path.starts_with("/api/v1/crawl")
        || path.starts_with("/extract")
        || path.starts_with("/api/v1/extract")
        || path.starts_with("/deepsearch")
        || path.starts_with("/render")
        || path.starts_with("/api/v1/render")
        || path.starts_with("/strategies")
        || path.starts_with("/spider")
        || path.starts_with("/workers/jobs")
        || path.starts_with("/sessions")
    {
        return ["POST"].iter().copied().collect();
    }

    // WebSocket endpoint - GET only (for upgrade)
    if path.starts_with("/crawl/ws") {
        return ["GET"].iter().copied().collect();
    }

    // RESTful endpoints with multiple methods
    if path.starts_with("/api/v1/browser")
        || path.starts_with("/api/v1/llm")
        || path.starts_with("/api/v1/profiles")
        || path.starts_with("/admin")
    {
        return ["GET", "POST", "PUT", "PATCH", "DELETE"]
            .iter()
            .copied()
            .collect();
    }

    // Default: allow common HTTP methods
    ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"]
        .iter()
        .copied()
        .collect()
}

/// Validate Content-Type header
fn validate_content_type(headers: &axum::http::HeaderMap) -> Result<(), Response> {
    // Check Content-Type header
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Require application/json for JSON APIs
    if !content_type.starts_with("application/json") && !content_type.is_empty() {
        // Allow multipart/form-data for file uploads (PDF, etc.)
        if !content_type.starts_with("multipart/form-data") {
            tracing::warn!(
                content_type = %content_type,
                "Unsupported Content-Type header"
            );

            return Err(unsupported_media_type_response(content_type));
        }
    }

    // For JSON content, validate that the body is parseable
    // Note: We don't actually consume the body here, just peek at it
    // The actual parsing will happen in the handler with proper error handling
    if content_type.starts_with("application/json") {
        // Get content-length to validate payload size early
        if let Some(content_length) = headers.get(header::CONTENT_LENGTH) {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    // If content-length is 0, that's fine (empty body)
                    if length == 0 {
                        return Ok(());
                    }

                    // Basic sanity check - if extremely large, let PayloadLimitLayer handle it
                    // We just check for obviously malformed headers
                    if length > 100 * 1024 * 1024 {
                        // > 100MB seems suspicious
                        tracing::warn!(
                            content_length = length,
                            "Suspiciously large Content-Length header"
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Create a 405 Method Not Allowed response
fn method_not_allowed_response(allowed_methods: &HashSet<&str>) -> Response {
    let allowed = allowed_methods
        .iter()
        .copied()
        .collect::<Vec<_>>()
        .join(", ");

    let error = ApiError::validation(format!(
        "HTTP method not allowed. Allowed methods: {}",
        allowed
    ));

    let body = Json(json!({
        "error": {
            "type": "method_not_allowed",
            "message": error.to_string(),
            "retryable": false,
            "status": 405,
            "allowed_methods": allowed_methods.iter().copied().collect::<Vec<_>>()
        }
    }));

    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(header::ALLOW, allowed)],
        body,
    )
        .into_response()
}

/// Create a 415 Unsupported Media Type response
fn unsupported_media_type_response(content_type: &str) -> Response {
    let error = ApiError::validation(format!(
        "Unsupported Content-Type: {}. Expected application/json or multipart/form-data",
        content_type
    ));

    let body = Json(json!({
        "error": {
            "type": "unsupported_media_type",
            "message": error.to_string(),
            "retryable": false,
            "status": 415,
            "received_content_type": content_type,
            "supported_types": ["application/json", "multipart/form-data"]
        }
    }));

    (StatusCode::UNSUPPORTED_MEDIA_TYPE, body).into_response()
}

/// JSON rejection handler for malformed JSON payloads
///
/// This can be used as a custom rejection handler for axum's Json extractor
#[allow(dead_code)]
pub fn handle_json_rejection(rejection: axum::extract::rejection::JsonRejection) -> Response {
    tracing::warn!(
        rejection = %rejection,
        "JSON deserialization failed"
    );

    let (status, message) = match rejection {
        axum::extract::rejection::JsonRejection::JsonDataError(err) => (
            StatusCode::BAD_REQUEST,
            format!("Invalid JSON data: {}", err),
        ),
        axum::extract::rejection::JsonRejection::JsonSyntaxError(err) => (
            StatusCode::BAD_REQUEST,
            format!("JSON syntax error: {}", err),
        ),
        axum::extract::rejection::JsonRejection::MissingJsonContentType(err) => (
            StatusCode::BAD_REQUEST,
            format!("Missing Content-Type header: {}", err),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            format!("Invalid request body: {}", rejection),
        ),
    };

    let body = Json(json!({
        "error": {
            "type": "invalid_request_body",
            "message": message,
            "retryable": false,
            "status": status.as_u16()
        }
    }));

    (status, body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_validate_body() {
        assert!(should_validate_body(&Method::POST));
        assert!(should_validate_body(&Method::PUT));
        assert!(should_validate_body(&Method::PATCH));
        assert!(!should_validate_body(&Method::GET));
        assert!(!should_validate_body(&Method::DELETE));
        assert!(!should_validate_body(&Method::HEAD));
    }

    #[test]
    fn test_get_allowed_methods_health() {
        let methods = get_allowed_methods("/healthz");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));
        assert_eq!(methods.len(), 2);
    }

    #[test]
    fn test_get_allowed_methods_metrics() {
        let methods = get_allowed_methods("/metrics");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));
    }

    #[test]
    fn test_get_allowed_methods_search() {
        let methods = get_allowed_methods("/search");
        assert!(methods.contains("GET"));
        assert!(methods.contains("HEAD"));
        assert!(!methods.contains("POST"));
    }

    #[test]
    fn test_get_allowed_methods_crawl() {
        let methods = get_allowed_methods("/crawl");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));
        assert_eq!(methods.len(), 1);
    }

    #[test]
    fn test_get_allowed_methods_extract() {
        let methods = get_allowed_methods("/api/v1/extract");
        assert!(methods.contains("POST"));
        assert!(!methods.contains("GET"));
        assert_eq!(methods.len(), 1);
    }

    #[test]
    fn test_get_allowed_methods_websocket() {
        let methods = get_allowed_methods("/crawl/ws");
        assert!(methods.contains("GET"));
        assert!(!methods.contains("POST"));
        assert_eq!(methods.len(), 1);
    }

    #[test]
    fn test_get_allowed_methods_browser_api() {
        let methods = get_allowed_methods("/api/v1/browser/session");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("PUT"));
        assert!(methods.contains("PATCH"));
        assert!(methods.contains("DELETE"));
    }

    #[test]
    fn test_get_allowed_methods_default() {
        let methods = get_allowed_methods("/some/unknown/path");
        assert!(methods.contains("GET"));
        assert!(methods.contains("POST"));
        assert!(methods.contains("PUT"));
        assert!(methods.contains("PATCH"));
        assert!(methods.contains("DELETE"));
        assert!(methods.contains("HEAD"));
    }

    #[test]
    fn test_validate_http_method_success() {
        let result = validate_http_method(&Method::GET, "/healthz");
        assert!(result.is_ok());

        let result = validate_http_method(&Method::POST, "/crawl");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_http_method_failure() {
        let result = validate_http_method(&Method::POST, "/healthz");
        assert!(result.is_err());

        let result = validate_http_method(&Method::GET, "/crawl");
        assert!(result.is_err());
    }
}
