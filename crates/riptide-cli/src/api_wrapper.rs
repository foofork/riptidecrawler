/// API wrapper module for RipTide CLI
/// Provides high-level API operations with automatic fallback to direct execution
use crate::client::ApiClient;
use crate::execution_mode::ExecutionMode;
use anyhow::{Context, Result};

/// Wrapper for API operations with fallback support
pub struct ApiWrapper {
    client: ApiClient,
    execution_mode: ExecutionMode,
}

impl ApiWrapper {
    /// Create a new API wrapper
    pub fn new(
        client: ApiClient,
        execution_mode: ExecutionMode,
    ) -> Result<Self> {
        Ok(Self {
            client,
            execution_mode,
        })
    }

    /// Check if API is available
    pub async fn is_api_available(&self) -> bool {
        // For now, assume available if client was created successfully
        // In the future, this could perform an actual health check
        self.execution_mode.allows_api()
    }

    /// Get the underlying HTTP client
    pub fn http_client(&self) -> &ApiClient {
        &self.client
    }

    /// Get execution mode
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }

    /// Attempt API call with automatic fallback
    pub async fn try_api_with_fallback<F, T>(
        &self,
        api_operation: F,
        fallback_operation: impl FnOnce() -> Result<T>,
        _operation_name: &str,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // If in direct-only mode, skip API entirely
        if !self.execution_mode.allows_api() {
            return fallback_operation();
        }

        // Try API first
        match api_operation.await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Check if fallback is allowed
                if self.execution_mode.allows_fallback() {
                    fallback_operation()
                } else {
                    // API-only mode - propagate error
                    Err(e).context("API operation failed and fallback is disabled")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_wrapper_creation() {
        let client = ApiClient::new(
            "http://localhost:8080".to_string(),
            Some("test-key".to_string()),
        )
        .unwrap();

        let wrapper = ApiWrapper::new(client, ExecutionMode::ApiFirst);

        assert!(wrapper.is_ok());
    }

    #[test]
    fn test_direct_mode_skips_api() {
        let client = ApiClient::new("http://localhost:8080".to_string(), None).unwrap();

        let wrapper = ApiWrapper::new(client, ExecutionMode::DirectOnly).unwrap();

        assert_eq!(wrapper.execution_mode(), ExecutionMode::DirectOnly);
    }
}
