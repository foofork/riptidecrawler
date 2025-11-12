//! PII redaction middleware using riptide-security
//!
//! Redacts personally identifiable information (PII) from request/response
//! bodies for GDPR/CCPA compliance.

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use riptide_security::PiiRedactionMiddleware;
use std::sync::Arc;
use tracing::debug;

/// Middleware that redacts PII from responses
///
/// This should be applied selectively to endpoints that may contain PII:
/// - User profile endpoints
/// - Search results
/// - Cache responses
/// - Extraction results
pub async fn pii_redaction_middleware(
    pii_redactor: Arc<PiiRedactionMiddleware>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Process the request
    let response = next.run(req).await;

    // Note: PII redaction on response body requires streaming body handling
    // For now, we're setting up the infrastructure. Full implementation
    // would require buffering and redacting the response body.
    //
    // This is a placeholder that demonstrates the middleware pattern.
    // In production, you'd want to:
    // 1. Buffer the response body
    // 2. Apply PII redaction using pii_redactor
    // 3. Return the redacted body
    //
    // For now, we'll just log that PII redaction is enabled
    debug!(
        middleware = "pii_redaction",
        "PII redaction middleware active (body redaction requires streaming implementation)"
    );

    // Return response unchanged for now
    // TODO: Implement body redaction when streaming support is added
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
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "test response with email@example.com"
    }

    #[tokio::test]
    async fn test_pii_redaction_middleware_setup() {
        let pii_redactor = Arc::new(PiiRedactionMiddleware::with_defaults());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(axum::middleware::from_fn_with_state(
                pii_redactor,
                pii_redaction_middleware,
            ));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify middleware doesn't break request flow
        assert_eq!(response.status(), StatusCode::OK);
    }
}
