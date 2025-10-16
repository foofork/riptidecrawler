//! Cache Key Generation Module
//!
//! Provides deterministic, collision-resistant cache key generation
//! for all extraction methods across the Riptide system.
//!
//! # Features
//! - SHA256-based hashing for collision resistance
//! - Deterministic key generation (same inputs = same key)
//! - Version-aware for cache invalidation
//! - Option order independence (using BTreeMap)
//! - Namespace support for different subsystems

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Cache key parameters structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKeyParams {
    pub url: String,
    pub method: String,
    pub version: String,
    pub options: BTreeMap<String, String>,
    pub namespace: Option<String>,
}

/// Builder for deterministic cache key generation
#[derive(Debug, Clone)]
pub struct CacheKeyBuilder {
    url: Option<String>,
    method: Option<String>,
    version: String,
    options: BTreeMap<String, String>,
    namespace: Option<String>,
}

impl Default for CacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheKeyBuilder {
    /// Create a new cache key builder with defaults
    pub fn new() -> Self {
        Self {
            url: None,
            method: None,
            version: "v1".to_string(),
            options: BTreeMap::new(),
            namespace: None,
        }
    }

    /// Create from params struct
    pub fn from_params(params: &CacheKeyParams) -> Self {
        Self {
            url: Some(params.url.clone()),
            method: Some(params.method.clone()),
            version: params.version.clone(),
            options: params.options.clone(),
            namespace: params.namespace.clone(),
        }
    }

    /// Set the URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the extraction method
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set the version (for cache invalidation)
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add a single option
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Set all options at once
    pub fn options(mut self, options: BTreeMap<String, String>) -> Self {
        self.options = options;
        self
    }

    /// Set namespace for isolation
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Build the final cache key
    ///
    /// Format: `riptide:{namespace}:{version}:{hash}`
    /// Where hash is SHA256 of: url|method|sorted_options
    pub fn build(self) -> Result<String, anyhow::Error> {
        let url = self
            .url
            .ok_or_else(|| anyhow::anyhow!("URL is required for cache key"))?;
        let method = self
            .method
            .ok_or_else(|| anyhow::anyhow!("Method is required for cache key"))?;

        // Create deterministic string from components
        let mut hasher = Sha256::new();

        // Hash URL
        hasher.update(url.as_bytes());
        hasher.update(b"|");

        // Hash method
        hasher.update(method.as_bytes());
        hasher.update(b"|");

        // Hash version
        hasher.update(self.version.as_bytes());
        hasher.update(b"|");

        // Hash options in sorted order (BTreeMap maintains sort)
        for (key, value) in &self.options {
            hasher.update(key.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
            hasher.update(b"&");
        }

        // Hash namespace if present
        if let Some(ns) = &self.namespace {
            hasher.update(b"|");
            hasher.update(ns.as_bytes());
        }

        let hash = format!("{:x}", hasher.finalize());

        // Build final key
        Ok(match self.namespace {
            Some(ns) => format!("riptide:{}:{}:{}", ns, self.version, hash),
            None => format!("riptide:{}:{}", self.version, hash),
        })
    }

    /// Convert to params struct
    pub fn to_params(&self) -> Option<CacheKeyParams> {
        if self.url.is_none() || self.method.is_none() {
            return None;
        }

        Some(CacheKeyParams {
            url: self.url.clone()?,
            method: self.method.clone()?,
            version: self.version.clone(),
            options: self.options.clone(),
            namespace: self.namespace.clone(),
        })
    }
}

/// Helper function to create cache key from strategy configuration
pub fn generate_strategies_cache_key(
    url: &str,
    extraction_method: &str,
    cache_mode: &str,
    version: &str,
) -> Result<String, anyhow::Error> {
    CacheKeyBuilder::new()
        .url(url)
        .method(extraction_method)
        .version(version)
        .option("cache_mode", cache_mode)
        .namespace("strategies")
        .build()
}

/// Helper function to create cache key for fetch operations
pub fn generate_fetch_cache_key(
    url: &str,
    version: &str,
    options: &BTreeMap<String, String>,
) -> Result<String, anyhow::Error> {
    CacheKeyBuilder::new()
        .url(url)
        .method("fetch")
        .version(version)
        .options(options.clone())
        .namespace("fetch")
        .build()
}

/// Helper function to create cache key for WASM extraction
pub fn generate_wasm_cache_key(
    url: &str,
    extraction_mode: &str,
    version: &str,
) -> Result<String, anyhow::Error> {
    CacheKeyBuilder::new()
        .url(url)
        .method("wasm")
        .version(version)
        .option("mode", extraction_mode)
        .namespace("wasm")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let key = CacheKeyBuilder::new()
            .url("https://example.com")
            .method("wasm")
            .build()
            .expect("Failed to build cache key");

        assert!(key.starts_with("riptide:v1:"));
        assert!(key.len() > 20); // Should have hash
    }

    #[test]
    fn test_builder_with_options() {
        let key = CacheKeyBuilder::new()
            .url("https://example.com")
            .method("wasm")
            .option("chunking", "sentence")
            .option("language", "en")
            .build()
            .expect("Failed to build cache key");

        assert!(key.starts_with("riptide:v1:"));
    }

    #[test]
    fn test_builder_with_namespace() {
        let key = CacheKeyBuilder::new()
            .url("https://example.com")
            .method("wasm")
            .namespace("strategies")
            .build()
            .expect("Failed to build cache key");

        assert!(key.starts_with("riptide:strategies:v1:"));
    }

    #[test]
    fn test_builder_missing_url() {
        let result = CacheKeyBuilder::new().method("trek").build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("URL is required"));
    }

    #[test]
    fn test_builder_missing_method() {
        let result = CacheKeyBuilder::new().url("https://example.com").build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Method is required"));
    }

    #[test]
    fn test_helper_strategies() {
        let key = generate_strategies_cache_key("https://example.com", "wasm", "write", "v1.0.0")
            .expect("Failed to generate strategies cache key");

        assert!(key.starts_with("riptide:strategies:v1.0.0:"));
    }

    #[test]
    fn test_helper_fetch() {
        let mut opts = BTreeMap::new();
        opts.insert("timeout".to_string(), "30".to_string());

        let key = generate_fetch_cache_key("https://example.com", "v1.0.0", &opts)
            .expect("Failed to generate fetch cache key");

        assert!(key.starts_with("riptide:fetch:v1.0.0:"));
    }

    #[test]
    fn test_helper_wasm() {
        let key = generate_wasm_cache_key("https://example.com", "article", "v1.0.0")
            .expect("Failed to generate wasm cache key");

        assert!(key.starts_with("riptide:wasm:v1.0.0:"));
    }

    #[test]
    fn test_params_conversion() {
        let builder = CacheKeyBuilder::new()
            .url("https://example.com")
            .method("wasm")
            .version("v1.0.0")
            .option("test", "value");

        let params = builder.to_params().unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.method, "wasm");
        assert_eq!(params.version, "v1.0.0");
        assert_eq!(params.options.get("test").unwrap(), "value");
    }
}
