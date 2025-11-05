use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Comprehensive error types for the RipTide API with appropriate HTTP status codes.
///
/// This enum covers all error scenarios that can occur during crawling operations,
/// from validation errors to internal system failures. Each error type maps to
/// an appropriate HTTP status code and user-friendly error message.
#[derive(Error, Debug)]
pub enum ApiError {
    /// Input validation errors (400 Bad Request)
    #[error("Validation error: {message}")]
    ValidationError { message: String },

    /// URL parsing or format errors (400 Bad Request)
    #[error("Invalid URL: {url} - {message}")]
    InvalidUrl { url: String, message: String },

    /// Rate limiting errors (429 Too Many Requests)
    #[error("Rate limit exceeded: {message}")]
    RateLimited { message: String },

    /// Authentication/authorization errors (401/403)
    ///
    /// Authentication is implemented via API key validation in the auth middleware.
    /// Supported headers: X-API-Key or Authorization: Bearer <token>
    /// Configuration via environment variables:
    ///   - API_KEYS: Comma-separated list of valid API keys
    ///   - REQUIRE_AUTH: Enable/disable authentication (default: true)
    ///
    /// See: crates/riptide-api/src/middleware/auth.rs
    #[error("Authentication failed: {message}")]
    #[allow(dead_code)]
    AuthenticationError { message: String },

    /// Content fetch errors (502 Bad Gateway or 404 Not Found)
    #[error("Failed to fetch content from {url}: {message}")]
    FetchError { url: String, message: String },

    /// Cache operation errors (503 Service Unavailable)
    #[error("Cache operation failed: {message}")]
    CacheError { message: String },

    /// WASM extractor errors (500 Internal Server Error)
    #[error("Content extraction failed: {message}")]
    ExtractionError { message: String },

    /// Gate/routing errors (500 Internal Server Error)
    #[error("Content routing failed: {message}")]
    #[allow(dead_code)] // Used by gate module for routing failures
    RoutingError { message: String },

    /// Pipeline orchestration errors (500 Internal Server Error)
    #[error("Pipeline execution failed: {message}")]
    PipelineError { message: String },

    /// Configuration errors (500 Internal Server Error)
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Dependency health errors (503 Service Unavailable)
    #[error("Dependency unavailable: {service} - {message}")]
    DependencyError { service: String, message: String },

    /// Generic internal errors (500 Internal Server Error)
    #[error("Internal server error: {message}")]
    InternalError { message: String },

    /// Timeout errors (408 Request Timeout)
    #[error("Operation timed out: {operation} - {message}")]
    TimeoutError { operation: String, message: String },

    /// Resource not found errors (404 Not Found)
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    /// Request payload too large (413 Payload Too Large)
    /// Used by payload_limit middleware to enforce request size limits
    #[error("Request payload too large: {message}")]
    PayloadTooLarge { message: String },

    /// Invalid Content-Type header (415 Unsupported Media Type)
    #[error("Invalid Content-Type: {content_type}. {message}")]
    InvalidContentType {
        content_type: String,
        message: String,
    },

    /// Missing required header (400 Bad Request)
    #[error("Missing required header: {header}")]
    MissingRequiredHeader { header: String },

    /// Invalid header value (400 Bad Request)
    #[error("Invalid header value for {header}: {message}")]
    InvalidHeaderValue { header: String, message: String },

    /// Invalid request parameter (400 Bad Request)
    #[error("Invalid parameter {parameter}: {message}")]
    InvalidParameter { parameter: String, message: String },

    /// Feature not enabled in this build (501 Not Implemented)
    /// Used when optional features are disabled at compile time
    #[error("Feature '{feature}' is not enabled in this build")]
    FeatureNotEnabled { feature: String },
}

impl ApiError {
    /// Create a validation error with a custom message.
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create an invalid request error (alias for validation error).
    pub fn invalid_request<S: Into<String>>(message: S) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create an invalid URL error.
    pub fn invalid_url<S1: Into<String>, S2: Into<String>>(url: S1, message: S2) -> Self {
        Self::InvalidUrl {
            url: url.into(),
            message: message.into(),
        }
    }

    /// Create a fetch error for a specific URL.
    pub fn fetch<S1: Into<String>, S2: Into<String>>(url: S1, message: S2) -> Self {
        Self::FetchError {
            url: url.into(),
            message: message.into(),
        }
    }

    /// Create a cache operation error.
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::CacheError {
            message: message.into(),
        }
    }

    /// Create an extraction error.
    pub fn extraction<S: Into<String>>(message: S) -> Self {
        Self::ExtractionError {
            message: message.into(),
        }
    }

    /// Create a pipeline error.
    pub fn pipeline<S: Into<String>>(message: S) -> Self {
        Self::PipelineError {
            message: message.into(),
        }
    }

    /// Create a dependency error.
    pub fn dependency<S1: Into<String>, S2: Into<String>>(service: S1, message: S2) -> Self {
        Self::DependencyError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error.
    pub fn timeout<S1: Into<String>, S2: Into<String>>(operation: S1, message: S2) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a service unavailable error.
    pub fn service_unavailable<S: Into<String>>(message: S) -> Self {
        Self::DependencyError {
            service: "service".into(),
            message: message.into(),
        }
    }

    /// Create a rate limited error.
    pub fn rate_limited<S: Into<String>>(message: S) -> Self {
        Self::RateLimited {
            message: message.into(),
        }
    }

    /// Create an internal error.
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Create a not found error.
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a feature not enabled error.
    pub fn feature_not_enabled<S: Into<String>>(feature: S) -> Self {
        Self::FeatureNotEnabled {
            feature: feature.into(),
        }
    }

    /// Get the appropriate HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ApiError::InvalidUrl { .. } => StatusCode::BAD_REQUEST,
            ApiError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiError::AuthenticationError { .. } => StatusCode::UNAUTHORIZED,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::PayloadTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::TimeoutError { .. } => StatusCode::REQUEST_TIMEOUT,
            ApiError::FetchError { .. } => StatusCode::BAD_GATEWAY,
            ApiError::CacheError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::DependencyError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::ExtractionError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RoutingError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::PipelineError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidContentType { .. } => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::MissingRequiredHeader { .. } => StatusCode::BAD_REQUEST,
            ApiError::InvalidHeaderValue { .. } => StatusCode::BAD_REQUEST,
            ApiError::InvalidParameter { .. } => StatusCode::BAD_REQUEST,
            ApiError::FeatureNotEnabled { .. } => StatusCode::NOT_IMPLEMENTED,
        }
    }

    /// Get the error type as a string for logging and client identification.
    pub fn error_type(&self) -> &'static str {
        match self {
            ApiError::ValidationError { .. } => "validation_error",
            ApiError::InvalidUrl { .. } => "invalid_url",
            ApiError::RateLimited { .. } => "rate_limited",
            ApiError::AuthenticationError { .. } => "authentication_error",
            ApiError::FetchError { .. } => "fetch_error",
            ApiError::CacheError { .. } => "cache_error",
            ApiError::ExtractionError { .. } => "extraction_error",
            ApiError::RoutingError { .. } => "routing_error",
            ApiError::PipelineError { .. } => "pipeline_error",
            ApiError::ConfigError { .. } => "config_error",
            ApiError::DependencyError { .. } => "dependency_error",
            ApiError::InternalError { .. } => "internal_error",
            ApiError::TimeoutError { .. } => "timeout_error",
            ApiError::NotFound { .. } => "not_found",
            ApiError::PayloadTooLarge { .. } => "payload_too_large",
            ApiError::InvalidContentType { .. } => "invalid_content_type",
            ApiError::MissingRequiredHeader { .. } => "missing_required_header",
            ApiError::InvalidHeaderValue { .. } => "invalid_header_value",
            ApiError::InvalidParameter { .. } => "invalid_parameter",
            ApiError::FeatureNotEnabled { .. } => "feature_not_enabled",
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ApiError::TimeoutError { .. }
                | ApiError::CacheError { .. }
                | ApiError::DependencyError { .. }
                | ApiError::FetchError { .. }
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_type = self.error_type();
        let message = self.to_string();

        // Log the error for internal monitoring
        match status {
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::BAD_GATEWAY => {
                tracing::error!(
                    error_type = error_type,
                    message = %message,
                    "API error occurred"
                );
            }
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                tracing::warn!(
                    error_type = error_type,
                    message = %message,
                    "Client error occurred"
                );
            }
            _ => {
                tracing::info!(
                    error_type = error_type,
                    message = %message,
                    "API error occurred"
                );
            }
        }

        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": message,
                "retryable": self.is_retryable(),
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

/// Convert common error types to ApiError
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::TimeoutError {
                operation: "http_request".to_string(),
                message: err.to_string(),
            }
        } else if err.is_connect() {
            ApiError::FetchError {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                message: format!("Connection failed: {}", err),
            }
        } else {
            ApiError::FetchError {
                url: err.url().map(|u| u.to_string()).unwrap_or_default(),
                message: err.to_string(),
            }
        }
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        ApiError::CacheError {
            message: err.to_string(),
        }
    }
}

impl From<url::ParseError> for ApiError {
    fn from(err: url::ParseError) -> Self {
        ApiError::InvalidUrl {
            url: "".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ValidationError {
            message: format!("JSON parsing error: {}", err),
        }
    }
}

/// Result type alias for API operations.
pub type ApiResult<T> = Result<T, ApiError>;
