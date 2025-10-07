use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Default maximum request body size (10 MB)
const DEFAULT_MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Layer for payload size limiting
#[derive(Clone)]
pub struct PayloadLimitLayer {
    max_size: usize,
}

impl PayloadLimitLayer {
    /// Create a new payload limit layer with default 10MB limit
    pub fn new() -> Self {
        Self {
            max_size: DEFAULT_MAX_PAYLOAD_SIZE,
        }
    }

    /// Create a new payload limit layer with custom limit in bytes
    pub fn with_limit(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for PayloadLimitLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for PayloadLimitLayer {
    type Service = PayloadLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PayloadLimitService {
            inner,
            max_size: self.max_size,
        }
    }
}

/// Service for payload size limiting
#[derive(Clone)]
pub struct PayloadLimitService<S> {
    inner: S,
    max_size: usize,
}

impl<S> Service<Request> for PayloadLimitService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let max_size = self.max_size;
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Check content-length header if present
            if let Some(content_length) = request.headers().get(http::header::CONTENT_LENGTH) {
                if let Ok(length_str) = content_length.to_str() {
                    if let Ok(length) = length_str.parse::<usize>() {
                        if length > max_size {
                            let error_msg = format!(
                                "Request payload too large: {} bytes (max {} bytes)",
                                length, max_size
                            );
                            tracing::warn!(
                                size = length,
                                max_size = max_size,
                                "Payload size limit exceeded"
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::PAYLOAD_TOO_LARGE)
                                .header(http::header::CONTENT_TYPE, "application/json")
                                .body(Body::from(
                                    serde_json::json!({
                                        "error": "PayloadTooLarge",
                                        "message": error_msg,
                                        "max_size_bytes": max_size,
                                        "received_bytes": length,
                                    })
                                    .to_string(),
                                ))
                                .unwrap()
                                .into_response());
                        }
                    }
                }
            }

            inner.call(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Router};
    use http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_payload_within_limit() {
        let app = Router::new()
            .route("/test", post(|| async { "OK" }))
            .layer(PayloadLimitLayer::with_limit(100));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(http::header::CONTENT_LENGTH, "50")
            .body(Body::from("small payload"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_payload_exceeds_limit() {
        let app = Router::new()
            .route("/test", post(|| async { "OK" }))
            .layer(PayloadLimitLayer::with_limit(100));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(http::header::CONTENT_LENGTH, "200")
            .body(Body::from("large payload"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_no_content_length_header() {
        let app = Router::new()
            .route("/test", post(|| async { "OK" }))
            .layer(PayloadLimitLayer::with_limit(100));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .body(Body::from("payload without content-length"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should allow through if no content-length header
        assert_eq!(response.status(), StatusCode::OK);
    }
}
