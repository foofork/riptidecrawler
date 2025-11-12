//! Security headers middleware using riptide-security
//!
//! Applies production security headers (HSTS, CSP, X-Frame-Options, etc.)
//! to all HTTP responses.

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use riptide_security::SecurityMiddleware;
use std::sync::Arc;
use tracing::{debug, warn};

/// Middleware that applies security headers to responses
pub async fn security_headers_middleware(
    security: Arc<SecurityMiddleware>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Process the request
    let mut response = next.run(req).await;

    // Apply security headers to the response
    match security.apply_security_headers(response.headers_mut()) {
        Ok(_) => {
            debug!("Security headers applied successfully");
        }
        Err(e) => {
            warn!(error = ?e, "Failed to apply security headers");
        }
    }

    response
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
    use riptide_security::SecurityConfig;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "test response"
    }

    #[tokio::test]
    async fn test_security_headers_applied() {
        let config = SecurityConfig::default();
        let security = Arc::new(SecurityMiddleware::new(config).unwrap());

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
