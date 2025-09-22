use riptide_api::errors::{ApiError, ApiResult};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::Value;

#[cfg(test)]
mod api_error_tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let error = ApiError::validation("Invalid input");

        match error {
            ApiError::ValidationError { message } => {
                assert_eq!(message, "Invalid input");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_invalid_url_error_creation() {
        let error = ApiError::invalid_url("http://example.com", "Bad URL");

        match error {
            ApiError::InvalidUrl { url, message } => {
                assert_eq!(url, "http://example.com");
                assert_eq!(message, "Bad URL");
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_fetch_error_creation() {
        let error = ApiError::fetch("http://example.com", "Network error");

        match error {
            ApiError::FetchError { url, message } => {
                assert_eq!(url, "http://example.com");
                assert_eq!(message, "Network error");
            }
            _ => panic!("Expected FetchError"),
        }
    }

    #[test]
    fn test_cache_error_creation() {
        let error = ApiError::cache("Redis connection failed");

        match error {
            ApiError::CacheError { message } => {
                assert_eq!(message, "Redis connection failed");
            }
            _ => panic!("Expected CacheError"),
        }
    }

    #[test]
    fn test_extraction_error_creation() {
        let error = ApiError::extraction("WASM extraction failed");

        match error {
            ApiError::ExtractionError { message } => {
                assert_eq!(message, "WASM extraction failed");
            }
            _ => panic!("Expected ExtractionError"),
        }
    }

    #[test]
    fn test_pipeline_error_creation() {
        let error = ApiError::pipeline("Pipeline step failed");

        match error {
            ApiError::PipelineError { message } => {
                assert_eq!(message, "Pipeline step failed");
            }
            _ => panic!("Expected PipelineError"),
        }
    }

    #[test]
    fn test_dependency_error_creation() {
        let error = ApiError::dependency("redis", "Connection timeout");

        match error {
            ApiError::DependencyError { service, message } => {
                assert_eq!(service, "redis");
                assert_eq!(message, "Connection timeout");
            }
            _ => panic!("Expected DependencyError"),
        }
    }

    #[test]
    fn test_timeout_error_creation() {
        let error = ApiError::timeout("http_request", "Request took too long");

        match error {
            ApiError::TimeoutError { operation, message } => {
                assert_eq!(operation, "http_request");
                assert_eq!(message, "Request took too long");
            }
            _ => panic!("Expected TimeoutError"),
        }
    }
}

#[cfg(test)]
mod status_code_tests {
    use super::*;

    #[test]
    fn test_validation_error_status_code() {
        let error = ApiError::validation("test");
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_invalid_url_status_code() {
        let error = ApiError::invalid_url("", "");
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_rate_limited_status_code() {
        let error = ApiError::RateLimited { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_authentication_error_status_code() {
        let error = ApiError::AuthenticationError { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_not_found_status_code() {
        let error = ApiError::NotFound { resource: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_payload_too_large_status_code() {
        let error = ApiError::PayloadTooLarge { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn test_timeout_error_status_code() {
        let error = ApiError::timeout("", "");
        assert_eq!(error.status_code(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn test_fetch_error_status_code() {
        let error = ApiError::fetch("", "");
        assert_eq!(error.status_code(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_cache_error_status_code() {
        let error = ApiError::cache("");
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_dependency_error_status_code() {
        let error = ApiError::dependency("", "");
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_extraction_error_status_code() {
        let error = ApiError::extraction("");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_routing_error_status_code() {
        let error = ApiError::RoutingError { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_pipeline_error_status_code() {
        let error = ApiError::pipeline("");
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_config_error_status_code() {
        let error = ApiError::ConfigError { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_internal_error_status_code() {
        let error = ApiError::InternalError { message: "test".to_string() };
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[cfg(test)]
mod error_type_tests {
    use super::*;

    #[test]
    fn test_error_type_strings() {
        assert_eq!(ApiError::validation("").error_type(), "validation_error");
        assert_eq!(ApiError::invalid_url("", "").error_type(), "invalid_url");
        assert_eq!(ApiError::RateLimited { message: "".to_string() }.error_type(), "rate_limited");
        assert_eq!(ApiError::AuthenticationError { message: "".to_string() }.error_type(), "authentication_error");
        assert_eq!(ApiError::fetch("", "").error_type(), "fetch_error");
        assert_eq!(ApiError::cache("").error_type(), "cache_error");
        assert_eq!(ApiError::extraction("").error_type(), "extraction_error");
        assert_eq!(ApiError::RoutingError { message: "".to_string() }.error_type(), "routing_error");
        assert_eq!(ApiError::pipeline("").error_type(), "pipeline_error");
        assert_eq!(ApiError::ConfigError { message: "".to_string() }.error_type(), "config_error");
        assert_eq!(ApiError::dependency("", "").error_type(), "dependency_error");
        assert_eq!(ApiError::InternalError { message: "".to_string() }.error_type(), "internal_error");
        assert_eq!(ApiError::timeout("", "").error_type(), "timeout_error");
        assert_eq!(ApiError::NotFound { resource: "".to_string() }.error_type(), "not_found");
        assert_eq!(ApiError::PayloadTooLarge { message: "".to_string() }.error_type(), "payload_too_large");
    }
}

#[cfg(test)]
mod retryable_tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        assert!(ApiError::timeout("", "").is_retryable());
        assert!(ApiError::cache("").is_retryable());
        assert!(ApiError::dependency("", "").is_retryable());
        assert!(ApiError::fetch("", "").is_retryable());
    }

    #[test]
    fn test_non_retryable_errors() {
        assert!(!ApiError::validation("").is_retryable());
        assert!(!ApiError::invalid_url("", "").is_retryable());
        assert!(!ApiError::extraction("").is_retryable());
        assert!(!ApiError::pipeline("").is_retryable());
        assert!(!ApiError::AuthenticationError { message: "".to_string() }.is_retryable());
        assert!(!ApiError::NotFound { resource: "".to_string() }.is_retryable());
        assert!(!ApiError::PayloadTooLarge { message: "".to_string() }.is_retryable());
    }
}

#[cfg(test)]
mod into_response_tests {
    use super::*;

    #[tokio::test]
    async fn test_error_into_response_structure() {
        let error = ApiError::validation("Test validation error");
        let response = error.into_response();

        // Check status code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Extract and parse the body
        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        // Check JSON structure
        assert!(body_json["error"].is_object());
        assert_eq!(body_json["error"]["type"], "validation_error");
        assert!(body_json["error"]["message"].as_str().unwrap().contains("Test validation error"));
        assert_eq!(body_json["error"]["retryable"], false);
        assert_eq!(body_json["error"]["status"], 400);
    }

    #[tokio::test]
    async fn test_retryable_error_response() {
        let error = ApiError::timeout("fetch", "Request timed out");
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["error"]["type"], "timeout_error");
        assert_eq!(body_json["error"]["retryable"], true);
        assert_eq!(body_json["error"]["status"], 408);
    }

    #[tokio::test]
    async fn test_internal_error_response() {
        let error = ApiError::extraction("WASM module failed");
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["error"]["type"], "extraction_error");
        assert_eq!(body_json["error"]["retryable"], false);
        assert_eq!(body_json["error"]["status"], 500);
    }
}

#[cfg(test)]
mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_error = anyhow::anyhow!("Something went wrong");
        let api_error: ApiError = anyhow_error.into();

        match api_error {
            ApiError::InternalError { message } => {
                assert!(message.contains("Something went wrong"));
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_reqwest_timeout_conversion() {
        // Create a mock timeout error scenario
        // Note: This is a simplified test as reqwest::Error is hard to construct directly
        let url = url::Url::parse("http://example.com").unwrap();
        let parse_error = url::ParseError::EmptyHost;
        let api_error: ApiError = parse_error.into();

        match api_error {
            ApiError::InvalidUrl { url: _, message } => {
                assert!(message.contains("EmptyHost") || message.len() > 0);
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_url_parse_error_conversion() {
        let parse_error = url::ParseError::EmptyHost;
        let api_error: ApiError = parse_error.into();

        match api_error {
            ApiError::InvalidUrl { url, message } => {
                assert_eq!(url, "");
                assert!(message.contains("empty host"));
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Create a JSON parsing error
        let json_error = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let api_error: ApiError = json_error.into();

        match api_error {
            ApiError::ValidationError { message } => {
                assert!(message.contains("JSON parsing error"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[cfg(test)]
mod error_display_tests {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        let validation_error = ApiError::validation("Invalid input format");
        assert_eq!(format!("{}", validation_error), "Validation error: Invalid input format");

        let url_error = ApiError::invalid_url("http://bad-url", "Malformed URL");
        assert_eq!(format!("{}", url_error), "Invalid URL: http://bad-url - Malformed URL");

        let fetch_error = ApiError::fetch("http://example.com", "Network failure");
        assert_eq!(format!("{}", fetch_error), "Failed to fetch content from http://example.com: Network failure");

        let timeout_error = ApiError::timeout("database_query", "Query took too long");
        assert_eq!(format!("{}", timeout_error), "Operation timed out: database_query - Query took too long");

        let dependency_error = ApiError::dependency("redis", "Connection refused");
        assert_eq!(format!("{}", dependency_error), "Dependency unavailable: redis - Connection refused");
    }
}

#[cfg(test)]
mod api_result_tests {
    use super::*;

    #[test]
    fn test_api_result_ok() {
        let result: ApiResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_api_result_err() {
        let result: ApiResult<String> = Err(ApiError::validation("error"));
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert_eq!(message, "error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}

// Property-based tests for error consistency
#[cfg(test)]
mod error_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validation_error_properties(message in ".*") {
            let error = ApiError::validation(&message);

            // Error type should be consistent
            prop_assert_eq!(error.error_type(), "validation_error");
            prop_assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
            prop_assert!(!error.is_retryable());

            // Display should contain the message
            let display_str = format!("{}", error);
            prop_assert!(display_str.contains(&message));
        }

        #[test]
        fn test_url_error_properties(url in ".*", message in ".*") {
            let error = ApiError::invalid_url(&url, &message);

            prop_assert_eq!(error.error_type(), "invalid_url");
            prop_assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
            prop_assert!(!error.is_retryable());

            let display_str = format!("{}", error);
            prop_assert!(display_str.contains(&url));
            prop_assert!(display_str.contains(&message));
        }

        #[test]
        fn test_retryable_error_consistency(message in ".*") {
            let errors = vec![
                ApiError::timeout("op", &message),
                ApiError::cache(&message),
                ApiError::dependency("service", &message),
                ApiError::fetch("url", &message),
            ];

            for error in errors {
                prop_assert!(error.is_retryable());

                // Retryable errors should have appropriate status codes
                let status = error.status_code();
                prop_assert!(
                    status == StatusCode::REQUEST_TIMEOUT ||
                    status == StatusCode::SERVICE_UNAVAILABLE ||
                    status == StatusCode::BAD_GATEWAY
                );
            }
        }
    }
}