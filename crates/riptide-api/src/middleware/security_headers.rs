//! Security headers middleware using riptide-security
//!
//! Applies production security headers (HSTS, CSP, X-Frame-Options, etc.)
//! to all HTTP responses.

use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderName, HeaderValue, Request, Response},
    middleware::Next,
};
use riptide_security::SecurityMiddleware;
use std::sync::Arc;
use tracing::{debug, warn};

/// Middleware that applies security headers to responses
pub async fn security_headers_middleware(
    State(_security): State<Arc<SecurityMiddleware>>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Process the request
    let mut response = next.run(req).await;

    // Apply security headers directly to the http::HeaderMap
    // Note: SecurityMiddleware::apply_security_headers expects reqwest::HeaderMap,
    // so we apply headers directly here for axum::http::HeaderMap
    let headers = response.headers_mut();

    // Apply security headers directly
    if let Err(e) = apply_axum_security_headers(headers) {
        warn!(error = ?e, "Failed to apply security headers");
    } else {
        debug!("Security headers applied successfully");
    }

    response
}

/// Apply security headers to axum's http::HeaderMap
fn apply_axum_security_headers(headers: &mut axum::http::HeaderMap) -> Result<(), String> {
    // XSS Protection
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Content Type Protection
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Frame Protection
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Content Security Policy
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("frame-ancestors 'none'"),
    );

    // Strict Transport Security (HSTS)
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Referrer Policy
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    debug!("Applied security headers to response");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "test response"
    }

    #[tokio::test]
    async fn test_security_headers_applied() {
        let security = Arc::new(SecurityMiddleware::with_defaults().unwrap());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn_with_state(
                security,
                security_headers_middleware,
            ));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify security headers are present
        let headers = response.headers();
        assert!(headers.contains_key("x-xss-protection"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("referrer-policy"));
    }
}
