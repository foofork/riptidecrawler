use crate::api_client::RiptideApiClient;
use crate::client::RipTideClient;
use crate::execution_mode::ExecutionMode;
/// API wrapper module for RipTide CLI
/// Provides high-level API operations with automatic fallback to direct execution
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Wrapper for API operations with fallback support
pub struct ApiWrapper {
    client: RipTideClient,
    api_client: Option<RiptideApiClient>,
    execution_mode: ExecutionMode,
}

impl ApiWrapper {
    /// Create a new API wrapper
    pub fn new(
        client: RipTideClient,
        base_url: String,
        api_key: Option<String>,
        execution_mode: ExecutionMode,
    ) -> Result<Self> {
        let api_client = if execution_mode.allows_api() {
            match RiptideApiClient::new(base_url, api_key) {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("Failed to create API client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            client,
            api_client,
            execution_mode,
        })
    }

    /// Check if API is available
    pub async fn is_api_available(&self) -> bool {
        if let Some(api_client) = &self.api_client {
            api_client.is_available().await
        } else {
            false
        }
    }

    /// Get the underlying HTTP client
    pub fn http_client(&self) -> &RipTideClient {
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
        operation_name: &str,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // If in direct-only mode, skip API entirely
        if !self.execution_mode.allows_api() {
            debug!("Direct-only mode: skipping API for {}", operation_name);
            return fallback_operation();
        }

        // Try API first
        match api_operation.await {
            Ok(result) => {
                info!("Successfully completed {} via API", operation_name);
                Ok(result)
            }
            Err(e) => {
                // Check if fallback is allowed
                if self.execution_mode.allows_fallback() {
                    warn!(
                        "API {} failed: {}. Falling back to direct execution.",
                        operation_name, e
                    );
                    fallback_operation()
                } else {
                    // API-only mode - propagate error
                    Err(e).context(format!(
                        "API {} failed and fallback is disabled",
                        operation_name
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_wrapper_creation() {
        let client = RipTideClient::new(
            "http://localhost:8080".to_string(),
            Some("test-key".to_string()),
        )
        .unwrap();

        let wrapper = ApiWrapper::new(
            client,
            "http://localhost:8080".to_string(),
            Some("test-key".to_string()),
            ExecutionMode::ApiFirst,
        );

        assert!(wrapper.is_ok());
    }

    #[tokio::test]
    async fn test_direct_mode_skips_api() {
        let client = RipTideClient::new("http://localhost:8080".to_string(), None).unwrap();

        let wrapper = ApiWrapper::new(
            client,
            "http://localhost:8080".to_string(),
            None,
            ExecutionMode::DirectOnly,
        )
        .unwrap();

        assert_eq!(wrapper.execution_mode(), ExecutionMode::DirectOnly);
    }
}
